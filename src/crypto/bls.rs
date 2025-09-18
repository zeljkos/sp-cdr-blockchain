// Real BLS signature implementation using blst crate
// Production-grade BLS12-381 signatures for SP consortium

use blst::{
    min_pk::{SecretKey, PublicKey, Signature, AggregatePublicKey, AggregateSignature},
    BLST_ERROR, blst_scalar_from_bendian, blst_bendian_from_scalar,
};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::primitives::{Blake2bHash, Result, BlockchainError};

// Domain Separation Tag for SP consortium
const DST: &[u8] = b"SP_CDR_CONSORTIUM_BLS_SIG";

/// Real BLS private key using blst
#[derive(Clone, Debug)]
pub struct BLSPrivateKey {
    secret_key: SecretKey,
}

/// Real BLS public key using blst
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BLSPublicKey {
    #[serde(with = "hex")]
    compressed: [u8; 48], // BLS12-381 G1 compressed point
}

/// Real BLS signature using blst
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BLSSignature {
    #[serde(with = "hex")]
    compressed: [u8; 96], // BLS12-381 G2 compressed point
}

/// BLS aggregate signature for multi-party signing
#[derive(Clone, Debug)]
pub struct BLSAggregateSignature {
    signature: AggregateSignature,
}

/// BLS aggregate public key for multi-party verification
#[derive(Clone, Debug)]
pub struct BLSAggregatePublicKey {
    public_key: AggregatePublicKey,
}

#[derive(Debug, thiserror::Error)]
pub enum BLSError {
    #[error("Invalid private key")]
    InvalidPrivateKey,
    #[error("Invalid public key")]
    InvalidPublicKey,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),
    #[error("Signature verification failed")]
    VerificationFailed,
    #[error("Aggregation failed: {0}")]
    AggregationFailed(String),
}

impl BLSPrivateKey {
    /// Generate a new random BLS private key
    pub fn generate() -> Result<Self> {
        let mut ikm = [0u8; 32];
        getrandom::getrandom(&mut ikm)
            .map_err(|e| BlockchainError::Crypto(format!("RNG failed: {}", e)))?;

        let secret_key = SecretKey::key_gen(&ikm, &[])
            .map_err(|_| BlockchainError::Crypto("BLS key generation failed".to_string()))?;

        Ok(Self { secret_key })
    }

    /// Create private key from bytes (for deterministic keys)
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 32 {
            return Err(BlockchainError::Crypto("BLS private key must be 32 bytes".to_string()));
        }

        let secret_key = SecretKey::key_gen(bytes, &[])
            .map_err(|_| BlockchainError::Crypto("Invalid BLS private key bytes".to_string()))?;

        Ok(Self { secret_key })
    }

    /// Get the corresponding public key
    pub fn public_key(&self) -> BLSPublicKey {
        let pubkey = self.secret_key.sk_to_pk();
        BLSPublicKey {
            compressed: pubkey.compress(),
        }
    }

    /// Sign a message with this private key
    pub fn sign(&self, message: &[u8]) -> Result<BLSSignature> {
        let signature = self.secret_key.sign(message, DST, &[]);

        Ok(BLSSignature {
            compressed: signature.compress(),
        })
    }

    /// Export private key bytes (for storage - handle with care!)
    pub fn to_bytes(&self) -> [u8; 32] {
        self.secret_key.to_bytes()
    }
}

impl BLSPublicKey {
    /// Create public key from compressed bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 48 {
            return Err(BlockchainError::Crypto("BLS public key must be 48 bytes".to_string()));
        }

        let mut compressed = [0u8; 48];
        compressed.copy_from_slice(bytes);

        // Validate the public key
        let pubkey = PublicKey::from_bytes(&compressed)
            .map_err(|_| BlockchainError::Crypto("Invalid BLS public key".to_string()))?;

        // Note: blst::PublicKey::from_bytes already validates the key
        // so if we reach here, the key is valid

        Ok(Self { compressed })
    }

    /// Get compressed public key bytes
    pub fn to_bytes(&self) -> &[u8; 48] {
        &self.compressed
    }

    /// Convert to hex string for display/storage
    pub fn to_hex(&self) -> String {
        hex::encode(self.compressed)
    }
}

impl BLSSignature {
    /// Create signature from compressed bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 96 {
            return Err(BlockchainError::Crypto("BLS signature must be 96 bytes".to_string()));
        }

        let mut compressed = [0u8; 96];
        compressed.copy_from_slice(bytes);

        // Validate the signature format
        let _signature = Signature::from_bytes(&compressed)
            .map_err(|_| BlockchainError::Crypto("Invalid BLS signature format".to_string()))?;

        Ok(Self { compressed })
    }

    /// Verify signature against public key and message
    pub fn verify(&self, public_key: &BLSPublicKey, message: &[u8]) -> Result<bool> {
        let pubkey = PublicKey::from_bytes(&public_key.compressed)
            .map_err(|_| BlockchainError::Crypto("Invalid public key for verification".to_string()))?;

        let signature = Signature::from_bytes(&self.compressed)
            .map_err(|_| BlockchainError::Crypto("Invalid signature for verification".to_string()))?;

        let result = signature.verify(true, message, DST, &[], &pubkey, true);
        Ok(result == BLST_ERROR::BLST_SUCCESS)
    }

    /// Get compressed signature bytes
    pub fn to_bytes(&self) -> &[u8; 96] {
        &self.compressed
    }

    /// Convert to hex string for display/storage
    pub fn to_hex(&self) -> String {
        hex::encode(self.compressed)
    }
}

