// Simplified chain store implementation that compiles
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::primitives::{Result, Blake2bHash};
use crate::blockchain::Block;
use super::ChainStore;

/// Simple in-memory chain store for development/testing
pub struct SimpleChainStore {
    blocks: Arc<RwLock<HashMap<Blake2bHash, Block>>>,
    head_hash: Arc<RwLock<Blake2bHash>>,
    macro_head_hash: Arc<RwLock<Blake2bHash>>,
    election_head_hash: Arc<RwLock<Blake2bHash>>,
}

impl SimpleChainStore {
    pub fn new() -> Self {
        Self {
            blocks: Arc::new(RwLock::new(HashMap::new())),
            head_hash: Arc::new(RwLock::new(Blake2bHash::zero())),
            macro_head_hash: Arc::new(RwLock::new(Blake2bHash::zero())),
            election_head_hash: Arc::new(RwLock::new(Blake2bHash::zero())),
        }
    }
}

#[async_trait::async_trait]
impl ChainStore for SimpleChainStore {
    async fn get_block(&self, hash: &Blake2bHash) -> Result<Option<Block>> {
        let blocks = self.blocks.read().await;
        Ok(blocks.get(hash).cloned())
    }

    async fn get_block_at(&self, _block_number: u32) -> Result<Option<Block>> {
        // Simple implementation - would need proper indexing in production
        Ok(None)
    }

    async fn put_block(&self, block: &Block) -> Result<()> {
        let mut blocks = self.blocks.write().await;
        blocks.insert(block.hash(), block.clone());
        Ok(())
    }

    async fn get_head_hash(&self) -> Result<Blake2bHash> {
        Ok(*self.head_hash.read().await)
    }

    async fn set_head(&self, hash: &Blake2bHash) -> Result<()> {
        *self.head_hash.write().await = *hash;
        Ok(())
    }

    async fn get_macro_head_hash(&self) -> Result<Blake2bHash> {
        Ok(*self.macro_head_hash.read().await)
    }

    async fn set_macro_head(&self, hash: &Blake2bHash) -> Result<()> {
        *self.macro_head_hash.write().await = *hash;
        Ok(())
    }

    async fn get_election_head_hash(&self) -> Result<Blake2bHash> {
        Ok(*self.election_head_hash.read().await)
    }

    async fn set_election_head(&self, hash: &Blake2bHash) -> Result<()> {
        *self.election_head_hash.write().await = *hash;
        Ok(())
    }
}