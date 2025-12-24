//! API Key generation and hashing utilities.
//!
//! Provides secure random key generation and SHA-256 hashing.

use rand::{rngs::OsRng, Rng};
use sha2::{Digest, Sha256};

/// Generator for secure API keys.
pub struct ApiKeyGenerator;

impl ApiKeyGenerator {
    /// Generate a 64-character secure API key.
    ///
    /// Format: "ak_" + 61 random alphanumeric characters.
    pub fn generate() -> String {
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        const KEY_LEN: usize = 61; // 64 total - 3 for "ak_" prefix

        let mut rng = OsRng;
        let mut key = String::with_capacity(64);
        key.push_str("ak_");

        for _ in 0..KEY_LEN {
            let idx = rng.gen_range(0..CHARSET.len());
            key.push(CHARSET[idx] as char);
        }

        key
    }

    /// Hash an API key using SHA-256.
    ///
    /// Returns the hex-encoded hash.
    pub fn hash(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Validate the format of an API key.
    ///
    /// Checks:
    /// - Starts with "ak_"
    /// - Length is exactly 64 characters
    /// - Contains only alphanumeric characters (after prefix)
    pub fn validate_format(key: &str) -> bool {
        if key.len() != 64 {
            return false;
        }

        if !key.starts_with("ak_") {
            return false;
        }

        key[3..].chars().all(|c| c.is_ascii_alphanumeric())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_format() {
        let key = ApiKeyGenerator::generate();
        assert!(key.starts_with("ak_"));
        assert_eq!(key.len(), 64);
        assert!(ApiKeyGenerator::validate_format(&key));
    }

    #[test]
    fn test_generate_unique_keys() {
        let mut keys = std::collections::HashSet::new();
        for _ in 0..100 {
            keys.insert(ApiKeyGenerator::generate());
        }
        // Collisions are statistically impossible
        assert_eq!(keys.len(), 100);
    }

    #[test]
    fn test_hash_deterministic() {
        let key = "ak_testkey1234567890123456789012345678901234567890123456789012345";
        let hash1 = ApiKeyGenerator::hash(key);
        let hash2 = ApiKeyGenerator::hash(key);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_different() {
        let key1 = ApiKeyGenerator::generate();
        let key2 = ApiKeyGenerator::generate();
        let hash1 = ApiKeyGenerator::hash(&key1);
        let hash2 = ApiKeyGenerator::hash(&key2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_validate_format_valid() {
        // Valid key
        let key = "ak_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        // Make it exactly 64 chars (3 + 61)
        let key = &key[0..64]; 
        // Need to construct a valid 64 char string
        let valid_key = format!("ak_{}", "a".repeat(61));
        assert!(ApiKeyGenerator::validate_format(&valid_key));
    }

    #[test]
    fn test_validate_format_missing_prefix() {
        let key = "bk_".to_string() + &"a".repeat(61);
        assert!(!ApiKeyGenerator::validate_format(&key));
    }

    #[test]
    fn test_validate_format_length() {
        let short = "ak_".to_string() + &"a".repeat(60); // 63 chars
        assert!(!ApiKeyGenerator::validate_format(&short));
        
        let long = "ak_".to_string() + &"a".repeat(62); // 65 chars
        assert!(!ApiKeyGenerator::validate_format(&long));
    }

    #[test]
    fn test_validate_format_invalid_chars() {
        let invalid = "ak_".to_string() + &"a".repeat(60) + "!";
        assert!(!ApiKeyGenerator::validate_format(&invalid));
    }
}
