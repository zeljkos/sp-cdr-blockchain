// Chain management and blockchain state
use serde::{Deserialize, Serialize};
use crate::primitives::primitives::{Blake2bHash, NetworkId, Height};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainInfo {
    pub head_hash: Blake2bHash,
    pub head_block_number: u32,
    pub macro_head_hash: Blake2bHash,
    pub macro_head_block_number: u32,
    pub election_head_hash: Blake2bHash,
    pub election_head_block_number: u32,
    pub total_work: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainState {
    pub network_id: NetworkId,
    pub height: Height,
    pub head_hash: Blake2bHash,
    pub timestamp: u64,
}

impl ChainState {
    pub fn new(network_id: NetworkId) -> Self {
        Self {
            network_id,
            height: 0,
            head_hash: Blake2bHash::zero(),
            timestamp: 0,
        }
    }
}