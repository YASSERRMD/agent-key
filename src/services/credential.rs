//! Credential management service.
//!
//! Handles lifecycle of credentials: creation, encryption, rotation, and audit logging.

use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::errors::ApiError;
use crate::models::{
    log_audit_event, Credential, CredentialResponse, CredentialVersion,
    CreateCredentialRequest, DecryptedCredentialResponse, PaginatedResponse,
    RotateCredentialRequest, UpdateCredentialRequest, VersionSummary, Team,
};
use crate::services::encryption::EncryptionService;
use crate::utils::aad::AadGenerator;

/// Service for managing credentials.
pub struct CredentialService {
    encryption: Arc<EncryptionService>,
}

impl CredentialService {
    pub fn new(encryption: Arc<EncryptionService>) -> Self {
        Self { encryption }
    }

    /// Create a new credential.
    pub async fn create_credential(
        &self,
        pool: &PgPool,
        agent_id: Uuid,
        team_id: Uuid,
        created_by: Uuid,
        request: CreateCredentialRequest,
    ) -> Result<CredentialResponse, ApiError> {
        // 1. Validate request
        request.validate().map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // 2. Check team credential quota
        let team = Team::find_by_id(pool, team_id).await?
            .ok_or_else(|| ApiError::NotFound("Team not found".to_string()))?;
        
        if !team.check_credential_quota(pool).await? {
            return Err(ApiError::Conflict("Team credential limit reached".to_string()));
        }

        // 3. Encrypt secret
        let credential_id = Uuid::new_v4();
        let aad = AadGenerator::generate(agent_id, credential_id);
        
        let encrypted_value = self.encryption
            .encrypt(request.secret.as_bytes(), &aad)
            .map_err(|e| ApiError::InternalError(format!("Encryption failed: {}", e)))?;

        // 4. Create credential
        let credential = Credential::create(
            pool,
            credential_id,
            agent_id,
            team_id,
            &request.name,
            &request.credential_type,
            request.description,
            encrypted_value,
            created_by,
            request.rotation_enabled.unwrap_or(false),
            request.rotation_interval_days,
        )
        .await?;

        // 5. Log audit event
        log_audit_event(
            pool,
            team_id,
            Some(created_by),
            "credential.create",
            Some("credential"),
            Some(credential.id),
            Some(&format!("Created credential '{}'", credential.name)),
            None,
        )
        .await?;

        Ok(credential.to_response())
    }

