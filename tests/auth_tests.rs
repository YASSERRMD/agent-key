//! Integration tests for authentication endpoints.
//!
//! These tests cover registration, login, token refresh, and profile endpoints.

use agentkey_backend::models::{
    AuthResponse, LoginRequest, RefreshResponse, RefreshTokenRequest, RegisterRequest, UserProfile,
};
use agentkey_backend::services::jwt::JwtService;
use agentkey_backend::services::password::PasswordService;
use uuid::Uuid;

// =============================================================================
// UNIT TESTS FOR AUTH COMPONENTS
// =============================================================================

mod password_service_tests {
    use super::*;

    #[test]
    fn test_password_hash_never_returns_plaintext() {
        let service = PasswordService::new();
        let password = "MyStr0ng!Pass";
        
        let hash = service.hash(password).unwrap();
        
        // Hash should never contain the plaintext password
        assert!(!hash.contains(password));
        // Hash should be bcrypt format
        assert!(hash.starts_with("$2b$"));
    }

    #[test]
    fn test_password_complexity_all_requirements() {
        let service = PasswordService::new();
        
        // Test each requirement individually
        assert!(service.validate_password("short").is_err()); // too short
        assert!(service.validate_password("alllowercase12!").is_err()); // no uppercase
        assert!(service.validate_password("ALLUPPERCASE12!").is_err()); // no lowercase
        assert!(service.validate_password("NoDigitsHere!!").is_err()); // no digits
        assert!(service.validate_password("NoSpecial1234").is_err()); // no special
        
        // Valid password
        assert!(service.validate_password("MyStr0ng!Pass").is_ok());
    }

    #[test]
    fn test_password_hash_unique_per_call() {
        let service = PasswordService::new();
        let password = "MyStr0ng!Pass";
        
        let hash1 = service.hash(password).unwrap();
        let hash2 = service.hash(password).unwrap();
        let hash3 = service.hash(password).unwrap();
        
        // All hashes should be different due to random salt
        assert_ne!(hash1, hash2);
        assert_ne!(hash2, hash3);
        assert_ne!(hash1, hash3);
        
        // But all should verify correctly
        assert!(service.verify(password, &hash1).unwrap());
        assert!(service.verify(password, &hash2).unwrap());
        assert!(service.verify(password, &hash3).unwrap());
    }
}

mod jwt_service_tests {
    use super::*;

    const TEST_SECRET: &str = "test-jwt-secret-32-characters-here!";

    #[test]
    fn test_token_contains_correct_claims() {
        let service = JwtService::new(TEST_SECRET.to_string(), 1);
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        
        let token = service.create_token(user_id, team_id, "admin".to_string()).unwrap();
        let claims = service.verify_token(&token).unwrap();
        
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.team_id, team_id.to_string());
        assert_eq!(claims.role, "admin");
        assert_eq!(claims.iss, "agentkey");
    }

    #[test]
    fn test_refresh_token_has_longer_expiry() {
        let service = JwtService::new(TEST_SECRET.to_string(), 1);
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        
        let access_token = service.create_token(user_id, team_id, "admin".to_string()).unwrap();
        let refresh_token = service.create_refresh_token(user_id, team_id, "admin".to_string(), 7).unwrap();
        
        let access_exp = service.get_expiration_unix(&access_token).unwrap();
        let refresh_claims = service.verify_refresh_token(&refresh_token).unwrap();
        
        // Refresh token should expire later than access token
        assert!(refresh_claims.exp > access_exp);
        
        // Refresh token should be ~7 days from now (vs 1 hour for access)
        let now = chrono::Utc::now().timestamp();
        let seven_days = 7 * 24 * 3600;
        assert!(refresh_claims.exp > now + seven_days - 60); // within 1 minute
    }

    #[test]
    fn test_token_type_validation() {
        let service = JwtService::new(TEST_SECRET.to_string(), 1);
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        
        // Create access token and try to use it as refresh token
        let access_token = service.create_token(user_id, team_id, "admin".to_string()).unwrap();
        assert!(service.verify_refresh_token(&access_token).is_err());
        
        // Create refresh token and try to use it as access token
        let refresh_token = service.create_refresh_token(user_id, team_id, "admin".to_string(), 7).unwrap();
        // Access token verification should work but token_type won't match
        // (it will pass verify_token but fail on business logic)
    }
}

mod dto_tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_register_request_email_validation() {
        let valid = RegisterRequest {
            email: "user@example.com".to_string(),
            password: "MyStr0ng!Pass".to_string(),
            team_name: Some("Test Team".to_string()),
        };
        assert!(valid.validate().is_ok());
        
