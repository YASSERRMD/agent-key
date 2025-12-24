//! Password hashing and validation service.
//!
//! Provides secure password hashing using bcrypt with 12 salt rounds,
//! and password strength validation.

use bcrypt::{hash, verify};
use thiserror::Error;

/// Bcrypt cost factor (12 rounds = 2^12 iterations)
const BCRYPT_COST: u32 = 12;

/// Minimum password length
const MIN_PASSWORD_LENGTH: usize = 12;

/// Password service errors
#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("Password too short: minimum {MIN_PASSWORD_LENGTH} characters required")]
    TooShort,

    #[error("Password must contain at least one uppercase letter")]
    MissingUppercase,

    #[error("Password must contain at least one lowercase letter")]
    MissingLowercase,

    #[error("Password must contain at least one digit")]
    MissingDigit,

    #[error("Password must contain at least one special character")]
    MissingSpecialChar,

    #[error("Password hashing failed: {0}")]
    HashingFailed(String),

    #[error("Password verification failed: {0}")]
    VerificationFailed(String),
}

/// Password hashing and validation service.
///
/// Uses bcrypt with 12 salt rounds for secure password hashing.
#[derive(Clone, Default)]
pub struct PasswordService;

impl PasswordService {
    /// Create a new password service instance.
    pub fn new() -> Self {
        PasswordService
    }

    /// Validate password strength requirements.
    ///
    /// Password must:
    /// - Be at least 12 characters long
    /// - Contain at least one uppercase letter
    /// - Contain at least one lowercase letter
    /// - Contain at least one digit
    /// - Contain at least one special character
    ///
    /// # Arguments
    ///
    /// * `password` - The password to validate
    ///
    /// # Returns
    ///
    /// `Ok(())` if password meets all requirements, `Err(PasswordError)` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use agentkey_backend::services::password::PasswordService;
    ///
    /// let service = PasswordService::new();
    /// assert!(service.validate_password("MyStr0ng!Pass").is_ok());
    /// assert!(service.validate_password("weak").is_err());
    /// ```
    pub fn validate_password(&self, password: &str) -> Result<(), PasswordError> {
        // Check minimum length
        if password.len() < MIN_PASSWORD_LENGTH {
            return Err(PasswordError::TooShort);
        }

        // Check for uppercase letter
        if !password.chars().any(|c| c.is_ascii_uppercase()) {
            return Err(PasswordError::MissingUppercase);
        }

        // Check for lowercase letter
        if !password.chars().any(|c| c.is_ascii_lowercase()) {
            return Err(PasswordError::MissingLowercase);
        }

        // Check for digit
        if !password.chars().any(|c| c.is_ascii_digit()) {
            return Err(PasswordError::MissingDigit);
        }

        // Check for special character
        let special_chars = "!@#$%^&*()_+-=[]{}|;':\",./<>?`~";
        if !password.chars().any(|c| special_chars.contains(c)) {
            return Err(PasswordError::MissingSpecialChar);
        }

        Ok(())
    }

    /// Hash a password using bcrypt with 12 salt rounds.
    ///
    /// # Arguments
    ///
    /// * `password` - The plaintext password to hash
    ///
    /// # Returns
    ///
    /// The bcrypt hash string on success.
    ///
    /// # Example
    ///
    /// ```
    /// use agentkey_backend::services::password::PasswordService;
    ///
    /// let service = PasswordService::new();
    /// let hash = service.hash("MyStr0ng!Pass").unwrap();
    /// assert!(hash.starts_with("$2b$"));
    /// ```
    pub fn hash(&self, password: &str) -> Result<String, PasswordError> {
        hash(password, BCRYPT_COST)
            .map_err(|e| PasswordError::HashingFailed(e.to_string()))
    }

