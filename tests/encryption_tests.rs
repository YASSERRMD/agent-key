//! Integration tests for encryption service.

use agentkey_backend::services::encryption::EncryptionService;

// 32 bytes as 64 hex chars
const TEST_KEY_HEX: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
const DUMMY_AAD: &[u8] = b"test-aad";

#[test]
fn test_encryption_service_roundtrip() {
    let service = EncryptionService::new(TEST_KEY_HEX.to_string())
        .expect("Service initialization failed");
    
    // Test various plaintext sizes
    let test_cases = vec![
        "",
        "short",
        "medium length string for testing",
        "A".repeat(1000).as_str(),
        "Unicode: 日本語 🔐 العربية",
    ];
    
    for plaintext in &test_cases {
        let plaintext_bytes = plaintext.as_bytes();
        let encrypted = service.encrypt(plaintext_bytes, DUMMY_AAD)
            .expect("Encryption should succeed");
        let decrypted = service.decrypt(&encrypted, DUMMY_AAD)
            .expect("Decryption should succeed");
        
        assert_eq!(plaintext_bytes, &decrypted[..]);
    }
}

#[test]
fn test_encryption_produces_unique_ciphertexts() {
    let service = EncryptionService::new(TEST_KEY_HEX.to_string()).unwrap();
    let plaintext = b"test message";
    
    let encrypted1 = service.encrypt(plaintext, DUMMY_AAD).unwrap();
    let encrypted2 = service.encrypt(plaintext, DUMMY_AAD).unwrap();
    
    // All should be different due to random nonce
    assert_ne!(encrypted1, encrypted2);
    
    // All should decrypt to same value
    assert_eq!(plaintext, &service.decrypt(&encrypted1, DUMMY_AAD).unwrap()[..]);
    assert_eq!(plaintext, &service.decrypt(&encrypted2, DUMMY_AAD).unwrap()[..]);
}

#[test]
fn test_authentication_tag_validation() {
    let service = EncryptionService::new(TEST_KEY_HEX.to_string()).unwrap();
    let plaintext = b"secret data";
    let encrypted = service.encrypt(plaintext, DUMMY_AAD).unwrap();
    
    // 1. Tamper with ciphertext
    let mut tampered = encrypted.clone();
    if let Some(last) = tampered.last_mut() {
        *last ^= 0xFF;
    }
    assert!(service.decrypt(&tampered, DUMMY_AAD).is_err());

    // 2. Wrong AAD
    assert!(service.decrypt(&encrypted, b"wrong-aad").is_err());
}

#[test]
fn test_different_keys_cannot_decrypt() {
    let service1 = EncryptionService::new(TEST_KEY_HEX.to_string()).unwrap();
    // Key offset by 1
    let other_key = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1e";
    let service2 = EncryptionService::new(other_key.to_string()).unwrap();
    
    let encrypted = service1.encrypt(b"secret", DUMMY_AAD).unwrap();
    
    // Other service should not be able to decrypt
    assert!(service2.decrypt(&encrypted, DUMMY_AAD).is_err());
}