/// BLS Verifier for SP consortium operations
pub struct BLSVerifier {
    /// Known public keys for SP operators
    sp_operators: HashMap<String, BLSPublicKey>,
}

impl BLSVerifier {
    pub fn new() -> Self {
        Self {
            sp_operators: HashMap::new(),
        }
    }

    /// Register an SP operator's public key
    pub fn register_operator(&mut self, operator_name: &str, public_key: BLSPublicKey) {
        self.sp_operators.insert(operator_name.to_string(), public_key);
    }

    /// Verify operator signature for CDR/settlement data
    pub fn verify_operator_signature(
        &self,
        operator_name: &str,
        message: &[u8],
        signature_bytes: &[u8],
    ) -> Result<bool> {
        let public_key = self.sp_operators.get(operator_name)
            .ok_or_else(|| BlockchainError::Crypto(format!("Unknown operator: {}", operator_name)))?;

        let signature = BLSSignature::from_bytes(signature_bytes)?;
        signature.verify(public_key, message)
    }

    /// Verify multi-party signature from multiple operators
    pub fn verify_multi_party_signature(
        &self,
        operator_names: &[String],
        message: &[u8],
        aggregate_signature_bytes: &[u8],
    ) -> Result<bool> {
        if operator_names.is_empty() {
            return Err(BlockchainError::Crypto("No operators specified".to_string()));
        }

        // Collect public keys
        let mut public_keys = Vec::new();
        for operator_name in operator_names {
            let pubkey = self.sp_operators.get(operator_name)
                .ok_or_else(|| BlockchainError::Crypto(format!("Unknown operator: {}", operator_name)))?;

            let blst_pubkey = PublicKey::from_bytes(&pubkey.compressed)
                .map_err(|_| BlockchainError::Crypto("Invalid operator public key".to_string()))?;

            public_keys.push(blst_pubkey);
        }

        // Aggregate public keys using the correct blst API
        let agg_pubkey = match public_keys.len() {
            0 => return Err(BlockchainError::Crypto("No public keys to aggregate".to_string())),
            1 => public_keys[0].clone(),
            _ => {
                let mut iter = public_keys.iter();
                let first = iter.next().unwrap();
                let mut agg = AggregatePublicKey::from_public_key(first);
                for pk in iter {
                    agg.add_public_key(pk, true).map_err(|_| BlockchainError::Crypto("Public key aggregation failed".to_string()))?;
                }
                agg.to_public_key()
            }
        };

        // Verify aggregate signature
        let signature = Signature::from_bytes(aggregate_signature_bytes)
            .map_err(|_| BlockchainError::Crypto("Invalid aggregate signature".to_string()))?;

        let result = signature.verify(true, message, DST, &[], &agg_pubkey, true);
        Ok(result == BLST_ERROR::BLST_SUCCESS)
    }
}

/// Aggregate multiple BLS signatures into one
pub fn aggregate_signatures(signatures: &[BLSSignature]) -> Result<BLSSignature> {
    if signatures.is_empty() {
        return Err(BlockchainError::Crypto("No signatures to aggregate".to_string()));
    }

    let mut blst_signatures = Vec::new();
    for sig in signatures {
        let blst_sig = Signature::from_bytes(&sig.compressed)
            .map_err(|_| BlockchainError::Crypto("Invalid signature for aggregation".to_string()))?;
        blst_signatures.push(blst_sig);
    }

    let aggregate = match blst_signatures.len() {
        0 => return Err(BlockchainError::Crypto("No signatures to aggregate".to_string())),
        1 => blst_signatures[0].clone(),
        _ => {
            let mut iter = blst_signatures.iter();
            let first = iter.next().unwrap();
            let mut agg = AggregateSignature::from_signature(first);
            for sig in iter {
                agg.add_signature(sig, true).map_err(|_| BlockchainError::Crypto("Signature aggregation failed".to_string()))?;
            }
            agg.to_signature()
        }
    };

    Ok(BLSSignature {
        compressed: aggregate.compress(),
    })
}

