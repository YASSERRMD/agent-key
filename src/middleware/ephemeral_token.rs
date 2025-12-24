//! Ephemeral token authentication middleware.
//!
//! Extracts and validates ephemeral tokens from the Authorization header.

use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::services::ephemeral_token::EphemeralTokenService;

/// Authenticated ephemeral token identity.
#[derive(Debug, Clone)]
pub struct EphemeralTokenAuth {
    pub agent_id: Uuid,
    pub credential_id: Uuid,
    pub team_id: Uuid,
    pub secret: String,
    pub credential_type: String,
    pub jti: String,
}

impl FromRequest for EphemeralTokenAuth {
    type Error = ApiError;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        // 1. Extract Bearer token from Authorization header
        let token = match extract_bearer_token(req) {
            Some(t) => t,
            None => {
                return Box::pin(async {
                    Err(ApiError::Unauthorized(
                        "Missing ephemeral token in Authorization header".to_string(),
                    ))
                })
            }
        };

        // 2. Get dependencies
        let pool = match req.app_data::<web::Data<PgPool>>() {
            Some(p) => p.clone(),
            None => {
                return Box::pin(async {
                    Err(ApiError::InternalError("Database pool not found".to_string()))
                })
            }
        };

        let service = match req.app_data::<web::Data<EphemeralTokenService>>() {
            Some(s) => s.clone(),
            None => {
                return Box::pin(async {
                    Err(ApiError::InternalError(
                        "EphemeralTokenService not found".to_string(),
                    ))
                })
            }
        };

        // 3. Extract IP address
        let ip_address = req
            .connection_info()
            .realip_remote_addr()
            .map(|s| s.to_string());

        Box::pin(async move {
            let verified = service
                .verify_token(&pool, &token, ip_address.as_deref())
                .await?;

            Ok(EphemeralTokenAuth {
                agent_id: verified.agent_id,
                credential_id: verified.credential_id,
                team_id: verified.team_id,
                secret: verified.secret,
                credential_type: verified.credential_type,
                jti: verified.jti,
            })
        })
    }
}

/// Extract Bearer token from Authorization header.
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
    fn test_ephemeral_token_auth_fields() {
        let auth = EphemeralTokenAuth {
            agent_id: Uuid::new_v4(),
            credential_id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            secret: "my-secret".to_string(),
            credential_type: "password".to_string(),
            jti: "jti-123".to_string(),
        };

        assert_eq!(auth.secret, "my-secret");
        assert_eq!(auth.credential_type, "password");
    }
}
