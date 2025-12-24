use uuid::Uuid;

/// Generator for Additional Authenticated Data (AAD)
/// Used to bind encrypted credentials to specific agents and IDs
pub struct AadGenerator;

impl AadGenerator {
    /// Generate AAD from agent_id and credential_id
    /// Returns 32 bytes: [agent_id (16) || credential_id (16)]
    pub fn generate(agent_id: Uuid, credential_id: Uuid) -> Vec<u8> {
        let mut aad = Vec::with_capacity(32);
        aad.extend_from_slice(agent_id.as_bytes());
        aad.extend_from_slice(credential_id.as_bytes());
        aad
    }

    /// Verify AAD matches the expected IDs
    pub fn verify(aad: &[u8], agent_id: Uuid, credential_id: Uuid) -> bool {
        if aad.len() != 32 {
            return false;
        }
        let expected = Self::generate(agent_id, credential_id);
        aad == expected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_verify() {
        let agent_id = Uuid::new_v4();
        let cred_id = Uuid::new_v4();
        
        let aad = AadGenerator::generate(agent_id, cred_id);
        assert_eq!(aad.len(), 32);
        
        assert!(AadGenerator::verify(&aad, agent_id, cred_id));
    }

    #[test]
    fn test_verify_fails_on_mismatch() {
        let agent_id = Uuid::new_v4();
        let cred_id = Uuid::new_v4();
        let other_id = Uuid::new_v4();
        
        let aad = AadGenerator::generate(agent_id, cred_id);
        
        assert!(!AadGenerator::verify(&aad, other_id, cred_id));
        assert!(!AadGenerator::verify(&aad, agent_id, other_id));
    }
}
