// Error types following Albatross pattern
use thiserror::Error;

pub type Result<T> = std::result::Result<T, BlockchainError>;

#[derive(Error, Debug)]
pub enum BlockchainError {
    #[error("Block validation failed: {0}")]
    BlockValidation(String),
    
    #[error("Transaction validation failed: {0}")]
    InvalidTransaction(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Consensus error: {0}")]
    Consensus(String),
    
    #[error("Cryptography error: {0}")]
    Crypto(String),
    
    #[error("ZK proof error: {0}")]
    ZkProof(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Invalid proof")]
    InvalidProof,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Contract not found")]
    ContractNotFound,

    #[error("Stack overflow")]
    StackOverflow,

    #[error("Stack underflow")]
    StackUnderflow,

    #[error("Out of gas")]
    OutOfGas,
}

/// Event types following Albatross blockchain events
#[derive(Debug, Clone)]
pub enum BlockchainEvent {
    /// Block was extended to the main chain
    Extended(crate::Blake2bHash),
    
    /// Block was reverted from the main chain
    Reverted(crate::Blake2bHash),
    
    /// Chain rebranched to different fork
    Rebranched {
        old_blocks: Vec<crate::Blake2bHash>,
        new_blocks: Vec<crate::Blake2bHash>,
    },
    
    /// Finality changed (macro block)
    Finalized(crate::Blake2bHash),
}

/// Consensus events following Albatross
#[derive(Debug, Clone)]
pub enum ConsensusEvent {
    /// Consensus is established
    Established { synced_validity_window: bool },

    /// Consensus was lost
    Lost,

    /// Waiting for more peers
    Waiting,
}

/// Conversion from CryptoError to BlockchainError
impl From<crate::crypto::CryptoError> for BlockchainError {
    fn from(err: crate::crypto::CryptoError) -> Self {
        BlockchainError::Crypto(err.to_string())
    }
}

/// Conversion from std::io::Error to BlockchainError
impl From<std::io::Error> for BlockchainError {
    fn from(err: std::io::Error) -> Self {
        BlockchainError::Storage(err.to_string())
    }
}

/// Conversion from libp2p noise error to BlockchainError
impl From<libp2p::noise::Error> for BlockchainError {
    fn from(err: libp2p::noise::Error) -> Self {
        BlockchainError::NetworkError(err.to_string())
    }
}

/// Conversion from libp2p gossipsub error to BlockchainError
impl From<libp2p::gossipsub::SubscriptionError> for BlockchainError {
    fn from(err: libp2p::gossipsub::SubscriptionError) -> Self {
        BlockchainError::NetworkError(err.to_string())
    }
}

/// Conversion from libp2p gossipsub publish error to BlockchainError
impl From<libp2p::gossipsub::PublishError> for BlockchainError {
    fn from(err: libp2p::gossipsub::PublishError) -> Self {
        BlockchainError::NetworkError(err.to_string())
    }
}

/// Conversion from libp2p transport error to BlockchainError
impl From<libp2p::TransportError<std::io::Error>> for BlockchainError {
    fn from(err: libp2p::TransportError<std::io::Error>) -> Self {
        BlockchainError::NetworkError(err.to_string())
    }
}

/// Conversion from libp2p dial error to BlockchainError
impl From<libp2p::swarm::DialError> for BlockchainError {
    fn from(err: libp2p::swarm::DialError) -> Self {
        BlockchainError::NetworkError(err.to_string())
    }
}

/// Conversion from libp2p listen error to BlockchainError
impl From<libp2p::swarm::ListenError> for BlockchainError {
    fn from(err: libp2p::swarm::ListenError) -> Self {
        BlockchainError::NetworkError(err.to_string())
    }
}

/// Conversion from multiaddr parse error to BlockchainError
impl From<libp2p::multiaddr::Error> for BlockchainError {
    fn from(err: libp2p::multiaddr::Error) -> Self {
        BlockchainError::NetworkError(err.to_string())
    }
}