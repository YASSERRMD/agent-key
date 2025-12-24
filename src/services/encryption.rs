//! AES-256-GCM Encryption Service.
//!
//! Provides secure encryption and decryption of sensitive data using
//! AES-256-GCM (Galois/Counter Mode) with random nonce generation.

use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use rand::{rngs::OsRng, RngCore};
use std::env;
use thiserror::Error;

/// Encryption Service Errors
#[derive(Debug, Error)]
pub enum EncryptionError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Invalid key length: expected 32 bytes (64 hex chars)")]
    InvalidKeyLength,

    #[error("Hex decode error: {0}")]
    HexDecodeError(#[from] hex::FromHexError),
}

/// Service for AES-256-GCM encryption
#[derive(Clone)]
pub struct EncryptionService {
    key: [u8; 32],
}

impl EncryptionService {
    /// Create a new encryption service with provided hex key
    pub fn new(key_hex: String) -> Result<Self, EncryptionError> {

        let key_bytes = hex::decode(&key_hex)
            .map_err(|e| EncryptionError::ConfigError(format!("Invalid hex key: {}", e)))?;

        if key_bytes.len() != 32 {
            return Err(EncryptionError::InvalidKeyLength);
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);

        Ok(Self { key })
    }

    /// Create from raw key bytes (testing mostly)
    pub fn from_key(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// Encrypt plaintext with AAD
    /// Returns [nonce (12) || ciphertext (variable) || tag (16)]
    pub fn encrypt(&self, plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let cipher = Aes256Gcm::new(&self.key.into());
        
        // Generate random 96-bit nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let payload = Payload {
            msg: plaintext,
            aad,
        };

        let ciphertext = cipher
            .encrypt(nonce, payload)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        // Format: Nonce + Ciphertext (Tag is included in ciphertext by aes-gcm)
        let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt ciphertext with AAD
    /// Expects [nonce (12) || ciphertext + tag]
    pub fn decrypt(&self, encrypted_data: &[u8], aad: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if encrypted_data.len() < 12 + 16 {
            return Err(EncryptionError::DecryptionFailed("Data too short".to_string()));
        }

        let cipher = Aes256Gcm::new(&self.key.into());

        // Extract nonce
        let nonce = Nonce::from_slice(&encrypted_data[0..12]);
        let ciphertext = &encrypted_data[12..];

        let payload = Payload {
            msg: ciphertext,
            aad,
        };

        let plaintext = cipher
            .decrypt(nonce, payload)
            .map_err(|_| EncryptionError::DecryptionFailed("Decryption failed (auth tag mismatch or bad key)".to_string()))?;

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock key for testing: 32 bytes of 0s
    const TEST_KEY: &str = "0000000000000000000000000000000000000000000000000000000000000000";

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        // setup_env() not needed anymore if we inject key

        let service = EncryptionService::new(TEST_KEY.to_string()).unwrap();
        let plaintext = b"Hello, World!";
        let aad = b"context";

        let encrypted = service.encrypt(plaintext, aad).unwrap();
        assert_ne!(encrypted, plaintext);

        let decrypted = service.decrypt(&encrypted, aad).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_unique_nonces() {
        let service = EncryptionService::new(TEST_KEY.to_string()).unwrap();
        let plaintext = b"same data";
        let aad = b"aad";

        let enc1 = service.encrypt(plaintext, aad).unwrap();
        let enc2 = service.encrypt(plaintext, aad).unwrap();

        // Should be different due to random nonce
        assert_ne!(enc1, enc2);
        // First 12 bytes (nonce) should be different
        assert_ne!(&enc1[0..12], &enc2[0..12]);
    }

    #[test]
    fn test_bad_aad_fails() {
        let service = EncryptionService::new(TEST_KEY.to_string()).unwrap();
        let plaintext = b"secret";
        let aad = b"correct";
        let bad_aad = b"wrong";

        let encrypted = service.encrypt(plaintext, aad).unwrap();
        
        let result = service.decrypt(&encrypted, bad_aad);
        assert!(matches!(result, Err(EncryptionError::DecryptionFailed(_))));
    }

    #[test]
    fn test_tampered_data_fails() {
        let service = EncryptionService::new(TEST_KEY.to_string()).unwrap();
        let plaintext = b"secret";
        let aad = b"aad";
        
        let mut encrypted = service.encrypt(plaintext, aad).unwrap();
        // Modification of ciphertext/tag
        let last = encrypted.len() - 1;
        encrypted[last] ^= 0xFF; // flip bits

        let result = service.decrypt(&encrypted, aad);
        assert!(matches!(result, Err(EncryptionError::DecryptionFailed(_))));
    }

    #[test]
    fn test_empty_plaintext() {
        let service = EncryptionService::new(TEST_KEY.to_string()).unwrap();
        let plaintext = b"";
        let aad = b"aad";

        let encrypted = service.encrypt(plaintext, aad).unwrap();
        let decrypted = service.decrypt(&encrypted, aad).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