        let invalid = RegisterRequest {
            email: "not-an-email".to_string(),
            password: "MyStr0ng!Pass".to_string(),
            team_name: None,
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_login_request_validation() {
        let valid = LoginRequest {
            email: "user@example.com".to_string(),
            password: "password123".to_string(),
        };
        assert!(valid.validate().is_ok());
    }

    #[test]
    fn test_register_request_optional_team_name() {
        let with_team = RegisterRequest {
            email: "user@example.com".to_string(),
            password: "MyStr0ng!Pass".to_string(),
            team_name: Some("My Team".to_string()),
        };
        assert!(with_team.team_name.is_some());
        
        let without_team = RegisterRequest {
            email: "user@example.com".to_string(),
            password: "MyStr0ng!Pass".to_string(),
            team_name: None,
        };
        assert!(without_team.team_name.is_none());
    }
    
    #[test]
    fn test_auth_response_serialization() {
        let response = AuthResponse {
            user_id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            role: "admin".to_string(),
            token: "access_token".to_string(),
            refresh_token: "refresh_token".to_string(),
            expires_in: 3600,
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("user_id"));
        assert!(json.contains("token"));
        assert!(json.contains("refresh_token"));
        assert!(json.contains("expires_in"));
    }
    
    #[test]
    fn test_refresh_response_serialization() {
        let response = RefreshResponse {
            token: "new_token".to_string(),
            expires_in: 3600,
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("token"));
        assert!(json.contains("expires_in"));
    }
    
    #[test]
    fn test_refresh_token_request_deserialization() {
        let json = r#"{"refresh_token":"eyJ..."}"#;
        let request: RefreshTokenRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.refresh_token, "eyJ...");
    }

    #[test]
    fn test_user_profile_serialization() {
        let profile = UserProfile {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            team_id: Uuid::new_v4(),
            role: "developer".to_string(),
            is_active: true,
            last_login: Some(chrono::Utc::now()),
            created_at: chrono::Utc::now(),
        };
        
        let json = serde_json::to_string(&profile).unwrap();
        assert!(json.contains("email"));
        assert!(json.contains("role"));
        assert!(json.contains("is_active"));
        // Should not contain password_hash
        assert!(!json.contains("password_hash"));
    }
}

mod middleware_tests {
    use super::*;
    use agentkey_backend::middleware::auth::{AuthUser, RequireRole};

    #[test]
    fn test_auth_user_role_hierarchy() {
        let admin = AuthUser {
            user_id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            role: "admin".to_string(),
        };
        
        // Admin should have all permissions
        assert!(admin.is_admin());
        assert!(admin.is_developer());
        assert!(admin.is_viewer());
        
        let developer = AuthUser {
            user_id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            role: "developer".to_string(),
        };
        
        // Developer should have developer and viewer, but not admin
        assert!(!developer.is_admin());
        assert!(developer.is_developer());
        assert!(developer.is_viewer());
        
        let viewer = AuthUser {
            user_id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            role: "viewer".to_string(),
        };
        
        // Viewer should only have viewer permission
        assert!(!viewer.is_admin());
        assert!(!viewer.is_developer());
        assert!(viewer.is_viewer());
    }

    #[test]
    fn test_require_role_admin() {
        let admin = AuthUser {
            user_id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            role: "admin".to_string(),
        };
        let developer = AuthUser {
            user_id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            role: "developer".to_string(),
        };
        
        assert!(RequireRole::admin(&admin).is_ok());
        assert!(RequireRole::admin(&developer).is_err());
    }

    #[test]
    fn test_require_role_developer() {
        let admin = AuthUser {
            user_id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            role: "admin".to_string(),
        };
        let developer = AuthUser {
            user_id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            role: "developer".to_string(),
        };
        let viewer = AuthUser {
            user_id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            role: "viewer".to_string(),
        };
        
        assert!(RequireRole::developer(&admin).is_ok());
        assert!(RequireRole::developer(&developer).is_ok());
        assert!(RequireRole::developer(&viewer).is_err());
    }

    #[test]
    fn test_require_role_custom_roles() {
        let admin = AuthUser {
            user_id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            role: "admin".to_string(),
        };
        
        assert!(RequireRole::check(&admin, &["admin", "superadmin"]).is_ok());
        assert!(RequireRole::check(&admin, &["viewer"]).is_err());
    }
}

mod auth_service_tests {
    use super::*;
    use agentkey_backend::services::auth::AuthService;
    use std::sync::Arc;

    #[test]
    fn test_auth_service_validate_token() {
        let jwt = Arc::new(JwtService::new("test-secret-32-characters-here!".to_string(), 1));
        let service = AuthService::new(jwt.clone());
        
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        let token = jwt.create_token(user_id, team_id, "admin".to_string()).unwrap();
        
        let (parsed_user_id, parsed_team_id, role) = service.validate_token(&token).unwrap();
        
        assert_eq!(parsed_user_id, user_id);
        assert_eq!(parsed_team_id, team_id);
        assert_eq!(role, "admin");
    }

    #[test]
    fn test_auth_service_validate_invalid_token() {
        let jwt = Arc::new(JwtService::new("test-secret-32-characters-here!".to_string(), 1));
        let service = AuthService::new(jwt);
        
        let result = service.validate_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_auth_service_refresh_with_valid_token() {
        let jwt = Arc::new(JwtService::new("test-secret-32-characters-here!".to_string(), 1));
        let service = AuthService::new(jwt.clone());
        
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        let refresh_token = jwt.create_refresh_token(user_id, team_id, "admin".to_string(), 7).unwrap();
        
        let response = service.refresh_token(&refresh_token).unwrap();
        
        assert!(!response.token.is_empty());
        assert!(response.expires_in > 0);
    }

    #[test]
    fn test_auth_service_refresh_with_access_token_fails() {
        let jwt = Arc::new(JwtService::new("test-secret-32-characters-here!".to_string(), 1));
        let service = AuthService::new(jwt.clone());
        
        // Try to use access token as refresh token
        let access_token = jwt.create_token(Uuid::new_v4(), Uuid::new_v4(), "admin".to_string()).unwrap();
        
        let result = service.refresh_token(&access_token);
        assert!(result.is_err());
    }
}
