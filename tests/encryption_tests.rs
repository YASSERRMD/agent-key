//! Integration tests for encryption service.

use agentkey_backend::services::encryption::EncryptionService;

const TEST_SECRET: &str = "integration-test-secret-32-chars!";

#[test]
fn test_encryption_service_roundtrip() {
    let service = EncryptionService::new(TEST_SECRET);
    
    // Test various plaintext sizes
    let test_cases = vec![
        "".to_string(),
        "short".to_string(),
        "medium length string for testing".to_string(),
        "A".repeat(1000),
        "Unicode: 日本語 🔐 العربية".to_string(),
    ];
    
    for plaintext in &test_cases {
        let encrypted = service.encrypt(plaintext)
            .expect("Encryption should succeed");
        let decrypted = service.decrypt(&encrypted)
            .expect("Decryption should succeed");
        
        assert_eq!(plaintext, &decrypted);
    }
}

#[test]
fn test_encryption_produces_unique_ciphertexts() {
    let service = EncryptionService::new(TEST_SECRET);
    let plaintext = "test message";
    
    let encrypted1 = service.encrypt(plaintext).unwrap();
    let encrypted2 = service.encrypt(plaintext).unwrap();
    let encrypted3 = service.encrypt(plaintext).unwrap();
    
    // All should be different due to random nonce
    assert_ne!(encrypted1, encrypted2);
    assert_ne!(encrypted2, encrypted3);
    assert_ne!(encrypted1, encrypted3);
    
    // All should decrypt to same value
    assert_eq!(plaintext, service.decrypt(&encrypted1).unwrap());
    assert_eq!(plaintext, service.decrypt(&encrypted2).unwrap());
    assert_eq!(plaintext, service.decrypt(&encrypted3).unwrap());
}

#[test]
fn test_encryption_tamper_detection() {
    let service = EncryptionService::new(TEST_SECRET);
    let encrypted = service.encrypt("secret data").unwrap();
    
    // Tamper with the ciphertext
    let mut bytes = hex::decode(&encrypted).unwrap();
    if let Some(last) = bytes.last_mut() {
        *last ^= 0xFF;
    }
    let tampered = hex::encode(bytes);
    
    // Decryption should fail
    assert!(service.decrypt(&tampered).is_err());
}

#[test]
fn test_different_keys_cannot_decrypt() {
    let service1 = EncryptionService::new(TEST_SECRET);
    let service2 = EncryptionService::new("different-key-32-characters-here");
    
    let encrypted = service1.encrypt("secret").unwrap();
    
    // Other service should not be able to decrypt
    assert!(service2.decrypt(&encrypted).is_err());
}

#[test]
fn test_encryption_of_binary_data() {
    let service = EncryptionService::new(TEST_SECRET);
    let binary_data: Vec<u8> = (0..256).map(|i| i as u8).collect();
    
    let encrypted = service.encrypt_bytes(&binary_data).unwrap();
    let decrypted = service.decrypt_bytes(&encrypted).unwrap();
    
    assert_eq!(binary_data, decrypted);
}

#[test]
fn test_encryption_with_special_characters() {
    let service = EncryptionService::new(TEST_SECRET);
    let plaintext = r#"{"key": "value", "special": "\n\t\r"}"#;
    
    let encrypted = service.encrypt(plaintext).unwrap();
    let decrypted = service.decrypt(&encrypted).unwrap();
    
    assert_eq!(plaintext, decrypted);
}

#[test]
fn test_invalid_hex_fails_gracefully() {
    let service = EncryptionService::new(TEST_SECRET);
    
    // Not valid hex
    assert!(service.decrypt("not-hex!!!").is_err());
    
    // Too short
    assert!(service.decrypt("abcd").is_err());
    
    // Empty
    assert!(service.decrypt("").is_err());
}
