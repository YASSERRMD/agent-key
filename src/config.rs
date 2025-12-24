//! Configuration module for AgentKey backend.
//!
//! Handles loading and validation of environment-based configuration.

use std::env;
use thiserror::Error;

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Invalid configuration value for {key}: {message}")]
    InvalidValue { key: String, message: String },
}

/// Application configuration loaded from environment variables.
#[derive(Clone, Debug)]
pub struct Config {
    /// PostgreSQL database connection URL
    pub database_url: String,

    /// Redis connection URL
    pub redis_url: String,

    /// Secret key for JWT token signing (minimum 32 characters)
    pub jwt_secret: String,

    /// JWT token expiry in hours
    pub jwt_expiry_hours: i64,

    /// Server bind host
    pub server_host: String,

    /// Server bind port
    pub server_port: u16,

    /// Environment name (development, staging, production)
    pub environment: String,

    /// Log level (trace, debug, info, warn, error)
    pub log_level: String,

    /// Encryption key for AES-256-GCM (minimum 32 characters)
    pub encryption_key: String,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if required variables are missing or invalid.
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL".to_string()))?;

        let redis_url = env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| ConfigError::MissingEnvVar("JWT_SECRET".to_string()))?;

        // Validate JWT secret length
        if jwt_secret.len() < 32 {
            return Err(ConfigError::InvalidValue {
                key: "JWT_SECRET".to_string(),
                message: "Must be at least 32 characters".to_string(),
            });
        }

        let jwt_expiry_hours: i64 = env::var("JWT_EXPIRY_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue {
                key: "JWT_EXPIRY_HOURS".to_string(),
                message: "Must be a valid integer".to_string(),
            })?;

        let server_host = env::var("SERVER_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let server_port: u16 = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue {
                key: "SERVER_PORT".to_string(),
                message: "Must be a valid port number".to_string(),
            })?;

        let environment = env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string());

        let log_level = env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_string());

        let encryption_key = env::var("AGENTKEY_MASTER_KEY")
            .map_err(|_| ConfigError::MissingEnvVar("AGENTKEY_MASTER_KEY".to_string()))?;

        // Validate encryption key length
        if encryption_key.len() < 32 {
            return Err(ConfigError::InvalidValue {
                key: "AGENTKEY_MASTER_KEY".to_string(),
                message: "Must be at least 32 characters".to_string(),
            });
        }

        Ok(Config {
            database_url,
            redis_url,
            jwt_secret,
            jwt_expiry_hours,
            server_host,
            server_port,
            environment,
            log_level,
            encryption_key,
        })
    }

    /// Check if running in production environment.
    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }

    /// Check if running in development environment.
    pub fn is_development(&self) -> bool {
        self.environment == "development"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn setup_test_env() {
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost/test");
        env::set_var("JWT_SECRET", "test-secret-key-must-be-32-chars-long!");
        env::set_var("ENCRYPTION_KEY", "test-encryption-key-32-chars-min!");
    }

    fn cleanup_test_env() {
        env::remove_var("DATABASE_URL");
        env::remove_var("JWT_SECRET");
        env::remove_var("ENCRYPTION_KEY");
        env::remove_var("SERVER_PORT");
        env::remove_var("JWT_EXPIRY_HOURS");
    }

    #[test]
    fn test_config_from_env_success() {
        setup_test_env();
        let config = Config::from_env();
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.server_port, 8080);
        assert!(config.is_development());
        cleanup_test_env();
    }

    #[test]
    fn test_config_missing_database_url() {
        cleanup_test_env();
        env::set_var("JWT_SECRET", "test-secret-key-must-be-32-chars-long!");
        env::set_var("ENCRYPTION_KEY", "test-encryption-key-32-chars-min!");
        
        let config = Config::from_env();
        assert!(config.is_err());
        cleanup_test_env();
    }

    #[test]
    fn test_config_short_jwt_secret() {
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost/test");
        env::set_var("JWT_SECRET", "short");
        env::set_var("ENCRYPTION_KEY", "test-encryption-key-32-chars-min!");
        
        let config = Config::from_env();
        assert!(config.is_err());
        cleanup_test_env();
    }
}