/// Aggregate multiple BLS public keys into one
pub fn aggregate_public_keys(public_keys: &[BLSPublicKey]) -> Result<BLSPublicKey> {
    if public_keys.is_empty() {
        return Err(BlockchainError::Crypto("No public keys to aggregate".to_string()));
    }

    let mut blst_pubkeys = Vec::new();
    for pubkey in public_keys {
        let blst_pubkey = PublicKey::from_bytes(&pubkey.compressed)
            .map_err(|_| BlockchainError::Crypto("Invalid public key for aggregation".to_string()))?;
        blst_pubkeys.push(blst_pubkey);
    }

    let aggregate = match blst_pubkeys.len() {
        0 => return Err(BlockchainError::Crypto("No public keys to aggregate".to_string())),
        1 => blst_pubkeys[0].clone(),
        _ => {
            let mut iter = blst_pubkeys.iter();
            let first = iter.next().unwrap();
            let mut agg = AggregatePublicKey::from_public_key(first);
            for pk in iter {
                agg.add_public_key(pk, true).map_err(|_| BlockchainError::Crypto("Public key aggregation failed".to_string()))?;
            }
            agg.to_public_key()
        }
    };

    Ok(BLSPublicKey {
        compressed: aggregate.compress(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bls_key_generation() {
        let private_key = BLSPrivateKey::generate().unwrap();
        let public_key = private_key.public_key();

        // Keys should be valid sizes
        assert_eq!(private_key.to_bytes().len(), 32);
        assert_eq!(public_key.to_bytes().len(), 48);
    }

    #[test]
    fn test_bls_sign_and_verify() {
        let private_key = BLSPrivateKey::generate().unwrap();
        let public_key = private_key.public_key();

        let message = b"SP Consortium Settlement Batch 001";
        let signature = private_key.sign(message).unwrap();

        // Signature should verify correctly
        assert!(signature.verify(&public_key, message).unwrap());

        // Should fail with wrong message
        let wrong_message = b"Different message";
        assert!(!signature.verify(&public_key, wrong_message).unwrap());
    }

    #[test]
    fn test_bls_signature_aggregation() {
        // Generate three key pairs (T-Mobile, Vodafone, Orange)
        let tmobile_sk = BLSPrivateKey::generate().unwrap();
        let vodafone_sk = BLSPrivateKey::generate().unwrap();
        let orange_sk = BLSPrivateKey::generate().unwrap();

        let tmobile_pk = tmobile_sk.public_key();
        let vodafone_pk = vodafone_sk.public_key();
        let orange_pk = orange_sk.public_key();

        let message = b"Settlement: T-Mobile 1.2M EUR -> Vodafone, Vodafone 800K EUR -> Orange";

        // Each operator signs the settlement
        let tmobile_sig = tmobile_sk.sign(message).unwrap();
        let vodafone_sig = vodafone_sk.sign(message).unwrap();
        let orange_sig = orange_sk.sign(message).unwrap();

        // Aggregate signatures
        let signatures = vec![tmobile_sig, vodafone_sig, orange_sig];
        let aggregate_sig = aggregate_signatures(&signatures).unwrap();

        // Aggregate public keys
        let public_keys = vec![tmobile_pk, vodafone_pk, orange_pk];
        let aggregate_pk = aggregate_public_keys(&public_keys).unwrap();

        // Verify aggregate signature
        assert!(aggregate_sig.verify(&aggregate_pk, message).unwrap());

        // Should fail with wrong message
        assert!(!aggregate_sig.verify(&aggregate_pk, b"Wrong message").unwrap());
    }

    #[test]
    fn test_sp_consortium_workflow() {
        let mut verifier = BLSVerifier::new();

        // Generate keys for SP operators
        let tmobile_sk = BLSPrivateKey::generate().unwrap();
        let vodafone_sk = BLSPrivateKey::generate().unwrap();
        let orange_sk = BLSPrivateKey::generate().unwrap();

        // Register operators
        verifier.register_operator("T-Mobile-DE", tmobile_sk.public_key());
        verifier.register_operator("Vodafone-UK", vodafone_sk.public_key());
        verifier.register_operator("Orange-FR", orange_sk.public_key());

        let settlement_data = b"CDR_Settlement_Batch_12345: Total 2.4M EUR cross-network charges";

        // Each operator signs
        let tmobile_sig = tmobile_sk.sign(settlement_data).unwrap();
        let vodafone_sig = vodafone_sk.sign(settlement_data).unwrap();
        let orange_sig = orange_sk.sign(settlement_data).unwrap();

        // Verify individual signatures
        assert!(verifier.verify_operator_signature("T-Mobile-DE", settlement_data, tmobile_sig.to_bytes()).unwrap());
        assert!(verifier.verify_operator_signature("Vodafone-UK", settlement_data, vodafone_sig.to_bytes()).unwrap());
        assert!(verifier.verify_operator_signature("Orange-FR", settlement_data, orange_sig.to_bytes()).unwrap());

        // Create aggregate signature for consensus
        let signatures = vec![tmobile_sig, vodafone_sig, orange_sig];
        let aggregate_sig = aggregate_signatures(&signatures).unwrap();

        // Verify multi-party signature
        let operators = vec!["T-Mobile-DE".to_string(), "Vodafone-UK".to_string(), "Orange-FR".to_string()];
        assert!(verifier.verify_multi_party_signature(&operators, settlement_data, aggregate_sig.to_bytes()).unwrap());

        println!("✅ SP Consortium BLS workflow test passed!");
    }
}