//! Ephemeral Token Service.
//!
//! Handles generation, verification, and revocation of short-lived credential tokens.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::models::{Agent, Credential, EphemeralToken, TokenUsageLog};
use crate::services::encryption::EncryptionService;
use crate::utils::aad::AadGenerator;

/// Default token TTL in seconds (5 minutes).
const DEFAULT_TOKEN_TTL_SECONDS: i64 = 300;

/// Ephemeral token JWT claims.
#[derive(Debug, Serialize, Deserialize)]
pub struct EphemeralTokenClaims {
    /// Subject: credential_id
    pub sub: String,
    /// Agent ID
    pub agent_id: String,
    /// Team ID
    pub team_id: String,
    /// Decrypted secret (plaintext)
    pub secret: String,
    /// Credential type
    pub credential_type: String,
    /// Credential name
    pub credential_name: String,
    /// Expiration timestamp (Unix)
    pub exp: i64,
    /// Issued at timestamp (Unix)
    pub iat: i64,
    /// JWT ID (for revocation tracking)
    pub jti: String,
    /// Token type
    pub token_type: String,
}

/// Response for token generation.
#[derive(Debug, Serialize, Deserialize)]
pub struct EphemeralTokenResponse {
    pub token: String,
    pub expires_in: i64,
    pub credential_type: String,
    pub credential_name: String,
    pub token_type: String,
}

