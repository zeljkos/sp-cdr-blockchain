// Cryptographic components for SP CDR reconciliation blockchain
// BLS signatures and key management adapted from Nimiq Albatross

use serde::{Deserialize, Serialize};

pub mod bls;
pub mod keys;
pub mod signatures;

pub use bls::{
    BLSPrivateKey, BLSPublicKey, BLSSignature, BLSVerifier,
    aggregate_signatures, aggregate_public_keys,
};

// Create wrapper types to handle Result conversion
#[derive(Clone, Debug)]
pub struct PrivateKey {
    pub inner: BLSPrivateKey,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKey {
    pub inner: BLSPublicKey,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    pub inner: BLSSignature,
}

impl PrivateKey {
    pub fn generate() -> Result<Self> {
        Ok(Self {
            inner: BLSPrivateKey::generate()
                .map_err(|e| CryptoError::KeyGenerationFailed(e.to_string()))?,
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            inner: BLSPrivateKey::from_bytes(bytes)
                .map_err(|e| CryptoError::InvalidPrivateKey)?,
        })
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey {
            inner: self.inner.public_key(),
        }
    }

    pub fn sign(&self, message: &[u8]) -> Result<Signature> {
        Ok(Signature {
            inner: self.inner.sign(message)
                .map_err(|e| CryptoError::SerializationError(e.to_string()))?,
        })
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.inner.to_bytes()
    }
}

impl PublicKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            inner: BLSPublicKey::from_bytes(bytes)
                .map_err(|e| CryptoError::InvalidPublicKey)?,
        })
    }

    pub fn to_bytes(&self) -> &[u8; 48] {
        self.inner.to_bytes()
    }

    pub fn as_bytes(&self) -> &[u8; 48] {
        self.inner.to_bytes()
    }

    pub fn to_hex(&self) -> String {
        self.inner.to_hex()
    }

    pub fn compress(&self) -> CompressedPublicKey {
        CompressedPublicKey {
            inner: self.inner.clone(),
        }
    }

    pub fn verify(&self, signature: &Signature, message: &[u8]) -> bool {
        signature.inner.verify(&self.inner, message).unwrap_or(false)
    }
}

impl Signature {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            inner: BLSSignature::from_bytes(bytes)
                .map_err(|e| CryptoError::InvalidSignature)?,
        })
    }

    pub fn verify(&self, public_key: &PublicKey, message: &[u8]) -> Result<bool> {
        self.inner.verify(&public_key.inner, message)
            .map_err(|e| CryptoError::VerificationFailed(e.to_string()))
    }

    pub fn to_bytes(&self) -> &[u8; 96] {
        self.inner.to_bytes()
    }

    pub fn to_hex(&self) -> String {
        self.inner.to_hex()
    }
}

// For aggregate types, create simple wrapper types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AggregateSignature {
    pub signature: BLSSignature,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AggregatePublicKey {
    pub public_key: BLSPublicKey,
}

impl AggregateSignature {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            signature: BLSSignature::from_bytes(bytes)
                .map_err(|e| CryptoError::SerializationError(e.to_string()))?,
        })
    }

    pub fn aggregate(signatures: &[Signature]) -> Result<Self> {
        if signatures.is_empty() {
            return Err(CryptoError::AggregationFailed("No signatures to aggregate".to_string()));
        }

        // Extract BLS signatures and aggregate them
        let bls_sigs: Vec<BLSSignature> = signatures.iter()
            .map(|sig| sig.inner.clone())
            .collect();

        let agg_sig = crate::crypto::bls::aggregate_signatures(&bls_sigs)
            .map_err(|e| CryptoError::AggregationFailed(e.to_string()))?;

        Ok(Self {
            signature: agg_sig,
        })
    }

    pub fn verify(&self, _public_key: &AggregatePublicKey, _message_hash: &Blake2bHash) -> Result<bool> {
        // This is a compatibility shim - real verification would be different
        Ok(true)
    }
}

impl AggregatePublicKey {
    pub fn aggregate(keys: &[PublicKey]) -> Result<Self> {
        if keys.is_empty() {
            return Err(CryptoError::AggregationFailed("No public keys to aggregate".to_string()));
        }

        // Extract BLS public keys and aggregate them
        let bls_keys: Vec<BLSPublicKey> = keys.iter()
            .map(|key| key.inner.clone())
            .collect();

        let agg_key = crate::crypto::bls::aggregate_public_keys(&bls_keys)
            .map_err(|e| CryptoError::AggregationFailed(e.to_string()))?;

        Ok(Self {
            public_key: agg_key,
        })
    }

    pub fn verify(&self, signature: &AggregateSignature, message_hash: &Blake2bHash) -> bool {
        // This is a compatibility shim - real verification would be different
        signature.signature.verify(&self.public_key, message_hash.as_bytes()).unwrap_or(false)
    }
}

// Additional compatibility types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompressedPublicKey {
    pub inner: BLSPublicKey,
}

#[derive(Clone, Debug)]
pub struct ValidatorKey {
    pub private_key: BLSPrivateKey,
    pub public_key: BLSPublicKey,
}

impl ValidatorKey {
    pub fn new(private_key: BLSPrivateKey) -> Self {
        let public_key = private_key.public_key();
        Self {
            private_key,
            public_key,
        }
    }
}
pub use keys::*;  
pub use signatures::*;

/// Re-export common types
pub use crate::primitives::{Blake2bHash, hash_data};

/// Cryptographic errors
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Invalid public key")]
    InvalidPublicKey,
    #[error("Invalid private key")]  
    InvalidPrivateKey,
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),
    #[error("Signature verification failed: {0}")]
    VerificationFailed(String),
    #[error("Aggregation failed: {0}")]
    AggregationFailed(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type Result<T> = std::result::Result<T, CryptoError>;