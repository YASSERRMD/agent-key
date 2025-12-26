//! User profile handlers.
//!
//! REST endpoints for user profile management.

use actix_web::{get, patch, post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::errors::ApiError;
use crate::middleware::auth::AuthUser;
use crate::models::{log_audit_event, User};
use crate::services::auth::AuthService;
use std::sync::Arc;

/// Request body for updating user profile.
#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
}

/// Request body for changing password.
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

/// Response for profile.
#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub team_id: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: String,
}

/// GET /api/v1/users/me
///
/// Get current user's profile.
#[get("/me")]
pub async fn get_profile(
    pool: web::Data<PgPool>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let user = User::find_by_id(pool.get_ref(), auth.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ProfileResponse {
        id: user.id.to_string(),
        email: user.email,
        name: user.name,
        team_id: user.team_id.to_string(),
        role: user.role,
        is_active: user.is_active,
        created_at: user.created_at.to_rfc3339(),
    }))
}

/// PATCH /api/v1/users/me
///
/// Update current user's profile (name only).
#[patch("/me")]
pub async fn update_profile(
    pool: web::Data<PgPool>,
    auth: AuthUser,
    body: web::Json<UpdateProfileRequest>,
) -> Result<HttpResponse, ApiError> {
    let user = User::find_by_id(pool.get_ref(), auth.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    // Update name if provided
    let name = body.name.clone().or(user.name.clone());

    // Update in database
    sqlx::query(
        r#"UPDATE users SET name = $1, updated_at = NOW() WHERE id = $2"#
    )
    .bind(&name)
    .bind(auth.user_id)
    .execute(pool.get_ref())
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    // Log audit event
    let _ = log_audit_event(
        pool.get_ref(),
        auth.team_id,
        Some(auth.user_id),
        "user.update",
        Some("user"),
        Some(auth.user_id),
        Some("User profile updated"),
        None,
    )
    .await;

    // Fetch updated user
    let updated_user = User::find_by_id(pool.get_ref(), auth.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ProfileResponse {
        id: updated_user.id.to_string(),
        email: updated_user.email,
        name: updated_user.name,
        team_id: updated_user.team_id.to_string(),
        role: updated_user.role,
        is_active: updated_user.is_active,
        created_at: updated_user.created_at.to_rfc3339(),
    }))
}

/// POST /api/v1/users/me/password
///
/// Change current user's password.
#[post("/me/password")]
pub async fn change_password(
    pool: web::Data<PgPool>,
    auth_service: web::Data<Arc<AuthService>>,
    auth: AuthUser,
    body: web::Json<ChangePasswordRequest>,
) -> Result<HttpResponse, ApiError> {
    // Get user
    let user = User::find_by_id(pool.get_ref(), auth.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    // Verify current password
    if !auth_service.verify_password(&body.current_password, &user.password_hash)? {
        return Err(ApiError::Unauthorized("Current password is incorrect".to_string()));
    }

    // Validate new password
    if body.new_password.len() < 12 {
        return Err(ApiError::BadRequest("Password must be at least 12 characters".to_string()));
    }

    // Hash new password
    let new_hash = auth_service.hash_password(&body.new_password)?;

    // Update password
    sqlx::query!(
        r#"UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2"#,
        new_hash,
        auth.user_id
    )
    .execute(pool.get_ref())
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    // Log audit event
    let _ = log_audit_event(
        pool.get_ref(),
        auth.team_id,
        Some(auth.user_id),
        "user.password_change",
        Some("user"),
        Some(auth.user_id),
        Some("Password changed"),
        None,
    )
    .await;

    Ok(HttpResponse::Ok().json(serde_json::json!({"message": "Password changed successfully"})))
}

/// Configure user routes.
///
/// Mounts routes under `/api/v1/users`:
/// - GET /me
/// - PATCH /me
/// - POST /me/password
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .service(get_profile)
            .service(update_profile)
            .service(change_password),
    );
}
