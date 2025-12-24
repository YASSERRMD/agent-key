//! JWT (JSON Web Token) Service.
//!
//! Provides secure token generation and validation for authentication.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// JWT service errors
#[derive(Debug, Error)]
pub enum JwtError {
    #[error("Token creation failed: {0}")]
    CreationFailed(String),

    #[error("Token validation failed: {0}")]
    ValidationFailed(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token: {0}")]
    InvalidToken(String),
}

impl From<jsonwebtoken::errors::Error> for JwtError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        use jsonwebtoken::errors::ErrorKind;
        match err.kind() {
            ErrorKind::ExpiredSignature => JwtError::TokenExpired,
            ErrorKind::InvalidToken => JwtError::InvalidToken("Malformed token".to_string()),
            ErrorKind::InvalidSignature => JwtError::InvalidToken("Invalid signature".to_string()),
            _ => JwtError::ValidationFailed(err.to_string()),
        }
    }
}

/// JWT claims embedded in the token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,

    /// Team ID
    pub team_id: String,

    /// User role (admin, developer, viewer)
    pub role: String,

    /// Expiration time (Unix timestamp)
    pub exp: i64,

    /// Issued at time (Unix timestamp)
    pub iat: i64,

    /// Not valid before (Unix timestamp)
    pub nbf: i64,

    /// Token issuer
    pub iss: String,
}

impl Claims {
    /// Get the user ID as a UUID.
    pub fn user_id(&self) -> Result<Uuid, uuid::Error> {
        Uuid::parse_str(&self.sub)
    }

    /// Get the team ID as a UUID.
    pub fn get_team_id(&self) -> Result<Uuid, uuid::Error> {
        Uuid::parse_str(&self.team_id)
    }

    /// Check if the token is expired.
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    /// Check if the user has admin role.
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }
}

/// Token type for response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    /// Access token (short-lived)
    pub access_token: String,

    /// Token type (always "Bearer")
    pub token_type: String,

    /// Expiration time in seconds
    pub expires_in: i64,
}

/// JWT service for token generation and validation.
#[derive(Clone)]
pub struct JwtService {
    secret: String,
    issuer: String,
    expiry_hours: i64,
}

impl JwtService {
    /// Create a new JWT service.
    ///
    /// # Arguments
    ///
    /// * `secret` - Secret key for signing tokens (minimum 32 characters)
    /// * `expiry_hours` - Token expiration time in hours
    ///
    /// # Example
    ///
    /// ```
    /// use agentkey_backend::services::jwt::JwtService;
    ///
    /// let service = JwtService::new(
    ///     "my-super-secret-key-32-chars-min!".to_string(),
    ///     24
    /// );
    /// ```
    pub fn new(secret: String, expiry_hours: i64) -> Self {
        JwtService {
            secret,
            issuer: "agentkey".to_string(),
            expiry_hours,
        }
    }

    /// Create a new JWT service with custom issuer.
    pub fn with_issuer(secret: String, expiry_hours: i64, issuer: String) -> Self {
        JwtService {
            secret,
            issuer,
            expiry_hours,
        }
    }

    /// Create an access token for a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - User's unique identifier
    /// * `team_id` - Team's unique identifier
    /// * `role` - User's role (admin, developer, viewer)
    ///
    /// # Returns
    ///
    /// JWT token string.
    ///
    /// # Example
    ///
    /// ```
    /// use agentkey_backend::services::jwt::JwtService;
    /// use uuid::Uuid;
    ///
    /// let service = JwtService::new("secret-key-must-be-32-characters!".to_string(), 24);
    /// let token = service.create_token(
    ///     Uuid::new_v4(),
    ///     Uuid::new_v4(),
    ///     "admin".to_string()
    /// ).unwrap();
    /// ```
    pub fn create_token(
        &self,
        user_id: Uuid,
        team_id: Uuid,
        role: String,
    ) -> Result<String, JwtError> {
        self.create_token_with_expiry(user_id, team_id, role, self.expiry_hours)
    }

    /// Create a token with custom expiry time.
    ///
    /// # Arguments
    ///
    /// * `user_id` - User's unique identifier
    /// * `team_id` - Team's unique identifier
    /// * `role` - User's role
    /// * `expiry_hours` - Custom expiry time in hours
    pub fn create_token_with_expiry(
        &self,
        user_id: Uuid,
        team_id: Uuid,
        role: String,
        expiry_hours: i64,
    ) -> Result<String, JwtError> {
        let now = Utc::now();
        let expiration = now + Duration::hours(expiry_hours);

        let claims = Claims {
            sub: user_id.to_string(),
            team_id: team_id.to_string(),
            role,
            exp: expiration.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            iss: self.issuer.clone(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| JwtError::CreationFailed(e.to_string()))
    }

    /// Create a token pair (access token with metadata).
    pub fn create_token_pair(
        &self,
        user_id: Uuid,
        team_id: Uuid,
        role: String,
    ) -> Result<TokenPair, JwtError> {
        let access_token = self.create_token(user_id, team_id, role)?;

        Ok(TokenPair {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: self.expiry_hours * 3600,
        })
    }

    /// Verify and decode a JWT token.
    ///
    /// # Arguments
    ///
    /// * `token` - JWT token string
    ///
    /// # Returns
    ///
    /// Decoded claims if the token is valid.
    ///
    /// # Errors
    ///
    /// Returns `JwtError` if the token is invalid, expired, or tampered.
    pub fn verify_token(&self, token: &str) -> Result<Claims, JwtError> {
        let mut validation = Validation::default();
        validation.set_issuer(&[&self.issuer]);

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )?;

        Ok(token_data.claims)
    }