/// Token status response.
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenStatus {
    pub jti: String,
    pub status: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Request to revoke a token.
#[derive(Debug, Serialize, Deserialize)]
pub struct RevokeTokenRequest {
    pub jti: String,
}

/// Verified token result.
#[derive(Debug)]
pub struct VerifiedToken {
    pub agent_id: Uuid,
    pub credential_id: Uuid,
    pub team_id: Uuid,
    pub secret: String,
    pub credential_type: String,
    pub jti: String,
}

/// Service for managing ephemeral tokens.
pub struct EphemeralTokenService {
    jwt_secret: String,
    encryption: Arc<EncryptionService>,
    token_ttl_seconds: i64,
}

impl EphemeralTokenService {
    /// Create a new ephemeral token service.
    pub fn new(jwt_secret: String, encryption: Arc<EncryptionService>) -> Self {
        Self {
            jwt_secret,
            encryption,
            token_ttl_seconds: DEFAULT_TOKEN_TTL_SECONDS,
        }
    }

    /// Create with custom TTL (for testing).
    pub fn with_ttl(jwt_secret: String, encryption: Arc<EncryptionService>, ttl_seconds: i64) -> Self {
        Self {
            jwt_secret,
            encryption,
            token_ttl_seconds: ttl_seconds,
        }
    }

    /// Generate an ephemeral token for a credential.
    pub async fn generate_token(
        &self,
        pool: &PgPool,
        agent_id: Uuid,
        credential_name: &str,
        ip_address: Option<&str>,
    ) -> Result<EphemeralTokenResponse, ApiError> {
        // 1. Find agent and verify active
        let agent = Agent::find_by_id(pool, agent_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Agent not found".to_string()))?;

        if agent.status != "active" {
            return Err(ApiError::Forbidden("Agent is not active".to_string()));
        }

        // 2. Find credential by name for this agent
        let credential = Credential::find_by_name(pool, agent_id, credential_name)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("Credential '{}' not found", credential_name)))?;

        if !credential.is_active {
            return Err(ApiError::Forbidden("Credential is not active".to_string()));
        }

        // 3. Decrypt credential secret
        let aad = AadGenerator::generate(agent_id, credential.id);
        let plaintext_bytes = self.encryption
            .decrypt(&credential.encrypted_value, &aad)
            .map_err(|e| ApiError::InternalError(format!("Decryption failed: {}", e)))?;

        let secret = String::from_utf8(plaintext_bytes)
            .map_err(|_| ApiError::InternalError("Invalid UTF-8 in secret".to_string()))?;

        // 4. Generate unique JTI
        let jti = Uuid::new_v4().to_string();

        // 5. Create JWT claims
        let now = Utc::now();
        let expires_at = now + Duration::seconds(self.token_ttl_seconds);

        let claims = EphemeralTokenClaims {
            sub: credential.id.to_string(),
            agent_id: agent_id.to_string(),
            team_id: credential.team_id.to_string(),
            secret, // Plaintext secret in JWT payload
            credential_type: credential.credential_type.clone(),
            credential_name: credential.name.clone(),
            exp: expires_at.timestamp(),
            iat: now.timestamp(),
            jti: jti.clone(),
            token_type: "ephemeral".to_string(),
        };

        // 6. Encode JWT
        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| ApiError::InternalError(format!("Token encoding failed: {}", e)))?;

        // 7. Store token record (signature = first 32 chars for verification)
        let token_signature = if token.len() >= 32 {
            &token[..32]
        } else {
            &token
        };

        EphemeralToken::create(
            pool,
            &jti,
            agent_id,
            credential.id,
            credential.team_id,
            token_signature,
            expires_at,
        )
        .await?;

        // 8. Log token issuance (WITHOUT secret!)
        TokenUsageLog::log_action(
            pool,
            &jti,
            agent_id,
            credential.team_id,
            "issued",
            ip_address,
        )
        .await?;

        // 9. Update agent last used
        Agent::update_last_used(pool, agent_id).await?;

        // 10. Update credential last accessed
        Credential::update_last_accessed(pool, credential.id).await?;

        Ok(EphemeralTokenResponse {
            token,
            expires_in: self.token_ttl_seconds,
            credential_type: credential.credential_type,
            credential_name: credential.name,
            token_type: "Bearer".to_string(),
        })
    }

    /// Verify an ephemeral token.
    pub async fn verify_token(
        &self,
        pool: &PgPool,
        token: &str,
        ip_address: Option<&str>,
    ) -> Result<VerifiedToken, ApiError> {
        // 1. Decode and verify JWT
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        let token_data = decode::<EphemeralTokenClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        )
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                ApiError::Unauthorized("Token has expired".to_string())
            }
            jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                ApiError::Unauthorized("Invalid token signature".to_string())
            }
            _ => ApiError::Unauthorized(format!("Token verification failed: {}", e)),
        })?;

        let claims = token_data.claims;

        // 2. Verify token_type
        if claims.token_type != "ephemeral" {
            return Err(ApiError::Unauthorized("Invalid token type".to_string()));
        }

        // 3. Check database for revocation
        let db_token = EphemeralToken::find_by_jti(pool, &claims.jti)
            .await?
            .ok_or_else(|| ApiError::Unauthorized("Token not found".to_string()))?;

        if db_token.status == "revoked" {
            return Err(ApiError::Unauthorized("Token has been revoked".to_string()));
        }

        if db_token.status == "expired" {
            return Err(ApiError::Unauthorized("Token has expired".to_string()));
        }

        // 4. Parse UUIDs
        let agent_id = Uuid::parse_str(&claims.agent_id)
            .map_err(|_| ApiError::InternalError("Invalid agent_id in token".to_string()))?;
        let credential_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| ApiError::InternalError("Invalid credential_id in token".to_string()))?;
        let team_id = Uuid::parse_str(&claims.team_id)
            .map_err(|_| ApiError::InternalError("Invalid team_id in token".to_string()))?;

        // 5. Log token usage
        TokenUsageLog::log_action(
            pool,
            &claims.jti,
            agent_id,
            team_id,
            "used",
            ip_address,
        )
        .await?;

        Ok(VerifiedToken {
            agent_id,
            credential_id,
            team_id,
            secret: claims.secret,
            credential_type: claims.credential_type,
            jti: claims.jti,
        })
    }

    /// Revoke a token by JTI.
    pub async fn revoke_token(
        &self,
        pool: &PgPool,
        jti: &str,
        ip_address: Option<&str>,
    ) -> Result<(), ApiError> {
        // 1. Find token
        let token = EphemeralToken::find_by_jti(pool, jti)
            .await?
            .ok_or_else(|| ApiError::NotFound("Token not found".to_string()))?;

        // 2. Check if already revoked
        if token.status == "revoked" {
            return Ok(()); // Idempotent
        }

        // 3. Revoke token
        EphemeralToken::revoke(pool, jti).await?;

        // 4. Log revocation
        TokenUsageLog::log_action(
            pool,
            jti,
            token.agent_id,
            token.team_id,
            "revoked",
            ip_address,
        )
        .await?;

        Ok(())
    }

    /// Cleanup expired tokens.
    pub async fn cleanup_expired_tokens(&self, pool: &PgPool) -> Result<i64, ApiError> {
        EphemeralToken::cleanup_expired(pool).await
    }

    /// Get token status by JTI.
    pub async fn get_token_status(
        &self,
        pool: &PgPool,
        jti: &str,
    ) -> Result<TokenStatus, ApiError> {
        let token = EphemeralToken::find_by_jti(pool, jti)
            .await?
            .ok_or_else(|| ApiError::NotFound("Token not found".to_string()))?;

        // Check if token should be marked as expired
        let status = if token.status == "active" && token.expires_at < Utc::now() {
            "expired".to_string()
        } else {
            token.status
        };

        Ok(TokenStatus {
            jti: token.jti,
            status,
            expires_at: token.expires_at,
            created_at: token.created_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ephemeral_token_claims_serialization() {
        let claims = EphemeralTokenClaims {
            sub: "cred-123".to_string(),
            agent_id: "agent-456".to_string(),
            team_id: "team-789".to_string(),
            secret: "my-secret".to_string(),
            credential_type: "password".to_string(),
            credential_name: "db-password".to_string(),
            exp: 1234567890,
            iat: 1234567890,
            jti: "jti-abc".to_string(),
            token_type: "ephemeral".to_string(),
        };

        let json = serde_json::to_string(&claims).unwrap();
        assert!(json.contains("ephemeral"));
        assert!(json.contains("my-secret"));
    }

    #[test]
    fn test_ephemeral_token_response_serialization() {
        let response = EphemeralTokenResponse {
            token: "jwt.token.here".to_string(),
            expires_in: 300,
            credential_type: "api_key".to_string(),
            credential_name: "openai-key".to_string(),
            token_type: "Bearer".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Bearer"));
        assert!(json.contains("300"));
    }

    #[test]
    fn test_token_status_serialization() {
        let status = TokenStatus {
            jti: "abc-123".to_string(),
            status: "active".to_string(),
            expires_at: Utc::now(),
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("active"));
    }
}
