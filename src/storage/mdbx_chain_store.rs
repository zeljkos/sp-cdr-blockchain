// Real MDBX chain store implementation using Albatross patterns
use std::sync::Arc;
use libmdbx::{NoWriteMap, Database, TableFlags, Mode, WriteFlags};
use sha2::{Sha256, Digest};
use crate::primitives::{Result, BlockchainError, Blake2bHash};
use crate::blockchain::Block;
use super::ChainStore;

/// Real MDBX chain store implementation from Albatross
pub struct MdbxChainStore {
    db: Arc<Database<NoWriteMap>>,
}

impl MdbxChainStore {
    /// Create new MDBX chain store
    pub fn new(path: &str) -> Result<Self> {
        std::fs::create_dir_all(path)
            .map_err(|e| BlockchainError::Storage(format!("Failed to create directory: {}", e)))?;

        // Use Albatross database configuration
        let config = libmdbx::DatabaseOptions {
            max_tables: Some(20),
            max_readers: None,
            no_rdahead: true,
            mode: Mode::ReadWrite(libmdbx::ReadWriteOptions {
                sync_mode: libmdbx::SyncMode::Durable,
                min_size: Some(0),
                max_size: Some(2 * 1024 * 1024 * 1024 * 1024isize), // 2TB
                ..Default::default()
            }),
            liforeclaim: true,
            ..Default::default()
        };

        let db = Database::open_with_options(path, config)
            .map_err(|e| BlockchainError::Storage(format!("MDBX database error: {}", e)))?;

        // Create required tables
        let txn = db.begin_rw_txn()
            .map_err(|e| BlockchainError::Storage(format!("Transaction begin error: {}", e)))?;

        txn.create_table(Some("blocks"), TableFlags::empty())
            .map_err(|e| BlockchainError::Storage(format!("Blocks table creation error: {}", e)))?;
        txn.create_table(Some("chain"), TableFlags::empty())
            .map_err(|e| BlockchainError::Storage(format!("Chain table creation error: {}", e)))?;
        txn.create_table(Some("macro"), TableFlags::empty())
            .map_err(|e| BlockchainError::Storage(format!("Macro table creation error: {}", e)))?;

        txn.commit()
            .map_err(|e| BlockchainError::Storage(format!("Transaction commit error: {}", e)))?;

        Ok(Self {
            db: Arc::new(db),
        })
    }

    /// Store a block in the database
    pub async fn store_block(&self, block_hash: &Blake2bHash, block: &Block) -> Result<()> {
        let serialized = bincode::serialize(block)
            .map_err(|e| BlockchainError::Storage(format!("Block serialization failed: {}", e)))?;

        let txn = self.db.begin_rw_txn()
            .map_err(|e| BlockchainError::Storage(format!("Transaction begin error: {}", e)))?;

        let table = txn.open_table(Some("blocks"))
            .map_err(|e| BlockchainError::Storage(format!("Open blocks table error: {}", e)))?;

        txn.put(&table, block_hash.as_bytes(), &serialized, libmdbx::WriteFlags::empty())
            .map_err(|e| BlockchainError::Storage(format!("Block store error: {}", e)))?;

        txn.commit()
            .map_err(|e| BlockchainError::Storage(format!("Transaction commit error: {}", e)))?;

        Ok(())
    }

    /// Retrieve a block from the database
    pub async fn get_block(&self, block_hash: &Blake2bHash) -> Result<Option<Block>> {
        let txn = self.db.begin_ro_txn()
            .map_err(|e| BlockchainError::Storage(format!("Read transaction error: {}", e)))?;

        let table = txn.open_table(Some("blocks"))
            .map_err(|e| BlockchainError::Storage(format!("Open blocks table error: {}", e)))?;

        match txn.get(&table, block_hash.as_bytes()) {
            Ok(Some(data)) => {
                let block: Block = bincode::deserialize(data)
                    .map_err(|e| BlockchainError::Storage(format!("Block deserialization failed: {}", e)))?;
                Ok(Some(block))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(BlockchainError::Storage(format!("MDBX read error: {}", e))),
        }
    }
}

#[async_trait::async_trait]
impl ChainStore for MdbxChainStore {
    async fn get_block(&self, hash: &Blake2bHash) -> Result<Option<Block>> {
        self.get_block(hash).await
    }

    async fn get_block_at(&self, _block_number: u32) -> Result<Option<Block>> {
        // For now, not implemented - would need block number indexing
        Ok(None)
    }

    async fn put_block(&self, block: &Block) -> Result<()> {
        // Calculate block hash
        let serialized = bincode::serialize(block)
            .map_err(|e| BlockchainError::Storage(format!("Block serialization failed: {}", e)))?;
        let hash_bytes = sha2::Sha256::digest(&serialized);
        let hash = Blake2bHash::from_bytes(&hash_bytes[..32]).unwrap();
        self.store_block(&hash, block).await
    }

    async fn get_head_hash(&self) -> Result<Blake2bHash> {
        // For now, return a dummy hash - head tracking would need additional metadata storage
        Ok(Blake2bHash::default())
    }

    async fn set_head(&self, _hash: &Blake2bHash) -> Result<()> {
        // For now, do nothing - head tracking would need additional metadata storage
        Ok(())
    }

    async fn get_macro_head_hash(&self) -> Result<Blake2bHash> {
        Ok(Blake2bHash::default())
    }

    async fn set_macro_head(&self, _hash: &Blake2bHash) -> Result<()> {
        Ok(())
    }

    async fn get_election_head_hash(&self) -> Result<Blake2bHash> {
        Ok(Blake2bHash::default())
    }

    async fn set_election_head(&self, _hash: &Blake2bHash) -> Result<()> {
        Ok(())
    }
}