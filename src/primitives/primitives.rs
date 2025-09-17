// Core primitives extracted from nimiq-primitives and nimiq-hash
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

pub type Balance = u64;
pub type Height = u32;  // Following Albatross pattern
pub type Timestamp = u64;

/// Blake2b hash following Albatross pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Blake2bHash(pub [u8; 32]);

impl Blake2bHash {
    pub fn zero() -> Self {
        Blake2bHash([0u8; 32])
    }

    pub fn default() -> Self {
        Self::zero()
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Blake2bHash(bytes)
    }

    pub fn from_data(data: &[u8]) -> Self {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        Blake2bHash(result.into())
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl std::fmt::Display for Blake2bHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Network ID for SP consortium
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NetworkId {
    SPConsortium,
    DevNet,
    TestNet,
    MainNet,
    Operator { name: String, country: String },
}

impl NetworkId {
    pub fn new(name: &str, country: &str) -> Self {
        NetworkId::Operator {
            name: name.to_string(),
            country: country.to_string(),
        }
    }
}

impl std::fmt::Display for NetworkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkId::SPConsortium => write!(f, "SPConsortium"),
            NetworkId::DevNet => write!(f, "DevNet"),
            NetworkId::TestNet => write!(f, "TestNet"),
            NetworkId::MainNet => write!(f, "MainNet"),
            NetworkId::Operator { name, country } => write!(f, "{}:{}", name, country),
        }
    }
}

/// Policy constants following Albatross
pub struct Policy;

impl Policy {
    /// Number of blocks in an epoch (macro block interval)
    pub const EPOCH_LENGTH: u32 = 32;
    
    /// Number of blocks in a batch (micro block batch)
    pub const BATCH_LENGTH: u32 = 8;
    
    /// Genesis block number
    pub const GENESIS_BLOCK_NUMBER: u32 = 0;
    
    /// Block time in milliseconds
    pub const BLOCK_TIME: u64 = 1000; // 1 second for SP reconciliation
}

pub fn hash_data(data: &[u8]) -> Blake2bHash {
    let mut hasher = Sha256::new();
    hasher.update(data);
    Blake2bHash(hasher.finalize().into())
}

pub fn hash_json<T: serde::Serialize>(data: &T) -> Blake2bHash {
    let json = serde_json::to_string(data).unwrap();
    hash_data(json.as_bytes())
}