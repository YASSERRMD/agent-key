//! Team API Keys handlers.
//!
//! REST endpoints for team-level API key management.

use actix_web::{delete, get, post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::middleware::auth::AuthUser;
use crate::models::log_audit_event;
use crate::services::auth::AuthService;
use std::sync::Arc;

/// Request body for creating an API key.
#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub expires_in_days: Option<i32>,
}

/// Response for API key creation.
#[derive(Debug, Serialize)]
pub struct CreateApiKeyResponse {
    pub id: String,
    pub api_key: String,
}

/// API key info (without the full key).
#[derive(Debug, Serialize)]
pub struct ApiKeyInfo {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub status: String,
    pub last_used: Option<String>,
    pub created_at: String,
}

/// GET /api/v1/api-keys
///
/// List all API keys for the current team.
#[get("")]
pub async fn list_api_keys(
    pool: web::Data<PgPool>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let keys = sqlx::query_as!(
        ApiKeyRow,
        r#"
        SELECT id, name, key_prefix, status, last_used_at, created_at
        FROM api_keys
        WHERE team_id = $1 AND deleted_at IS NULL
        ORDER BY created_at DESC
        "#,
        auth.team_id
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let response: Vec<ApiKeyInfo> = keys
        .into_iter()
        .map(|k| ApiKeyInfo {
            id: k.id.to_string(),
            name: k.name,
            key_prefix: k.key_prefix,
            status: k.status,
            last_used: k.last_used_at.map(|t| t.to_rfc3339()),
            created_at: k.created_at.to_rfc3339(),
        })
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Debug)]
struct ApiKeyRow {
    id: Uuid,
    name: String,
    key_prefix: String,
    status: String,
    last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
}

/// POST /api/v1/api-keys
///
/// Create a new API key for the current team.
#[post("")]
pub async fn create_api_key(
    pool: web::Data<PgPool>,
    auth_service: web::Data<Arc<AuthService>>,
    auth: AuthUser,
    body: web::Json<CreateApiKeyRequest>,
) -> Result<HttpResponse, ApiError> {
    // Generate API key
    let api_key = format!("ak_{}", Uuid::new_v4().to_string().replace("-", ""));
    let key_prefix = api_key[..12].to_string();
    let key_hash = auth_service.hash_api_key(&api_key)?;

    let key_id = Uuid::new_v4();
    let expires_at = body.expires_in_days.map(|days| {
        chrono::Utc::now() + chrono::Duration::days(days as i64)
    });

    sqlx::query!(
        r#"
        INSERT INTO api_keys (id, team_id, user_id, name, key_hash, key_prefix, status, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6, 'active', $7)
        "#,
        key_id,
        auth.team_id,
        auth.user_id,
        body.name,
        key_hash,
        key_prefix,
        expires_at
    )
    .execute(pool.get_ref())
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    // Log audit event
    let _ = log_audit_event(
        pool.get_ref(),
        auth.team_id,
        Some(auth.user_id),
        "api_key.create",
        Some("api_key"),
        Some(key_id),
        Some(&format!("API key created: {}", body.name)),
        None,
    )
    .await;

    Ok(HttpResponse::Created().json(CreateApiKeyResponse {
        id: key_id.to_string(),
        api_key,
    }))
}

/// DELETE /api/v1/api-keys/{id}
///
/// Revoke an API key.
#[delete("/{id}")]
pub async fn revoke_api_key(
    pool: web::Data<PgPool>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let key_id = path.into_inner();

    // Update status to revoked
    let result = sqlx::query!(
        r#"
        UPDATE api_keys 
        SET status = 'revoked', deleted_at = NOW(), updated_at = NOW() 
        WHERE id = $1 AND team_id = $2
        "#,
        key_id,
        auth.team_id
    )
    .execute(pool.get_ref())
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("API key not found".to_string()));
    }

    // Log audit event
    let _ = log_audit_event(
        pool.get_ref(),
        auth.team_id,
        Some(auth.user_id),
        "api_key.revoke",
        Some("api_key"),
        Some(key_id),
        Some("API key revoked"),
        None,
    )
    .await;

    Ok(HttpResponse::NoContent().finish())
}

/// Configure API key routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api-keys")
            .service(list_api_keys)
            .service(create_api_key)
            .service(revoke_api_key),
    );
}