    /// Extract claims from a token without full validation.
    ///
    /// This is useful for debugging or inspection, but should NOT be used
    /// for authentication decisions.
    ///
    /// # Warning
    ///
    /// This method does not verify the signature. Always use `verify_token`
    /// for authentication.
    pub fn decode_without_validation(&self, token: &str) -> Result<Claims, JwtError> {
        let mut validation = Validation::default();
        validation.insecure_disable_signature_validation();
        validation.validate_exp = false;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )?;

        Ok(token_data.claims)
    }

    /// Create a refresh token for a user.
    ///
    /// Refresh tokens have a longer expiry (default 7 days) and are used
    /// to obtain new access tokens without re-authentication.
    ///
    /// # Arguments
    ///
    /// * `user_id` - User's unique identifier
    /// * `team_id` - Team's unique identifier
    /// * `role` - User's role
    /// * `days` - Token expiry in days (default 7)
    pub fn create_refresh_token(
        &self,
        user_id: Uuid,
        team_id: Uuid,
        role: String,
        days: i64,
    ) -> Result<String, JwtError> {
        let now = Utc::now();
        let expiration = now + Duration::days(days);

        let claims = RefreshClaims {
            sub: user_id.to_string(),
            team_id: team_id.to_string(),
            role,
            exp: expiration.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            iss: self.issuer.clone(),
            token_type: "refresh".to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| JwtError::CreationFailed(e.to_string()))
    }

    /// Verify and decode a refresh token.
    ///
    /// Ensures the token is valid and has token_type == "refresh".
    pub fn verify_refresh_token(&self, token: &str) -> Result<RefreshClaims, JwtError> {
        let mut validation = Validation::default();
        validation.set_issuer(&[&self.issuer]);

        let token_data = decode::<RefreshClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )?;

        // Ensure it's a refresh token
        if token_data.claims.token_type != "refresh" {
            return Err(JwtError::InvalidToken("Not a refresh token".to_string()));
        }

        Ok(token_data.claims)
    }

    /// Get the expiration timestamp from a token.
    pub fn get_expiration_unix(&self, token: &str) -> Result<i64, JwtError> {
        let claims = self.decode_without_validation(token)?;
        Ok(claims.exp)
    }

    /// Check if a token is expiring soon (within 5 minutes).
    ///
    /// Useful for clients to know when to refresh their token.
    pub fn is_token_expiring_soon(&self, token: &str) -> Result<bool, JwtError> {
        let claims = self.decode_without_validation(token)?;
        let five_minutes_from_now = Utc::now().timestamp() + 300;
        Ok(claims.exp < five_minutes_from_now)
    }
}

/// Refresh token claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshClaims {
    /// Subject (user ID)
    pub sub: String,

    /// Team ID
    pub team_id: String,

    /// User role
    pub role: String,

    /// Expiration time (Unix timestamp)
    pub exp: i64,

    /// Issued at time (Unix timestamp)
    pub iat: i64,

    /// Not valid before (Unix timestamp)
    pub nbf: i64,

    /// Token issuer
    pub iss: String,

    /// Token type (always "refresh" for refresh tokens)
    pub token_type: String,
}

impl RefreshClaims {
    /// Get the user ID as a UUID.
    pub fn user_id(&self) -> Result<Uuid, uuid::Error> {
        Uuid::parse_str(&self.sub)
    }

