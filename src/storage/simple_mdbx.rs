// Real Sled storage implementation for blockchain data
use crate::primitives::{Result, Blake2bHash, BlockchainError};
use crate::blockchain::Block;
use super::{ChainStore, sled_store::SledStore};
use serde::{Serialize, Deserialize};

/// Production Sled storage for blockchain with typed operations
pub struct SimpleMdbxStore {
    sled: SledStore,
    data_dir: String,
}

impl SimpleMdbxStore {
    pub fn new(data_dir: &str) -> Result<Self> {
        let sled = SledStore::new(data_dir)?;

        Ok(Self {
            sled,
            data_dir: data_dir.to_string(),
        })
    }

    /// Serialize and store a block
    pub async fn store_block(&self, block_hash: &Blake2bHash, block: &Block) -> Result<()> {
        let serialized = bincode::serialize(block)
            .map_err(|e| BlockchainError::Storage(format!("Block serialization failed: {}", e)))?;

        self.sled.put(block_hash, serialized).await
    }

    /// Load and deserialize a block
    pub async fn load_block(&self, block_hash: &Blake2bHash) -> Result<Option<Block>> {
        match self.sled.get(block_hash).await? {
            Some(data) => {
                let block = bincode::deserialize(&data)
                    .map_err(|e| BlockchainError::Storage(format!("Block deserialization failed: {}", e)))?;
                Ok(Some(block))
            }
            None => Ok(None),
        }
    }

    /// Store chain metadata (like head hash, height, etc.)
    pub async fn store_metadata(&self, key: &str, value: &Blake2bHash) -> Result<()> {
        let serialized = bincode::serialize(value)
            .map_err(|e| BlockchainError::Storage(format!("Metadata serialization failed: {}", e)))?;

        self.sled.put_metadata(key, serialized).await
    }

    /// Load chain metadata
    pub async fn load_metadata(&self, key: &str) -> Result<Option<Blake2bHash>> {
        match self.sled.get_metadata(key).await? {
            Some(data) => {
                let value = bincode::deserialize(&data)
                    .map_err(|e| BlockchainError::Storage(format!("Metadata deserialization failed: {}", e)))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Delete a block
    pub async fn delete_block(&self, block_hash: &Blake2bHash) -> Result<bool> {
        self.sled.delete(block_hash).await
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<super::sled_store::DatabaseStats> {
        self.sled.stats().await
    }

    /// Force sync to disk
    pub async fn sync_to_disk(&self) -> Result<()> {
        self.sled.sync().await
    }
}

#[async_trait::async_trait]
impl ChainStore for SimpleMdbxStore {
    async fn get_block(&self, hash: &Blake2bHash) -> Result<Option<Block>> {
        self.load_block(hash).await
    }

    async fn get_block_at(&self, _block_number: u32) -> Result<Option<Block>> {
        // Would need block number index for full implementation
        // For now, this is a placeholder
        Ok(None)
    }

    async fn put_block(&self, block: &Block) -> Result<()> {
        let hash = block.hash();
        self.store_block(&hash, block).await?;

        // Auto-sync every 100 blocks for performance/durability balance
        let stats = self.get_stats().await?;
        if stats.blocks_entries % 100 == 0 {
            self.sync_to_disk().await?;
        }

        Ok(())
    }

    async fn get_head_hash(&self) -> Result<Blake2bHash> {
        self.load_metadata("head").await?
            .ok_or_else(|| BlockchainError::Storage("No head hash found".to_string()))
    }

    async fn set_head(&self, hash: &Blake2bHash) -> Result<()> {
        self.store_metadata("head", hash).await
    }

    async fn get_macro_head_hash(&self) -> Result<Blake2bHash> {
        self.load_metadata("macro_head").await?
            .ok_or_else(|| BlockchainError::Storage("No macro head hash found".to_string()))
    }

    async fn set_macro_head(&self, hash: &Blake2bHash) -> Result<()> {
        self.store_metadata("macro_head", hash).await
    }

    async fn get_election_head_hash(&self) -> Result<Blake2bHash> {
        self.load_metadata("election_head").await?
            .ok_or_else(|| BlockchainError::Storage("No election head hash found".to_string()))
    }

    async fn set_election_head(&self, hash: &Blake2bHash) -> Result<()> {
        self.store_metadata("election_head", hash).await
    }
}

/// Contract storage implementation using Sled
pub struct SimpleMdbxContractStorage {
    sled: SledStore,
    data_dir: String,
}

impl SimpleMdbxContractStorage {
    pub fn new(data_dir: &str) -> Result<Self> {
        let contract_dir = format!("{}/contracts", data_dir);
        let sled = SledStore::new(&contract_dir)?;

        Ok(Self {
            sled,
            data_dir: contract_dir,
        })
    }

    /// Encode contract storage key
    fn encode_storage_key(contract: &Blake2bHash, key: &Blake2bHash) -> Blake2bHash {
        let mut combined = Vec::new();
        combined.extend_from_slice(contract.as_bytes());
        combined.extend_from_slice(key.as_bytes());
        Blake2bHash::from_data(&combined)
    }

    /// Encode contract code key
    fn encode_code_key(contract: &Blake2bHash) -> Blake2bHash {
        let mut prefixed = Vec::new();
        prefixed.extend_from_slice(b"CODE:");
        prefixed.extend_from_slice(contract.as_bytes());
        Blake2bHash::from_data(&prefixed)
    }
}

impl crate::smart_contracts::ContractStorage for SimpleMdbxContractStorage {
    fn get(&self, contract: &Blake2bHash, key: &Blake2bHash) -> Result<Option<Vec<u8>>> {
        let storage_key = Self::encode_storage_key(contract, key);

        // We need to make this sync for the trait, but MDBX operations are async
        // In production, you'd want to redesign the trait to be async
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.sled.get(&storage_key).await
            })
        })
    }

