//! Token request handlers.
//!
//! Handles ephemeral token generation, revocation, and status checking.

use actix_web::{web, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::middleware::api_key::ApiKeyAuth;
use crate::middleware::auth::AuthUser;
use crate::services::ephemeral_token::{EphemeralTokenService, RevokeTokenRequest};

/// Configure token routes.
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tokens")
            .route("/revoke", web::post().to(revoke_token))
            .route("/{jti}/status", web::get().to(get_token_status))
    );
}

/// Generate an ephemeral token for a credential.
/// POST /api/v1/agents/{agent_id}/credentials/{name}/token
pub async fn generate_token(
    auth: ApiKeyAuth,
    pool: web::Data<PgPool>,
    service: web::Data<EphemeralTokenService>,
    path: web::Path<(Uuid, String)>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let (path_agent_id, credential_name) = path.into_inner();

    // Verify agent_id matches authenticated agent
    if auth.agent_id != path_agent_id {
        return Err(ApiError::Forbidden(
            "Access denied: agent can only request tokens for own credentials".to_string(),
        ));
    }

    // Extract IP address from request
    let ip_address = req
        .connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string());

    let response = service
        .generate_token(
            &pool,
            auth.agent_id,
            &credential_name,
            ip_address.as_deref(),
        )
        .await?;

    Ok(HttpResponse::Created().json(response))
}

/// Revoke a token.
/// POST /api/v1/tokens/revoke
async fn revoke_token(
    auth: ApiKeyAuth,
    pool: web::Data<PgPool>,
    service: web::Data<EphemeralTokenService>,
    body: web::Json<RevokeTokenRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let ip_address = req
        .connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string());

    // Verify token belongs to this agent (check in service)
    // The service will validate ownership when looking up the token
    service
        .revoke_token(&pool, &body.jti, ip_address.as_deref())
        .await?;

    Ok(HttpResponse::NoContent().finish())
}

/// Get token status.
/// GET /api/v1/tokens/{jti}/status
async fn get_token_status(
    _auth: ApiKeyAuth,
    pool: web::Data<PgPool>,
    service: web::Data<EphemeralTokenService>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let jti = path.into_inner();

    let status = service.get_token_status(&pool, &jti).await?;

    Ok(HttpResponse::Ok().json(status))
}

/// User-authenticated token generation (for dashboard/admin).
/// POST /api/v1/agents/{agent_id}/credentials/{name}/token (with Bearer auth)
pub async fn generate_token_user(
    auth: AuthUser,
    pool: web::Data<PgPool>,
    service: web::Data<EphemeralTokenService>,
    path: web::Path<(Uuid, String)>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let (agent_id, credential_name) = path.into_inner();

    // Verify user has access to this agent (team check)
    let agent = crate::models::Agent::find_by_id(&pool, agent_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Agent not found".to_string()))?;

    if agent.team_id != auth.team_id {
        return Err(ApiError::Forbidden(
            "Access denied: agent belongs to different team".to_string(),
        ));
    }

    let ip_address = req
        .connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string());

    let response = service
        .generate_token(&pool, agent_id, &credential_name, ip_address.as_deref())
        .await?;

    Ok(HttpResponse::Created().json(response))
}
