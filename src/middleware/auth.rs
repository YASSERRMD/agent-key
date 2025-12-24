//! Authentication middleware and extractors.
//!
//! Provides JWT-based authentication for protected routes.

use actix_web::{dev::Payload, FromRequest, HttpRequest};
use futures::future::{ready, Ready};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::services::jwt::JwtService;

/// Authenticated user information extracted from JWT.
///
/// Use this as a handler parameter to require authentication.
///
/// # Example
///
/// ```rust,ignore
/// #[get("/api/v1/protected")]
/// async fn protected_route(auth: AuthUser) -> Result<HttpResponse, ApiError> {
///     // auth.user_id, auth.team_id, auth.role are available
///     Ok(HttpResponse::Ok().body("Success"))
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AuthUser {
    /// Authenticated user's ID
    pub user_id: Uuid,

    /// User's team ID
    pub team_id: Uuid,

    /// User's role (admin, developer, viewer)
    pub role: String,
}

impl AuthUser {
    /// Check if user has admin role.
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    /// Check if user has developer role (includes admin).
    pub fn is_developer(&self) -> bool {
        self.role == "admin" || self.role == "developer"
    }

    /// Check if user has viewer role (includes all roles).
    pub fn is_viewer(&self) -> bool {
        self.role == "admin" || self.role == "developer" || self.role == "viewer"
    }
}

impl FromRequest for AuthUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // Get JWT service from app state
        let jwt_service = match req.app_data::<actix_web::web::Data<Arc<JwtService>>>() {
            Some(service) => service.get_ref().clone(),
            None => {
                return ready(Err(ApiError::InternalError(
                    "JWT service not configured".to_string(),
                )));
            }
        };

        // Extract token from Authorization header
        let token = match extract_bearer_token(req) {
            Some(token) => token,
            None => {
                return ready(Err(ApiError::Unauthorized(
                    "Missing authorization token".to_string(),
                )));
            }
        };

        // Verify token and extract claims
        match jwt_service.verify_token(&token) {
            Ok(claims) => {
                let user_id = match claims.user_id() {
                    Ok(id) => id,
                    Err(_) => {
                        return ready(Err(ApiError::Unauthorized(
                            "Invalid user ID in token".to_string(),
                        )));
                    }
                };

                let team_id = match claims.get_team_id() {
                    Ok(id) => id,
                    Err(_) => {
                        return ready(Err(ApiError::Unauthorized(
                            "Invalid team ID in token".to_string(),
                        )));
                    }
                };

                ready(Ok(AuthUser {
                    user_id,
                    team_id,
                    role: claims.role,
                }))
            }
            Err(e) => ready(Err(ApiError::Unauthorized(format!(
                "Invalid token: {}",
                e
            )))),
        }
    }
}

/// Optional authenticated user extractor.
///
/// Use this for routes that work with or without authentication.
///
/// # Example
///
/// ```rust,ignore
/// #[get("/api/v1/public")]
/// async fn public_route(auth: OptionalAuthUser) -> Result<HttpResponse, ApiError> {
///     if let Some(user) = auth.0 {
///         // User is authenticated
///     }
///     Ok(HttpResponse::Ok().body("Success"))
/// }
/// ```
#[derive(Debug, Clone)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

impl FromRequest for OptionalAuthUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // Get JWT service from app state
        let jwt_service = match req.app_data::<actix_web::web::Data<Arc<JwtService>>>() {
            Some(service) => service.get_ref().clone(),
            None => {
                return ready(Ok(OptionalAuthUser(None)));
            }
        };

        // Try to extract token from Authorization header
        let token = match extract_bearer_token(req) {
            Some(token) => token,
            None => {
                return ready(Ok(OptionalAuthUser(None)));
            }
        };

        // Try to verify token
        match jwt_service.verify_token(&token) {
            Ok(claims) => {
                if let (Ok(user_id), Ok(team_id)) = (claims.user_id(), claims.get_team_id()) {
                    ready(Ok(OptionalAuthUser(Some(AuthUser {
                        user_id,
                        team_id,
                        role: claims.role,
                    }))))
                } else {
                    ready(Ok(OptionalAuthUser(None)))
                }
            }
            Err(_) => ready(Ok(OptionalAuthUser(None))),
        }
    }
}

/// Role-based access control checker.
///
/// Use this to enforce that a user has one of the specified roles.
///
/// # Example
///
/// ```rust,ignore
/// async fn admin_only(auth: AuthUser) -> Result<HttpResponse, ApiError> {
///     RequireRole::check(&auth, &["admin"])?;
///     // User is admin
///     Ok(HttpResponse::Ok().body("Admin only"))
/// }
/// ```
pub struct RequireRole;

impl RequireRole {
    /// Check if user has one of the required roles.
    ///
    /// # Arguments
    ///
    /// * `auth` - Authenticated user
    /// * `roles` - List of allowed roles
    ///
    /// # Returns
    ///
    /// `Ok(())` if user has one of the roles, `Err(ApiError::Forbidden)` otherwise.
    pub fn check(auth: &AuthUser, roles: &[&str]) -> Result<(), ApiError> {
        if roles.contains(&auth.role.as_str()) {
            Ok(())
        } else {
            Err(ApiError::Forbidden(format!(
                "Required role: one of {:?}",
                roles
            )))
        }
    }

    /// Require admin role.
    pub fn admin(auth: &AuthUser) -> Result<(), ApiError> {
        Self::check(auth, &["admin"])
    }

    /// Require admin or developer role.
    pub fn developer(auth: &AuthUser) -> Result<(), ApiError> {
        Self::check(auth, &["admin", "developer"])
    }

    /// Require any authenticated role.
    pub fn viewer(auth: &AuthUser) -> Result<(), ApiError> {
        Self::check(auth, &["admin", "developer", "viewer"])
    }
}

/// Extract bearer token from Authorization header.
fn extract_bearer_token(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|auth_str| {
            if auth_str.starts_with("Bearer ") {
                Some(auth_str[7..].to_string())
            } else {
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_user_role_checks() {
        let admin = AuthUser {
            user_id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            role: "admin".to_string(),
        };

        assert!(admin.is_admin());
        assert!(admin.is_developer());
        assert!(admin.is_viewer());

        let developer = AuthUser {
            user_id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            role: "developer".to_string(),
        };

        assert!(!developer.is_admin());
        assert!(developer.is_developer());
        assert!(developer.is_viewer());

        let viewer = AuthUser {
            user_id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            role: "viewer".to_string(),
        };

        assert!(!viewer.is_admin());
        assert!(!viewer.is_developer());
        assert!(viewer.is_viewer());
    }

    #[test]
    fn test_require_role_check() {
        let admin = AuthUser {
            user_id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            role: "admin".to_string(),
        };

        assert!(RequireRole::check(&admin, &["admin"]).is_ok());
        assert!(RequireRole::check(&admin, &["admin", "developer"]).is_ok());
        assert!(RequireRole::admin(&admin).is_ok());
        assert!(RequireRole::developer(&admin).is_ok());
        assert!(RequireRole::viewer(&admin).is_ok());
    }

    #[test]
    fn test_require_role_forbidden() {
        let viewer = AuthUser {
            user_id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            role: "viewer".to_string(),
        };

        assert!(RequireRole::admin(&viewer).is_err());
        assert!(RequireRole::developer(&viewer).is_err());
        assert!(RequireRole::viewer(&viewer).is_ok());
    }
}
