// Cryptographic components for SP CDR reconciliation blockchain
// BLS signatures and key management adapted from Nimiq Albatross

pub mod bls;
pub mod keys;
pub mod signatures;

pub use bls::*;
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