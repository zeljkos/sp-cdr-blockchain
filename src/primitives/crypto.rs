// Cryptographic utilities shared across the SP CDR blockchain

use serde::{Deserialize, Serialize};
use crate::primitives::primitives::Blake2bHash;

/// Additional crypto utilities for SP CDR use cases
/// (The main crypto functionality is in the crypto module)

/// Digital signature verification result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureVerificationResult {
    Valid,
    Invalid,
    UnknownKey,
    ExpiredSignature,
}

/// Cryptographic hash verification
pub fn verify_hash_integrity(data: &[u8], expected_hash: &Blake2bHash) -> bool {
    let computed_hash = crate::primitives::primitives::hash_data(data);
    computed_hash == *expected_hash
}

/// Create commitment to private data for ZK proofs
pub fn create_commitment(data: &[u8], randomness: &[u8]) -> Blake2bHash {
    let mut combined = Vec::new();
    combined.extend_from_slice(data);
    combined.extend_from_slice(randomness);
    crate::primitives::primitives::hash_data(&combined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_integrity_verification() {
        let data = b"test data for integrity check";
        let hash = crate::primitives::primitives::hash_data(data);
        
        assert!(verify_hash_integrity(data, &hash));
        assert!(!verify_hash_integrity(b"different data", &hash));
    }

    #[test]
    fn test_commitment_creation() {
        let data = b"secret data";
        let randomness = b"random_value_12345";
        
        let commitment1 = create_commitment(data, randomness);
        let commitment2 = create_commitment(data, randomness);
        
        // Same inputs should produce same commitment
        assert_eq!(commitment1, commitment2);
        
        // Different randomness should produce different commitment
        let commitment3 = create_commitment(data, b"different_random");
        assert_ne!(commitment1, commitment3);
    }
}