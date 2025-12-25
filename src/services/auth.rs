//! Authentication service.
//!
//! Handles user registration, login, token refresh, and audit logging.

use sqlx::PgPool;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;
use validator::Validate;

use crate::errors::ApiError;
use crate::models::{
    log_audit_event, AuthResponse, LoginRequest, RefreshResponse, RegisterRequest, Team, User,
};
use crate::services::jwt::JwtService;
use crate::services::password::PasswordService;

/// Default refresh token expiry in days
const REFRESH_TOKEN_DAYS: i64 = 7;

/// Default access token expiry in hours  
const ACCESS_TOKEN_HOURS: i64 = 1;

/// Authentication service for user management and token operations.
pub struct AuthService {
    jwt_service: Arc<JwtService>,
    password_service: PasswordService,
}

impl AuthService {
    /// Create a new authentication service.
    pub fn new(jwt_service: Arc<JwtService>) -> Self {
        AuthService {
            jwt_service,
            password_service: PasswordService::new(),
        }
    }

    /// Register a new user and create their team.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `request` - Registration request with email, password, and optional team name
    ///
    /// # Returns
    ///
    /// `AuthResponse` with user details and tokens on success.
    pub async fn register(
        &self,
        pool: &PgPool,
        request: RegisterRequest,
    ) -> Result<AuthResponse, ApiError> {
        // Validate request
        request.validate().map_err(|e| {
            warn!("Registration validation failed: {}", e);
            ApiError::ValidationError(e.to_string())
        })?;

        // Validate password strength
        self.password_service
            .validate_password(&request.password)
            .map_err(|e| {
                warn!("Password validation failed: {}", e);
                ApiError::BadRequest(e.to_string())
            })?;

        // Start transaction
        let mut tx = pool.begin().await.map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        // Check if email already exists
        if User::find_by_email(pool, &request.email).await?.is_some() {
            warn!("Registration failed: email already exists");
            return Err(ApiError::Conflict("Email already registered".to_string()));
        }

        // Hash password
        let password_hash = self.password_service.hash(&request.password).map_err(|e| {
            warn!("Password hashing failed: {}", e);
            ApiError::InternalError(e.to_string())
        })?;

        // Create team first (with temporary owner_id that we'll update)
        let team_name = request
            .team_name
            .unwrap_or_else(|| format!("{}'s Team", request.email.split('@').next().unwrap_or("User")));

        let temp_owner_id = Uuid::new_v4();
        let team = Team::create(&mut *tx, &team_name, temp_owner_id, "free").await?;

        // Create user as admin of the new team
        let user = match User::create(&mut *tx, &request.email, &password_hash, team.id, "admin").await
        {
            Ok(user) => user,
            Err(e) => {
                // Clean up team if user creation failed
                warn!("User creation failed, cleaning up team: {}", e);
                return Err(e);
            }
        };

        // Update team owner to the new user
        Team::update_owner(&mut *tx, team.id, user.id).await?;

        // Generate tokens
        let access_token = self
            .jwt_service
            .create_token_with_expiry(user.id, team.id, user.role.clone(), ACCESS_TOKEN_HOURS)
            .map_err(|e| ApiError::InternalError(e.to_string()))?;

        let refresh_token = self
            .jwt_service
            .create_refresh_token(user.id, team.id, user.role.clone(), REFRESH_TOKEN_DAYS)
            .map_err(|e| ApiError::InternalError(e.to_string()))?;

        // Log registration event
        if let Err(e) = log_audit_event(
            &mut *tx,
            team.id,
            Some(user.id),
            "register",
            Some("user"),
            Some(user.id),
            Some("User registered"),
            None,
        )
        .await
        {
            warn!("Failed to log registration event: {}", e);
        }

        // Commit transaction
        tx.commit().await.map_err(|e| ApiError::DatabaseError(e.to_string()))?;


        info!("User registered successfully: {}", user.email);

        Ok(AuthResponse {
            user_id: user.id,
            team_id: team.id,
            email: user.email,
            role: user.role,
            token: access_token,
            refresh_token,
            expires_in: ACCESS_TOKEN_HOURS * 3600,
        })
    }

    /// Authenticate a user with email and password.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `request` - Login request with email and password
    ///
    /// # Returns
    ///
    /// `AuthResponse` with user details and tokens on success.
    pub async fn login(
        &self,
        pool: &PgPool,
        request: LoginRequest,
    ) -> Result<AuthResponse, ApiError> {
        // Validate request
        request.validate().map_err(|e| {
            warn!("Login validation failed: {}", e);
            ApiError::ValidationError(e.to_string())
        })?;

        // Find user by email
        let user = User::find_by_email(pool, &request.email)
            .await?
            .ok_or_else(|| {
                warn!("Login failed: user not found");
                // Don't reveal that user doesn't exist
                ApiError::Unauthorized("Invalid credentials".to_string())
            })?;

        // Check if user is active
        if !user.is_active {
            warn!("Login failed: user account disabled");
            return Err(ApiError::Unauthorized("Account is disabled".to_string()));
        }

        // Verify password
        let password_valid = self
            .password_service
            .verify(&request.password, &user.password_hash)
            .map_err(|e| {
                warn!("Password verification error: {}", e);
                ApiError::InternalError("Authentication failed".to_string())
            })?;

        if !password_valid {
            warn!("Login failed: invalid password for user {}", user.email);
            // Log failed login attempt
            if let Err(e) = log_audit_event(
                pool,
                user.team_id,
                Some(user.id),
                "login_failed",
                Some("user"),
                Some(user.id),
                Some("Invalid password"),
                None,
            )
            .await
            {
                warn!("Failed to log failed login event: {}", e);
            }
            return Err(ApiError::Unauthorized("Invalid credentials".to_string()));
        }

        // Update last login timestamp
        User::update_last_login(pool, user.id).await?;

        // Generate tokens
        let access_token = self
            .jwt_service
            .create_token_with_expiry(user.id, user.team_id, user.role.clone(), ACCESS_TOKEN_HOURS)
            .map_err(|e| ApiError::InternalError(e.to_string()))?;

        let refresh_token = self
            .jwt_service
            .create_refresh_token(user.id, user.team_id, user.role.clone(), REFRESH_TOKEN_DAYS)
            .map_err(|e| ApiError::InternalError(e.to_string()))?;

        // Log login event
        if let Err(e) = log_audit_event(
            pool,
            user.team_id,
            Some(user.id),
            "login",
            Some("user"),
            Some(user.id),
            Some("User logged in"),
            None,
        )
        .await
        {
            warn!("Failed to log login event: {}", e);
        }

        info!("User logged in successfully: {}", user.email);

        Ok(AuthResponse {
            user_id: user.id,
            team_id: user.team_id,
            email: user.email,
            role: user.role,
            token: access_token,
            refresh_token,
            expires_in: ACCESS_TOKEN_HOURS * 3600,
        })
    }

