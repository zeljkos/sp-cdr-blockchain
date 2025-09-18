// Real MDBX storage implementation using Albatross patterns
use std::{ops::Range, path::Path, sync::Arc};
use libmdbx::{NoWriteMap, TableFlags, WriteFlags};
use crate::primitives::{Result, BlockchainError, Blake2bHash};
use crate::blockchain::Block;
use super::ChainStore;

const GIGABYTE: usize = 1024 * 1024 * 1024;
const TERABYTE: usize = GIGABYTE * 1024;

/// Database config options (copied from Albatross)
pub struct DatabaseConfig {
    pub max_tables: Option<u64>,
    pub max_readers: Option<u32>,
    pub no_rdahead: bool,
    pub size: Option<Range<isize>>,
    pub growth_step: Option<isize>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            max_tables: Some(20),
            max_readers: None,
            no_rdahead: true,
            // Default max database size: 2TB
            size: Some(0..(2 * TERABYTE as isize)),
            // Default growth step: 4GB
            growth_step: Some(4 * GIGABYTE as isize),
        }
    }
}

impl From<DatabaseConfig> for libmdbx::DatabaseOptions {
    fn from(value: DatabaseConfig) -> Self {
        libmdbx::DatabaseOptions {
            max_tables: value.max_tables,
            max_readers: value.max_readers,
            no_rdahead: value.no_rdahead,
            mode: libmdbx::Mode::ReadWrite(libmdbx::ReadWriteOptions {
                sync_mode: libmdbx::SyncMode::Durable,
                min_size: value.size.as_ref().map(|r| r.start),
                max_size: value.size.map(|r| r.end),
                ..Default::default()
            }),
            liforeclaim: true,
            ..Default::default()
        }
    }
}

/// Real MDBX Database following Albatross patterns exactly
#[derive(Clone)]
pub struct MdbxChainStore {
    db: Arc<libmdbx::Database<NoWriteMap>>,
}

impl MdbxChainStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        std::fs::create_dir_all(path.as_ref())
            .map_err(|e| BlockchainError::Storage(format!("Failed to create directory: {}", e)))?;

        let config = DatabaseConfig::default();
        let db = libmdbx::Database::open_with_options(path, libmdbx::DatabaseOptions::from(config))
            .map_err(|e| BlockchainError::Storage(format!("MDBX open failed: {}", e)))?;

        let store = Self {
            db: Arc::new(db),
        };

        // Create required tables
        store.create_tables()?;

        Ok(store)
    }

    fn create_tables(&self) -> Result<()> {
        let txn = self.db.begin_rw_txn()
            .map_err(|e| BlockchainError::Storage(format!("Transaction failed: {}", e)))?;

        // Create blocks table
        if let Err(e) = txn.create_table(Some("blocks"), TableFlags::empty()) {
            // Ignore error if table already exists
            if !e.to_string().contains("already exists") {
                return Err(BlockchainError::Storage(format!("Create blocks table failed: {}", e)));
            }
        }

        // Create metadata table
        if let Err(e) = txn.create_table(Some("metadata"), TableFlags::empty()) {
            // Ignore error if table already exists
            if !e.to_string().contains("already exists") {
                return Err(BlockchainError::Storage(format!("Create metadata table failed: {}", e)));
            }
        }

        txn.commit()
            .map_err(|e| BlockchainError::Storage(format!("Transaction commit failed: {}", e)))?;

        Ok(())
    }

    // Direct MDBX put operation
    fn mdbx_put(&self, table_name: &str, key: &[u8], value: &[u8]) -> Result<()> {
        let txn = self.db.begin_rw_txn()
            .map_err(|e| BlockchainError::Storage(format!("Write transaction failed: {}", e)))?;

        let table = txn.open_table(Some(table_name))
            .map_err(|e| BlockchainError::Storage(format!("Open table failed: {}", e)))?;

        txn.put(&table, key, value, WriteFlags::empty())
            .map_err(|e| BlockchainError::Storage(format!("MDBX put failed: {}", e)))?;

        txn.commit()
            .map_err(|e| BlockchainError::Storage(format!("Transaction commit failed: {}", e)))?;

        Ok(())
    }

    // Direct MDBX get operation
    fn mdbx_get(&self, table_name: &str, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let txn = self.db.begin_ro_txn()
            .map_err(|e| BlockchainError::Storage(format!("Read transaction failed: {}", e)))?;

        let table = txn.open_table(Some(table_name))
            .map_err(|e| BlockchainError::Storage(format!("Open table failed: {}", e)))?;

        // Use explicit type annotation to avoid inference issues
        match txn.get::<Vec<u8>>(&table, key) {
            Ok(Some(data)) => Ok(Some(data)),
            Ok(None) => Ok(None),
            Err(e) => Err(BlockchainError::Storage(format!("MDBX get failed: {}", e))),
        }
    }
}