    fn set(&mut self, contract: &Blake2bHash, key: &Blake2bHash, value: Vec<u8>) -> Result<()> {
        let storage_key = Self::encode_storage_key(contract, key);

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.sled.put(&storage_key, value).await
            })
        })
    }

    fn get_code(&self, contract: &Blake2bHash) -> Result<Option<Vec<crate::smart_contracts::Instruction>>> {
        let code_key = Self::encode_code_key(contract);

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                match self.sled.get(&code_key).await? {
                    Some(data) => {
                        let code = bincode::deserialize(&data)
                            .map_err(|e| BlockchainError::Storage(format!("Code deserialization failed: {}", e)))?;
                        Ok(Some(code))
                    }
                    None => Ok(None),
                }
            })
        })
    }

    fn set_code(&mut self, contract: &Blake2bHash, code: Vec<crate::smart_contracts::Instruction>) -> Result<()> {
        let code_key = Self::encode_code_key(contract);
        let serialized = bincode::serialize(&code)
            .map_err(|e| BlockchainError::Storage(format!("Code serialization failed: {}", e)))?;

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.sled.put(&code_key, serialized).await
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_real_mdbx_store() {
        let temp_dir = "/tmp/test_real_mdbx";
        let _ = std::fs::remove_dir_all(temp_dir);

        let store = SimpleMdbxStore::new(temp_dir).unwrap();

        // Test head operations
        let test_hash = crate::primitives::primitives::hash_data(b"test_block");
        store.set_head(&test_hash).await.unwrap();
        let retrieved_hash = store.get_head_hash().await.unwrap();
        assert_eq!(test_hash, retrieved_hash);

        // Test persistence by syncing
        store.sync_to_disk().await.unwrap();

        // Create new store instance - should persist across instances
        let store2 = SimpleMdbxStore::new(temp_dir).unwrap();
        let loaded_hash = store2.get_head_hash().await.unwrap();
        assert_eq!(test_hash, loaded_hash);

        // Check database stats
        let stats = store2.get_stats().await.unwrap();
        assert!(stats.metadata_entries > 0);

        // Cleanup
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[tokio::test]
    async fn test_block_storage() {
        let temp_dir = "/tmp/test_mdbx_blocks";
        let _ = std::fs::remove_dir_all(temp_dir);

        let store = SimpleMdbxStore::new(temp_dir).unwrap();

        // Create a test block
        use crate::blockchain::*;
        let block = Block::Micro(MicroBlock {
            header: MicroHeader {
                version: 1,
                block_number: 1,
                timestamp: 1234567890,
                parent_hash: Blake2bHash::zero(),
                seed: [0u8; 32],
                extra_data: vec![],
                state_root: Blake2bHash::zero(),
                transactions_root: Blake2bHash::zero(),
                receipts_root: Blake2bHash::zero(),
                producer: crate::primitives::ProducerSlot { slot: 0, id: Blake2bHash::zero() },
                justification: None,
            },
            body: MicroBody { transactions: vec![] },
        });

        // Store and retrieve block
        let block_hash = block.hash();
        store.store_block(&block_hash, &block).await.unwrap();

        let retrieved_block = store.load_block(&block_hash).await.unwrap();
        assert!(retrieved_block.is_some());

        let retrieved_block = retrieved_block.unwrap();
        assert_eq!(retrieved_block.hash(), block_hash);

        // Cleanup
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}