    /// Get the team ID as a UUID.
    pub fn get_team_id(&self) -> Result<Uuid, uuid::Error> {
        Uuid::parse_str(&self.team_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = "test-secret-key-must-be-32-chars!";

    fn create_test_service() -> JwtService {
        JwtService::new(TEST_SECRET.to_string(), 24)
    }

    #[test]
    fn test_create_and_verify_token() {
        let service = create_test_service();
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();

        let token = service
            .create_token(user_id, team_id, "admin".to_string())
            .expect("Token creation should succeed");

        let claims = service
            .verify_token(&token)
            .expect("Token verification should succeed");

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.team_id, team_id.to_string());
        assert_eq!(claims.role, "admin");
        assert_eq!(claims.iss, "agentkey");
    }

    #[test]
    fn test_token_pair_creation() {
        let service = create_test_service();
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();

        let pair = service
            .create_token_pair(user_id, team_id, "developer".to_string())
            .expect("Token pair creation should succeed");

        assert_eq!(pair.token_type, "Bearer");
        assert_eq!(pair.expires_in, 24 * 3600);
        assert!(!pair.access_token.is_empty());
    }

    #[test]
    fn test_invalid_token() {
        let service = create_test_service();
        let result = service.verify_token("invalid.token.here");

        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_secret_fails() {
        let service1 = JwtService::new(TEST_SECRET.to_string(), 24);
        let service2 = JwtService::new("different-secret-key-32-chars!!".to_string(), 24);

        let token = service1
            .create_token(Uuid::new_v4(), Uuid::new_v4(), "admin".to_string())
            .unwrap();

        let result = service2.verify_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_expired_token() {
        let service = create_test_service();
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();

        // Create a token that's already expired
        let token = service
            .create_token_with_expiry(user_id, team_id, "admin".to_string(), -1)
            .unwrap();

        let result = service.verify_token(&token);
        assert!(matches!(result, Err(JwtError::TokenExpired)));
    }

    #[test]
    fn test_claims_helper_methods() {
        let claims = Claims {
            sub: Uuid::new_v4().to_string(),
            team_id: Uuid::new_v4().to_string(),
            role: "admin".to_string(),
            exp: (Utc::now() + Duration::hours(1)).timestamp(),
            iat: Utc::now().timestamp(),
            nbf: Utc::now().timestamp(),
            iss: "agentkey".to_string(),
        };

        assert!(claims.is_admin());
        assert!(!claims.is_expired());
        assert!(claims.user_id().is_ok());
        assert!(claims.get_team_id().is_ok());
    }

    #[test]
    fn test_custom_issuer() {
        let service = JwtService::with_issuer(
            TEST_SECRET.to_string(),
            24,
            "custom-issuer".to_string(),
        );

        let token = service
            .create_token(Uuid::new_v4(), Uuid::new_v4(), "admin".to_string())
            .unwrap();

        let claims = service.verify_token(&token).unwrap();
        assert_eq!(claims.iss, "custom-issuer");
    }

    #[test]
    fn test_decode_without_validation() {
        let service = create_test_service();
        let user_id = Uuid::new_v4();

        let token = service
            .create_token(user_id, Uuid::new_v4(), "admin".to_string())
            .unwrap();

        let claims = service.decode_without_validation(&token).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
    }

    #[test]
    fn test_create_and_verify_refresh_token() {
        let service = create_test_service();
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();

        let refresh_token = service
            .create_refresh_token(user_id, team_id, "admin".to_string(), 7)
            .expect("Refresh token creation should succeed");

        let claims = service
            .verify_refresh_token(&refresh_token)
            .expect("Refresh token verification should succeed");

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.team_id, team_id.to_string());
        assert_eq!(claims.token_type, "refresh");
    }

    #[test]
    fn test_refresh_token_different_from_access_token() {
        let service = create_test_service();
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();

        let access_token = service
            .create_token(user_id, team_id, "admin".to_string())
            .unwrap();
        let refresh_token = service
            .create_refresh_token(user_id, team_id, "admin".to_string(), 7)
            .unwrap();

        assert_ne!(access_token, refresh_token);
    }

    #[test]
    fn test_access_token_fails_refresh_validation() {
        let service = create_test_service();
        let access_token = service
            .create_token(Uuid::new_v4(), Uuid::new_v4(), "admin".to_string())
            .unwrap();

        // Access token should fail refresh token validation (missing token_type)
        let result = service.verify_refresh_token(&access_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_expiration_unix() {
        let service = create_test_service();
        let token = service
            .create_token(Uuid::new_v4(), Uuid::new_v4(), "admin".to_string())
            .unwrap();

        let exp = service.get_expiration_unix(&token).unwrap();
        let now = Utc::now().timestamp();
        
        // Expiration should be ~24 hours from now
        assert!(exp > now);
        assert!(exp < now + 24 * 3600 + 60); // within 24h + 1min buffer
    }

    #[test]
    fn test_is_token_expiring_soon() {
        let service = create_test_service();
        
        // Token with 24 hours expiry should not be expiring soon
        let long_token = service
            .create_token(Uuid::new_v4(), Uuid::new_v4(), "admin".to_string())
            .unwrap();
        assert!(!service.is_token_expiring_soon(&long_token).unwrap());

        // Token with 1 minute expiry should be expiring soon
        let short_service = JwtService::new(TEST_SECRET.to_string(), 0);
        let short_token = short_service
            .create_token_with_expiry(Uuid::new_v4(), Uuid::new_v4(), "admin".to_string(), 0)
            .unwrap();
        assert!(short_service.is_token_expiring_soon(&short_token).unwrap());
    }
}