#[async_trait::async_trait]
impl ChainStore for MdbxChainStore {
    async fn get_block(&self, hash: &Blake2bHash) -> Result<Option<Block>> {
        let store = self.clone();
        let hash = *hash;

        tokio::task::spawn_blocking(move || {
            match store.mdbx_get("blocks", hash.as_bytes())? {
                Some(data) => {
                    let block: Block = bincode::deserialize(&data)
                        .map_err(|e| BlockchainError::Storage(format!("Block deserialize failed: {}", e)))?;
                    Ok(Some(block))
                }
                None => Ok(None),
            }
        })
        .await
        .map_err(|e| BlockchainError::Storage(format!("Task join error: {}", e)))?
    }

    async fn get_block_at(&self, _block_number: u32) -> Result<Option<Block>> {
        // Would need block number index - not implemented
        Ok(None)
    }

    async fn put_block(&self, block: &Block) -> Result<()> {
        let hash = block.hash();
        let serialized = bincode::serialize(block)
            .map_err(|e| BlockchainError::Storage(format!("Block serialize failed: {}", e)))?;

        let store = self.clone();
        tokio::task::spawn_blocking(move || {
            store.mdbx_put("blocks", hash.as_bytes(), &serialized)
        })
        .await
        .map_err(|e| BlockchainError::Storage(format!("Task join error: {}", e)))?
    }

    async fn get_head_hash(&self) -> Result<Blake2bHash> {
        let store = self.clone();
        tokio::task::spawn_blocking(move || {
            match store.mdbx_get("metadata", b"head")? {
                Some(data) => {
                    let hash: Blake2bHash = bincode::deserialize(&data)
                        .map_err(|e| BlockchainError::Storage(format!("Head hash deserialize failed: {}", e)))?;
                    Ok(hash)
                }
                None => Err(BlockchainError::Storage("No head hash found".to_string())),
            }
        })
        .await
        .map_err(|e| BlockchainError::Storage(format!("Task join error: {}", e)))?
    }

    async fn set_head(&self, hash: &Blake2bHash) -> Result<()> {
        let serialized = bincode::serialize(hash)
            .map_err(|e| BlockchainError::Storage(format!("Head hash serialize failed: {}", e)))?;

        let store = self.clone();
        tokio::task::spawn_blocking(move || {
            store.mdbx_put("metadata", b"head", &serialized)
        })
        .await
        .map_err(|e| BlockchainError::Storage(format!("Task join error: {}", e)))?
    }

    async fn get_macro_head_hash(&self) -> Result<Blake2bHash> {
        let store = self.clone();
        tokio::task::spawn_blocking(move || {
            match store.mdbx_get("metadata", b"macro_head")? {
                Some(data) => {
                    let hash: Blake2bHash = bincode::deserialize(&data)
                        .map_err(|e| BlockchainError::Storage(format!("Macro head deserialize failed: {}", e)))?;
                    Ok(hash)
                }
                None => Err(BlockchainError::Storage("No macro head hash found".to_string())),
            }
        })
        .await
        .map_err(|e| BlockchainError::Storage(format!("Task join error: {}", e)))?
    }

    async fn set_macro_head(&self, hash: &Blake2bHash) -> Result<()> {
        let serialized = bincode::serialize(hash)
            .map_err(|e| BlockchainError::Storage(format!("Macro head serialize failed: {}", e)))?;

        let store = self.clone();
        tokio::task::spawn_blocking(move || {
            store.mdbx_put("metadata", b"macro_head", &serialized)
        })
        .await
        .map_err(|e| BlockchainError::Storage(format!("Task join error: {}", e)))?
    }

    async fn get_election_head_hash(&self) -> Result<Blake2bHash> {
        let store = self.clone();
        tokio::task::spawn_blocking(move || {
            match store.mdbx_get("metadata", b"election_head")? {
                Some(data) => {
                    let hash: Blake2bHash = bincode::deserialize(&data)
                        .map_err(|e| BlockchainError::Storage(format!("Election head deserialize failed: {}", e)))?;
                    Ok(hash)
                }
                None => Err(BlockchainError::Storage("No election head hash found".to_string())),
            }
        })
        .await
        .map_err(|e| BlockchainError::Storage(format!("Task join error: {}", e)))?
    }

    async fn set_election_head(&self, hash: &Blake2bHash) -> Result<()> {
        let serialized = bincode::serialize(hash)
            .map_err(|e| BlockchainError::Storage(format!("Election head serialize failed: {}", e)))?;

        let store = self.clone();
        tokio::task::spawn_blocking(move || {
            store.mdbx_put("metadata", b"election_head", &serialized)
        })
        .await
        .map_err(|e| BlockchainError::Storage(format!("Task join error: {}", e)))?
    }
}