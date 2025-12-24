//! Integration tests for JWT service.

use agentkey_backend::services::jwt::{JwtService, JwtError};
use uuid::Uuid;

const TEST_SECRET: &str = "jwt-test-secret-key-32-chars-min!";

#[test]
fn test_jwt_token_creation_and_verification() {
    let service = JwtService::new(TEST_SECRET.to_string(), 24);
    let user_id = Uuid::new_v4();
    let team_id = Uuid::new_v4();
    
    let token = service.create_token(user_id, team_id, "admin".to_string())
        .expect("Token creation should succeed");
    
    let claims = service.verify_token(&token)
        .expect("Token verification should succeed");
    
    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.team_id, team_id.to_string());
    assert_eq!(claims.role, "admin");
    assert_eq!(claims.iss, "agentkey");
}

#[test]
fn test_jwt_token_pair() {
    let service = JwtService::new(TEST_SECRET.to_string(), 24);
    let user_id = Uuid::new_v4();
    let team_id = Uuid::new_v4();
    
    let pair = service.create_token_pair(user_id, team_id, "developer".to_string())
        .expect("Token pair creation should succeed");
    
    assert_eq!(pair.token_type, "Bearer");
    assert_eq!(pair.expires_in, 24 * 3600);
    assert!(!pair.access_token.is_empty());
    
    // Verify the access token works
    let claims = service.verify_token(&pair.access_token).unwrap();
    assert_eq!(claims.role, "developer");
}

#[test]
fn test_jwt_expired_token() {
    let service = JwtService::new(TEST_SECRET.to_string(), 24);
    let user_id = Uuid::new_v4();
    let team_id = Uuid::new_v4();
    
    // Create a token that's already expired (-1 hour)
    let token = service.create_token_with_expiry(user_id, team_id, "admin".to_string(), -1)
        .expect("Token creation should succeed");
    
    let result = service.verify_token(&token);
    assert!(matches!(result, Err(JwtError::TokenExpired)));
}

#[test]
fn test_jwt_wrong_secret() {
    let service1 = JwtService::new(TEST_SECRET.to_string(), 24);
    let service2 = JwtService::new("different-secret-key-32-chars!!".to_string(), 24);
    
    let token = service1.create_token(Uuid::new_v4(), Uuid::new_v4(), "admin".to_string())
        .unwrap();
    
    let result = service2.verify_token(&token);
    assert!(result.is_err());
}

#[test]
fn test_jwt_invalid_token_format() {
    let service = JwtService::new(TEST_SECRET.to_string(), 24);
    
    // Completely invalid
    assert!(service.verify_token("invalid").is_err());
    
    // Wrong format
    assert!(service.verify_token("a.b.c").is_err());
    
    // Empty
    assert!(service.verify_token("").is_err());
}

#[test]
fn test_jwt_claims_helpers() {
    let service = JwtService::new(TEST_SECRET.to_string(), 24);
    let user_id = Uuid::new_v4();
    let team_id = Uuid::new_v4();
    
    let token = service.create_token(user_id, team_id, "admin".to_string()).unwrap();
    let claims = service.verify_token(&token).unwrap();
    
    // Test helper methods
    assert!(claims.is_admin());
    assert!(!claims.is_expired());
    assert_eq!(claims.user_id().unwrap(), user_id);
    assert_eq!(claims.get_team_id().unwrap(), team_id);
}

#[test]
fn test_jwt_different_roles() {
    let service = JwtService::new(TEST_SECRET.to_string(), 24);
    let user_id = Uuid::new_v4();
    let team_id = Uuid::new_v4();
    
    let roles = vec!["admin", "developer", "viewer"];
    
    for role in roles {
        let token = service.create_token(user_id, team_id, role.to_string()).unwrap();
        let claims = service.verify_token(&token).unwrap();
        assert_eq!(claims.role, role);
        
        if role == "admin" {
            assert!(claims.is_admin());
        } else {
            assert!(!claims.is_admin());
        }
    }
}

#[test]
fn test_jwt_custom_issuer() {
    let service = JwtService::with_issuer(
        TEST_SECRET.to_string(),
        24,
        "custom-issuer".to_string()
    );
    
    let token = service.create_token(Uuid::new_v4(), Uuid::new_v4(), "admin".to_string())
        .unwrap();
    
    let claims = service.verify_token(&token).unwrap();
    assert_eq!(claims.iss, "custom-issuer");
}

#[test]
fn test_jwt_decode_without_validation() {
    let service = JwtService::new(TEST_SECRET.to_string(), 24);
    let user_id = Uuid::new_v4();
    
    let token = service.create_token(user_id, Uuid::new_v4(), "admin".to_string()).unwrap();
    
    // This should work even without full validation
    let claims = service.decode_without_validation(&token).unwrap();
    assert_eq!(claims.sub, user_id.to_string());
}

#[test]
fn test_jwt_custom_expiry() {
    let service = JwtService::new(TEST_SECRET.to_string(), 24);
    
    // Create token with 48 hour expiry
    let token = service.create_token_with_expiry(
        Uuid::new_v4(),
        Uuid::new_v4(),
        "admin".to_string(),
        48
    ).unwrap();
    
    let claims = service.verify_token(&token).unwrap();
    
    // Check that expiry is roughly 48 hours from now
    let now = chrono::Utc::now().timestamp();
    let expected_exp = now + (48 * 3600);
    
    // Allow 5 seconds tolerance
    assert!((claims.exp - expected_exp).abs() < 5);
}
