//! AES-256-GCM Encryption Service.
//!
//! Provides secure encryption and decryption of sensitive data using
//! AES-256-GCM (Galois/Counter Mode) with random nonce generation.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use thiserror::Error;

/// Nonce size in bytes (96 bits as recommended for GCM)
const NONCE_SIZE: usize = 12;

/// Key size in bytes (256 bits for AES-256)
const KEY_SIZE: usize = 32;

/// Encryption service errors
#[derive(Debug, Error)]
pub enum EncryptionError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid key length: expected {expected}, got {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },

    #[error("Invalid ciphertext format")]
    InvalidCiphertext,

    #[error("Hex decode error: {0}")]
    HexDecodeError(#[from] hex::FromHexError),
}

/// AES-256-GCM encryption service.
///
/// Provides secure encryption with:
/// - Random nonce generation for each encryption
/// - Hex-encoded output (nonce + ciphertext)
/// - Authentication tag for integrity verification
#[derive(Clone)]
pub struct EncryptionService {
    key: [u8; KEY_SIZE],
}

impl EncryptionService {
    /// Create a new encryption service with the given secret key.
    ///
    /// The key will be derived from the secret string. For best security,
    /// provide a secret of at least 32 characters.
    ///
    /// # Arguments
    ///
    /// * `secret` - Secret key string (minimum 32 characters recommended)
    ///
    /// # Example
    ///
    /// ```
    /// use agentkey_backend::services::encryption::EncryptionService;
    ///
    /// let service = EncryptionService::new("my-super-secret-key-32-chars-min!");
    /// ```
    pub fn new(secret: &str) -> Self {
        let mut key = [0u8; KEY_SIZE];
        let secret_bytes = secret.as_bytes();
        let copy_len = secret_bytes.len().min(KEY_SIZE);
        key[..copy_len].copy_from_slice(&secret_bytes[..copy_len]);

        EncryptionService { key }
    }

    /// Create an encryption service with a raw 32-byte key.
    ///
    /// # Arguments
    ///
    /// * `key` - 32-byte encryption key
    ///
    /// # Errors
    ///
    /// Returns `EncryptionError::InvalidKeyLength` if key is not 32 bytes.
    pub fn from_key(key: &[u8]) -> Result<Self, EncryptionError> {
        if key.len() != KEY_SIZE {
            return Err(EncryptionError::InvalidKeyLength {
                expected: KEY_SIZE,
                actual: key.len(),
            });
        }

        let mut key_array = [0u8; KEY_SIZE];
        key_array.copy_from_slice(key);

        Ok(EncryptionService { key: key_array })
    }

    /// Encrypt plaintext and return hex-encoded ciphertext.
    ///
    /// The output format is: `hex(nonce || ciphertext || auth_tag)`
    ///
    /// # Arguments
    ///
    /// * `plaintext` - Data to encrypt
    ///
    /// # Returns
    ///
    /// Hex-encoded string containing nonce and ciphertext.
    ///
    /// # Example
    ///
    /// ```
    /// use agentkey_backend::services::encryption::EncryptionService;
    ///
    /// let service = EncryptionService::new("my-super-secret-key-32-chars-min!");
    /// let encrypted = service.encrypt("secret data").unwrap();
    /// ```
    pub fn encrypt(&self, plaintext: &str) -> Result<String, EncryptionError> {
        let cipher = Aes256Gcm::new((&self.key).into());

        // Generate random nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        // Combine nonce + ciphertext and hex encode
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(hex::encode(result))
    }

