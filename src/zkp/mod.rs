// Zero-knowledge proof components for SP CDR reconciliation blockchain
// Based on Nimiq's Albatross ZKP system adapted for CDR privacy

pub use proof_system::*;
pub use verifying_key::*;
pub use albatross_zkp::*;

pub(crate) mod proof_system;
pub mod verifying_key;
pub mod albatross_zkp;
pub mod circuits;
pub mod trusted_setup;

#[allow(dead_code)]
mod poseidon;

/// Re-export common types for convenience
pub use crate::primitives::{Blake2bHash, NetworkId};

/// Error types for ZKP operations
#[derive(Debug, thiserror::Error)]
pub enum ZKPError {
    #[error("Invalid ZKP proof")]
    InvalidProof,
    #[error("Proof verification failed: {0}")]
    VerificationFailed(String),
    #[error("Unsupported network for ZKP: {0:?}")]
    UnsupportedNetwork(NetworkId),
    #[error("CDR data encryption error: {0}")]
    EncryptionError(String),
    #[error("Privacy proof generation failed: {0}")]
    ProofGenerationFailed(String),
}

pub type Result<T> = std::result::Result<T, ZKPError>;