    /// Get a credential by ID.
    pub async fn get_credential(
        &self,
        pool: &PgPool,
        team_id: Uuid,
        credential_id: Uuid,
    ) -> Result<CredentialResponse, ApiError> {
        let credential = Credential::find_by_id(pool, credential_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Credential not found".to_string()))?;

        if credential.team_id != team_id {
            return Err(ApiError::Forbidden("Access denied to this credential".to_string()));
        }

        // Update last accessed
        Credential::update_last_accessed(pool, credential_id).await?;

        // Log audit event
        log_audit_event(
            pool,
            team_id,
            None,
            "credential.read",
            Some("credential"),
            Some(credential_id),
            None,
            None,
        )
        .await?;

        Ok(credential.to_response())
    }

    /// Decrypt a credential.
    pub async fn decrypt_credential(
        &self,
        pool: &PgPool,
        team_id: Uuid,
        credential_id: Uuid,
    ) -> Result<DecryptedCredentialResponse, ApiError> {
        let credential = Credential::find_by_id(pool, credential_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Credential not found".to_string()))?;

        if credential.team_id != team_id {
            return Err(ApiError::Forbidden("Access denied to this credential".to_string()));
        }

        // Decrypt
        let aad = AadGenerator::generate(credential.agent_id, credential.id);
        let plaintext_bytes = self.encryption
            .decrypt(&credential.encrypted_value, &aad)
            .map_err(|e| ApiError::InternalError(format!("Decryption failed: {}", e)))?;
        
        let secret = String::from_utf8(plaintext_bytes)
            .map_err(|_| ApiError::InternalError("Invalid UTF-8 in secret".to_string()))?;

        // Update last accessed
        Credential::update_last_accessed(pool, credential_id).await?;

        // Log audit event (CRITICAL: Do NOT log the secret)
        log_audit_event(
            pool,
            team_id,
            None,
            "credential.decrypt",
            Some("credential"),
            Some(credential_id),
            Some("Secret decrypted"),
            None,
        )
        .await?;

        Ok(DecryptedCredentialResponse {
            id: credential.id,
            agent_id: credential.agent_id,
            name: credential.name,
            credential_type: credential.credential_type,
            description: credential.description,
            secret,
            is_active: credential.is_active,
            created_at: credential.created_at,
        })
    }

    /// List credentials for an agent.
    pub async fn list_credentials(
        &self,
        pool: &PgPool,
        agent_id: Uuid,
        page: Option<i32>,
        limit: Option<i32>,
    ) -> Result<PaginatedResponse<CredentialResponse>, ApiError> {
        let page = page.unwrap_or(1).max(1);
        let limit = limit.unwrap_or(20).clamp(1, 100);

        let (credentials, total) = Credential::find_by_agent(pool, agent_id, page, limit).await?;
        
        let total_pages = (total as f64 / limit as f64).ceil() as i32;

        Ok(PaginatedResponse {
            data: credentials.into_iter().map(|c| c.to_response()).collect(),
            total,
            page,
            limit,
            pages: total_pages,
        })
    }

    /// Update a credential.
    pub async fn update_credential(
        &self,
        pool: &PgPool,
        team_id: Uuid,
        credential_id: Uuid,
        request: UpdateCredentialRequest,
    ) -> Result<CredentialResponse, ApiError> {
        let credential = Credential::find_by_id(pool, credential_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Credential not found".to_string()))?;

        if credential.team_id != team_id {
            return Err(ApiError::Forbidden("Access denied to this credential".to_string()));
        }

        // If secret is updated, rotate it
        if let Some(new_secret) = &request.secret {
            if new_secret.is_empty() {
                return Err(ApiError::ValidationError("Secret cannot be empty".to_string()));
            }

            let aad = AadGenerator::generate(credential.agent_id, credential.id);
            let encrypted_value = self.encryption
                .encrypt(new_secret.as_bytes(), &aad)
                .map_err(|e| ApiError::InternalError(format!("Encryption failed: {}", e)))?;

            Credential::rotate(pool, credential_id, encrypted_value).await?;
        }

        // Update metadata
        let updated_credential = Credential::update(
            pool,
            credential_id,
            request.description.clone(),
            request.rotation_enabled,
            request.rotation_interval_days,
        )
        .await?;

        // Log audit event
        let mut changes = Vec::new();
        if request.description.is_some() { changes.push("description"); }
        if request.rotation_enabled.is_some() { changes.push("rotation_enabled"); }
        if request.secret.is_some() { changes.push("secret (rotated)"); }
        
        log_audit_event(
            pool,
            team_id,
            None,
            "credential.update",
            Some("credential"),
            Some(credential_id),
            Some(&format!("Updated: {}", changes.join(", "))),
            None,
        )
        .await?;

        Ok(updated_credential.to_response())
    }

    /// Delete a credential.
    pub async fn delete_credential(
        &self,
        pool: &PgPool,
        team_id: Uuid,
        credential_id: Uuid,
    ) -> Result<(), ApiError> {
        let credential = Credential::find_by_id(pool, credential_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Credential not found".to_string()))?;

        if credential.team_id != team_id {
            return Err(ApiError::Forbidden("Access denied to this credential".to_string()));
        }

        Credential::soft_delete(pool, credential_id).await?;

        log_audit_event(
            pool,
            team_id,
            None,
            "credential.delete",
            Some("credential"),
            Some(credential_id),
            Some("Soft deleted credential"),
            None,
        )
        .await?;

        Ok(())
    }

    /// Rotate credential manually.
    pub async fn rotate_credential(
        &self,
        pool: &PgPool,
        team_id: Uuid,
        credential_id: Uuid,
        request: RotateCredentialRequest,
    ) -> Result<CredentialResponse, ApiError> {
        let credential = Credential::find_by_id(pool, credential_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Credential not found".to_string()))?;

        if credential.team_id != team_id {
            return Err(ApiError::Forbidden("Access denied to this credential".to_string()));
        }

        if !credential.rotation_enabled {
            return Err(ApiError::ValidationError("Rotation is not enabled for this credential".to_string()));
        }
        
        if request.new_secret.is_empty() {
             return Err(ApiError::ValidationError("New secret cannot be empty".to_string()));
        }

        let aad = AadGenerator::generate(credential.agent_id, credential.id);
        let encrypted_value = self.encryption
            .encrypt(request.new_secret.as_bytes(), &aad)
            .map_err(|e| ApiError::InternalError(format!("Encryption failed: {}", e)))?;

        let updated = Credential::rotate(pool, credential_id, encrypted_value).await?;

        log_audit_event(
            pool,
            team_id,
            None,
            "credential.rotate",
            Some("credential"),
            Some(credential_id),
            Some("Manual rotation"),
            None,
        )
        .await?;

        Ok(updated.to_response())
    }

    /// Get versions history (without secrets).
    pub async fn get_versions(
        &self,
        pool: &PgPool,
        team_id: Uuid,
        credential_id: Uuid,
    ) -> Result<Vec<VersionSummary>, ApiError> {
        let credential = Credential::find_by_id(pool, credential_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Credential not found".to_string()))?;

        if credential.team_id != team_id {
            return Err(ApiError::Forbidden("Access denied to this credential".to_string()));
        }
        
        let versions = sqlx::query_as::<_, VersionSummary>(
            r#"
            SELECT id, version, status, created_at 
            FROM credential_versions 
            WHERE credential_id = $1
            ORDER BY version DESC
            "#
        )
        .bind(credential_id)
        .fetch_all(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(versions)
    }
}