    /// Decrypt hex-encoded ciphertext and return plaintext.
    ///
    /// # Arguments
    ///
    /// * `encrypted` - Hex-encoded string from `encrypt()`
    ///
    /// # Returns
    ///
    /// Decrypted plaintext string.
    ///
    /// # Example
    ///
    /// ```
    /// use agentkey_backend::services::encryption::EncryptionService;
    ///
    /// let service = EncryptionService::new("my-super-secret-key-32-chars-min!");
    /// let encrypted = service.encrypt("secret data").unwrap();
    /// let decrypted = service.decrypt(&encrypted).unwrap();
    /// assert_eq!(decrypted, "secret data");
    /// ```
    pub fn decrypt(&self, encrypted: &str) -> Result<String, EncryptionError> {
        let cipher = Aes256Gcm::new((&self.key).into());

        // Decode from hex
        let decoded = hex::decode(encrypted)?;

        // Validate minimum length (nonce + auth tag of 16 bytes)
        if decoded.len() < NONCE_SIZE + 16 {
            return Err(EncryptionError::InvalidCiphertext);
        }

        // Split nonce and ciphertext
        let (nonce_bytes, ciphertext) = decoded.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decrypt
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;

        String::from_utf8(plaintext)
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))
    }

    /// Encrypt bytes and return hex-encoded ciphertext.
    ///
    /// # Arguments
    ///
    /// * `data` - Bytes to encrypt
    ///
    /// # Returns
    ///
    /// Hex-encoded string containing nonce and ciphertext.
    pub fn encrypt_bytes(&self, data: &[u8]) -> Result<String, EncryptionError> {
        let cipher = Aes256Gcm::new((&self.key).into());

        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(hex::encode(result))
    }

    /// Decrypt hex-encoded ciphertext and return raw bytes.
    ///
    /// # Arguments
    ///
    /// * `encrypted` - Hex-encoded string from `encrypt_bytes()`
    ///
    /// # Returns
    ///
    /// Decrypted bytes.
    pub fn decrypt_bytes(&self, encrypted: &str) -> Result<Vec<u8>, EncryptionError> {
        let cipher = Aes256Gcm::new((&self.key).into());

        let decoded = hex::decode(encrypted)?;

        if decoded.len() < NONCE_SIZE + 16 {
            return Err(EncryptionError::InvalidCiphertext);
        }

        let (nonce_bytes, ciphertext) = decoded.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = "test-secret-key-must-be-32-chars!";

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let service = EncryptionService::new(TEST_SECRET);
        let plaintext = "This is a secret message!";

        let encrypted = service.encrypt(plaintext).expect("Encryption should succeed");
        let decrypted = service.decrypt(&encrypted).expect("Decryption should succeed");

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_different_encryptions_produce_different_output() {
        let service = EncryptionService::new(TEST_SECRET);
        let plaintext = "Same message";

        let encrypted1 = service.encrypt(plaintext).unwrap();
        let encrypted2 = service.encrypt(plaintext).unwrap();

        // Different nonces produce different ciphertexts
        assert_ne!(encrypted1, encrypted2);

        // Both decrypt to the same plaintext
        assert_eq!(plaintext, service.decrypt(&encrypted1).unwrap());
        assert_eq!(plaintext, service.decrypt(&encrypted2).unwrap());
    }

    #[test]
    fn test_empty_string_encryption() {
        let service = EncryptionService::new(TEST_SECRET);
        let plaintext = "";

        let encrypted = service.encrypt(plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_unicode_encryption() {
        let service = EncryptionService::new(TEST_SECRET);
        let plaintext = "Hello, 世界! 🔐";

        let encrypted = service.encrypt(plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_long_plaintext() {
        let service = EncryptionService::new(TEST_SECRET);
        let plaintext = "A".repeat(10000);

        let encrypted = service.encrypt(&plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_invalid_ciphertext() {
        let service = EncryptionService::new(TEST_SECRET);
        let result = service.decrypt("invalid");

        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_ciphertext() {
        let service = EncryptionService::new(TEST_SECRET);
        let encrypted = service.encrypt("secret").unwrap();

        // Tamper with the ciphertext
        let mut tampered = hex::decode(&encrypted).unwrap();
        if let Some(byte) = tampered.last_mut() {
            *byte ^= 0xFF;
        }
        let tampered_hex = hex::encode(tampered);

        let result = service.decrypt(&tampered_hex);
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_key_fails() {
        let service1 = EncryptionService::new(TEST_SECRET);
        let service2 = EncryptionService::new("different-secret-key-32-chars-!");

        let encrypted = service1.encrypt("secret").unwrap();
        let result = service2.decrypt(&encrypted);

        assert!(result.is_err());
    }

    #[test]
    fn test_from_key_valid() {
        let key = [0u8; 32];
        let result = EncryptionService::from_key(&key);
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_key_invalid_length() {
        let key = [0u8; 16]; // Too short
        let result = EncryptionService::from_key(&key);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_decrypt_bytes() {
        let service = EncryptionService::new(TEST_SECRET);
        let data = b"binary data \x00\x01\x02\x03";

        let encrypted = service.encrypt_bytes(data).unwrap();
        let decrypted = service.decrypt_bytes(&encrypted).unwrap();

        assert_eq!(data.to_vec(), decrypted);
    }
}