    /// Refresh an access token using a refresh token.
    ///
    /// # Arguments
    ///
    /// * `refresh_token` - Valid refresh token
    ///
    /// # Returns
    ///
    /// `RefreshResponse` with new access token on success.
    pub fn refresh_token(&self, refresh_token: &str) -> Result<RefreshResponse, ApiError> {
        // Verify refresh token
        let claims = self
            .jwt_service
            .verify_refresh_token(refresh_token)
            .map_err(|e| {
                warn!("Refresh token verification failed: {}", e);
                ApiError::Unauthorized("Invalid refresh token".to_string())
            })?;

        let user_id = claims.user_id().map_err(|e| {
            ApiError::InternalError(format!("Invalid user ID in token: {}", e))
        })?;

        let team_id = claims.get_team_id().map_err(|e| {
            ApiError::InternalError(format!("Invalid team ID in token: {}", e))
        })?;

        // Generate new access token
        let access_token = self
            .jwt_service
            .create_token_with_expiry(user_id, team_id, claims.role, ACCESS_TOKEN_HOURS)
            .map_err(|e| ApiError::InternalError(e.to_string()))?;

        info!("Token refreshed for user: {}", user_id);

        Ok(RefreshResponse {
            token: access_token,
            expires_in: ACCESS_TOKEN_HOURS * 3600,
        })
    }

    /// Validate an access token and return user information.
    ///
    /// # Arguments
    ///
    /// * `token` - Access token to validate
    ///
    /// # Returns
    ///
    /// Tuple of (user_id, team_id, role) on success.
    pub fn validate_token(&self, token: &str) -> Result<(Uuid, Uuid, String), ApiError> {
        let claims = self.jwt_service.verify_token(token).map_err(|e| {
            ApiError::Unauthorized(format!("Invalid token: {}", e))
        })?;

        let user_id = claims.user_id().map_err(|e| {
            ApiError::InternalError(format!("Invalid user ID in token: {}", e))
        })?;

        let team_id = claims.get_team_id().map_err(|e| {
            ApiError::InternalError(format!("Invalid team ID in token: {}", e))
        })?;

        Ok((user_id, team_id, claims.role))
    }

    /// Hash a password using the password service.
    pub fn hash_password(&self, password: &str) -> Result<String, ApiError> {
        self.password_service.hash(password).map_err(|e| {
            ApiError::InternalError(format!("Failed to hash password: {}", e))
        })
    }

    /// Verify a password against a hash.
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, ApiError> {
        self.password_service.verify(password, hash).map_err(|e| {
            ApiError::InternalError(format!("Failed to verify password: {}", e))
        })
    }

    /// Hash an API key for storage.
    pub fn hash_api_key(&self, api_key: &str) -> Result<String, ApiError> {
        // Use SHA256 for API keys (faster than bcrypt, still secure for random keys)
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(api_key.as_bytes());
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    /// Log an authentication event.
    pub async fn log_auth_event(
        pool: &PgPool,
        user_id: Uuid,
        team_id: Uuid,
        event_type: &str,
        description: &str,
    ) -> Result<(), ApiError> {
        log_audit_event(
            pool,
            team_id,
            Some(user_id),
            event_type,
            Some("auth"),
            None,
            Some(description),
            None,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = "test-jwt-secret-32-characters-here!";

    fn create_test_service() -> AuthService {
        let jwt = Arc::new(JwtService::new(TEST_SECRET.to_string(), 1));
        AuthService::new(jwt)
    }

    #[test]
    fn test_auth_service_creation() {
        let service = create_test_service();
        assert!(service.password_service.validate_password("MyStr0ng!Pass").is_ok());
    }

    #[test]
    fn test_validate_token() {
        let jwt = Arc::new(JwtService::new(TEST_SECRET.to_string(), 1));
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
    fn test_validate_invalid_token() {
        let service = create_test_service();
        let result = service.validate_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_refresh_token_with_access_token_fails() {
        let jwt = Arc::new(JwtService::new(TEST_SECRET.to_string(), 1));
        let service = AuthService::new(jwt.clone());
        
        // Create access token (not refresh token)
        let access_token = jwt.create_token(Uuid::new_v4(), Uuid::new_v4(), "admin".to_string()).unwrap();
        
        // Should fail because it's not a refresh token
        let result = service.refresh_token(&access_token);
        assert!(result.is_err());
    }
}
