//! Authentication handlers.
//!
//! REST endpoints for user registration, login, token refresh, and profile.

use actix_web::{get, post, web, HttpResponse};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::info;

use crate::errors::ApiError;
use crate::middleware::auth::AuthUser;
use crate::models::{log_audit_event, LoginRequest, RefreshTokenRequest, RegisterRequest, User};
use crate::services::auth::AuthService;

/// POST /api/v1/auth/register
///
/// Register a new user and create their team.
///
/// # Request Body
///
/// ```json
/// {
///     "email": "user@example.com",
///     "password": "MyStr0ng!Pass",
///     "team_name": "My Team"  // optional
/// }
/// ```
///
/// # Response
///
/// 201 Created with AuthResponse
#[post("/register")]
pub async fn register(
    pool: web::Data<PgPool>,
    auth_service: web::Data<Arc<AuthService>>,
    body: web::Json<RegisterRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = auth_service.register(pool.get_ref(), body.into_inner()).await?;

    Ok(HttpResponse::Created().json(response))
}

/// POST /api/v1/auth/login
///
/// Authenticate user with email and password.
///
/// # Request Body
///
/// ```json
/// {
///     "email": "user@example.com",
///     "password": "MyStr0ng!Pass"
/// }
/// ```
///
/// # Response
///
/// 200 OK with AuthResponse
#[post("/login")]
pub async fn login(
    pool: web::Data<PgPool>,
    auth_service: web::Data<Arc<AuthService>>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = auth_service.login(pool.get_ref(), body.into_inner()).await?;

    Ok(HttpResponse::Ok().json(response))
}

/// POST /api/v1/auth/refresh
///
/// Refresh access token using refresh token.
///
/// # Request Body
///
/// ```json
/// {
///     "refresh_token": "eyJ..."
/// }
/// ```
///
/// # Response
///
/// 200 OK with RefreshResponse
#[post("/refresh")]
pub async fn refresh_token(
    auth_service: web::Data<Arc<AuthService>>,
    body: web::Json<RefreshTokenRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = auth_service.refresh_token(&body.refresh_token)?;

    Ok(HttpResponse::Ok().json(response))
}

/// GET /api/v1/auth/me
///
/// Get current user's profile.
///
/// Requires valid Bearer token in Authorization header.
///
/// # Response
///
/// 200 OK with UserProfile
#[get("/me")]
pub async fn get_profile(
    pool: web::Data<PgPool>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let user = User::find_by_id(pool.get_ref(), auth.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    Ok(HttpResponse::Ok().json(user.to_profile()))
}

/// POST /api/v1/auth/logout
///
/// Log out current user.
///
/// Requires valid Bearer token in Authorization header.
/// Client should delete the token after this call.
///
/// # Response
///
/// 204 No Content
#[post("/logout")]
pub async fn logout(
    pool: web::Data<PgPool>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    // Log logout event
    if let Err(e) = log_audit_event(
        pool.get_ref(),
        auth.team_id,
        Some(auth.user_id),
        "logout",
        Some("user"),
        Some(auth.user_id),
        Some("User logged out"),
        None,
    )
    .await
    {
        tracing::warn!("Failed to log logout event: {}", e);
    }

    info!("User logged out: {}", auth.user_id);

    Ok(HttpResponse::NoContent().finish())
}

/// Configure authentication routes.
///
/// Mounts routes under `/api/v1/auth`:
/// - POST /register
/// - POST /login
/// - POST /refresh
/// - GET /me
/// - POST /logout
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(register)
            .service(login)
            .service(refresh_token)
            .service(get_profile)
            .service(logout),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_request_parsing() {
        let json = r#"{"email":"test@example.com","password":"MyStr0ng!Pass","team_name":"Test"}"#;
        let request: Result<RegisterRequest, _> = serde_json::from_str(json);
        assert!(request.is_ok());
        
        let req = request.unwrap();
        assert_eq!(req.email, "test@example.com");
        assert_eq!(req.team_name, Some("Test".to_string()));
    }

    #[test]
    fn test_login_request_parsing() {
        let json = r#"{"email":"test@example.com","password":"password"}"#;
        let request: Result<LoginRequest, _> = serde_json::from_str(json);
        assert!(request.is_ok());
    }

    #[test]
    fn test_refresh_token_request_parsing() {
        let json = r#"{"refresh_token":"eyJ..."}"#;
        let request: Result<RefreshTokenRequest, _> = serde_json::from_str(json);
        assert!(request.is_ok());
    }
}
