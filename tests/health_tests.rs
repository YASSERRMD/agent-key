//! Integration tests for health check endpoints.
//!
//! Note: These tests require Docker containers to be running.
//! Run `docker-compose up -d` before running these tests.

use agentkey_backend::config::Config;

#[test]
fn test_config_loads_from_env() {
    // Set up test environment
    std::env::set_var("DATABASE_URL", "postgresql://test:test@localhost/test");
    std::env::set_var("JWT_SECRET", "test-jwt-secret-32-characters-here!");
    std::env::set_var("ENCRYPTION_KEY", "test-encryption-32-characters-ok!");
    std::env::set_var("SERVER_PORT", "8080");
    std::env::set_var("ENVIRONMENT", "test");
    
    let config = Config::from_env().expect("Config should load");
    
    assert_eq!(config.server_port, 8080);
    assert_eq!(config.environment, "test");
    assert!(config.is_development() == false); // environment is "test" not "development"
    assert!(config.is_production() == false);
    
    // Cleanup
    std::env::remove_var("DATABASE_URL");
    std::env::remove_var("JWT_SECRET");
    std::env::remove_var("ENCRYPTION_KEY");
    std::env::remove_var("SERVER_PORT");
    std::env::remove_var("ENVIRONMENT");
}

#[test]
fn test_config_validates_jwt_secret_length() {
    std::env::set_var("DATABASE_URL", "postgresql://test:test@localhost/test");
    std::env::set_var("JWT_SECRET", "short"); // Too short
    std::env::set_var("ENCRYPTION_KEY", "test-encryption-32-characters-ok!");
    
    let result = Config::from_env();
    assert!(result.is_err());
    
    // Cleanup
    std::env::remove_var("DATABASE_URL");
    std::env::remove_var("JWT_SECRET");
    std::env::remove_var("ENCRYPTION_KEY");
}

#[test]
fn test_config_validates_encryption_key_length() {
    std::env::set_var("DATABASE_URL", "postgresql://test:test@localhost/test");
    std::env::set_var("JWT_SECRET", "test-jwt-secret-32-characters-here!");
    std::env::set_var("ENCRYPTION_KEY", "short"); // Too short
    
    let result = Config::from_env();
    assert!(result.is_err());
    
    // Cleanup
    std::env::remove_var("DATABASE_URL");
    std::env::remove_var("JWT_SECRET");
    std::env::remove_var("ENCRYPTION_KEY");
}

#[test]
fn test_config_default_values() {
    std::env::set_var("DATABASE_URL", "postgresql://test:test@localhost/test");
    std::env::set_var("JWT_SECRET", "test-jwt-secret-32-characters-here!");
    std::env::set_var("ENCRYPTION_KEY", "test-encryption-32-characters-ok!");
    
    // Remove optional vars to test defaults
    std::env::remove_var("SERVER_PORT");
    std::env::remove_var("SERVER_HOST");
    std::env::remove_var("ENVIRONMENT");
    std::env::remove_var("REDIS_URL");
    
    let config = Config::from_env().expect("Config should load with defaults");
    
    assert_eq!(config.server_port, 8080);
    assert_eq!(config.server_host, "127.0.0.1");
    assert_eq!(config.environment, "development");
    assert_eq!(config.redis_url, "redis://localhost:6379");
    
    // Cleanup
    std::env::remove_var("DATABASE_URL");
    std::env::remove_var("JWT_SECRET");
    std::env::remove_var("ENCRYPTION_KEY");
}