    /// Verify a password against a bcrypt hash.
    ///
    /// # Arguments
    ///
    /// * `password` - The plaintext password to verify
    /// * `hash` - The bcrypt hash to compare against
    ///
    /// # Returns
    ///
    /// `Ok(true)` if password matches, `Ok(false)` if it doesn't.
    ///
    /// # Example
    ///
    /// ```
    /// use agentkey_backend::services::password::PasswordService;
    ///
    /// let service = PasswordService::new();
    /// let hash = service.hash("MyStr0ng!Pass").unwrap();
    /// assert!(service.verify("MyStr0ng!Pass", &hash).unwrap());
    /// assert!(!service.verify("WrongPassword!", &hash).unwrap());
    /// ```
    pub fn verify(&self, password: &str, hash: &str) -> Result<bool, PasswordError> {
        verify(password, hash)
            .map_err(|e| PasswordError::VerificationFailed(e.to_string()))
    }

    /// Hash a password after validating its strength.
    ///
    /// Combines validation and hashing in one operation.
    ///
    /// # Arguments
    ///
    /// * `password` - The plaintext password to validate and hash
    ///
    /// # Returns
    ///
    /// The bcrypt hash string if password is valid.
    pub fn hash_validated(&self, password: &str) -> Result<String, PasswordError> {
        self.validate_password(password)?;
        self.hash(password)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let service = PasswordService::new();
        let password = "MyStr0ng!Pass";

        let hash = service.hash(password).expect("Hashing should succeed");
        
        // Hash should start with bcrypt identifier
        assert!(hash.starts_with("$2b$"));
        
        // Verification should succeed with correct password
        assert!(service.verify(password, &hash).unwrap());
        
        // Verification should fail with wrong password
        assert!(!service.verify("WrongPassword!", &hash).unwrap());
    }

    #[test]
    fn test_different_hashes_same_password() {
        let service = PasswordService::new();
        let password = "MyStr0ng!Pass";

        let hash1 = service.hash(password).unwrap();
        let hash2 = service.hash(password).unwrap();

        // Different salts produce different hashes
        assert_ne!(hash1, hash2);

        // Both should verify correctly
        assert!(service.verify(password, &hash1).unwrap());
        assert!(service.verify(password, &hash2).unwrap());
    }

    #[test]
    fn test_password_too_short() {
        let service = PasswordService::new();
        
        let result = service.validate_password("Short1!");
        assert!(matches!(result, Err(PasswordError::TooShort)));
    }

    #[test]
    fn test_password_missing_uppercase() {
        let service = PasswordService::new();
        
        let result = service.validate_password("mystrongpass1!");
        assert!(matches!(result, Err(PasswordError::MissingUppercase)));
    }

    #[test]
    fn test_password_missing_lowercase() {
        let service = PasswordService::new();
        
        let result = service.validate_password("MYSTRONGPASS1!");
        assert!(matches!(result, Err(PasswordError::MissingLowercase)));
    }

    #[test]
    fn test_password_missing_digit() {
        let service = PasswordService::new();
        
        let result = service.validate_password("MyStrongPass!!");
        assert!(matches!(result, Err(PasswordError::MissingDigit)));
    }

    #[test]
    fn test_password_missing_special() {
        let service = PasswordService::new();
        
        let result = service.validate_password("MyStrongPass12");
        assert!(matches!(result, Err(PasswordError::MissingSpecialChar)));
    }

    #[test]
    fn test_valid_password() {
        let service = PasswordService::new();
        
        // All requirements met
        assert!(service.validate_password("MyStr0ng!Pass").is_ok());
        assert!(service.validate_password("C0mplex@Pass123").is_ok());
        assert!(service.validate_password("Sup3r$ecure#Key").is_ok());
    }

    #[test]
    fn test_hash_validated() {
        let service = PasswordService::new();
        
        // Valid password should hash
        let hash = service.hash_validated("MyStr0ng!Pass");
        assert!(hash.is_ok());
        
        // Invalid password should fail validation
        let result = service.hash_validated("weak");
        assert!(result.is_err());
    }

    #[test]
    fn test_unicode_password() {
        let service = PasswordService::new();
        let password = "MyStr0ng!日本語";

        let hash = service.hash(password).unwrap();
        assert!(service.verify(password, &hash).unwrap());
    }

    #[test]
    fn test_long_password() {
        let service = PasswordService::new();
        let password = "MyStr0ng!Pass".repeat(10);

        let hash = service.hash(&password).unwrap();
        assert!(service.verify(&password, &hash).unwrap());
    }
}
