// Real MDBX database wrapper for blockchain storage
use crate::primitives::{Blake2bHash, Result, BlockchainError};
use libmdbx::*;
use std::path::Path;
use std::sync::Arc;

/// Production MDBX storage with real persistence
pub struct MdbxStore {
    env: Arc<Environment<WriteMap>>,
    blocks_db: Database<'static>,
    metadata_db: Database<'static>,
    transactions_db: Database<'static>,
}

impl MdbxStore {
    pub fn new(path: &str) -> Result<Self> {
        // Create the database directory if it doesn't exist
        std::fs::create_dir_all(path)
            .map_err(|e| BlockchainError::Storage(format!("Failed to create directory: {}", e)))?;

        // Initialize MDBX environment
        let env = Environment::new()
            .set_max_dbs(10)           // Support multiple databases
            .set_max_readers(126)      // Allow multiple concurrent readers
            .set_geometry(-1, -1, 1024 * 1024 * 1024, -1, -1, -1) // 1GB max size
            .open(Path::new(path))
            .map_err(|e| BlockchainError::Storage(format!("Failed to open MDBX environment: {}", e)))?;

        let env = Arc::new(env);

        // Create databases for different data types
        let blocks_db = {
            let txn = env.begin_rw_txn()
                .map_err(|e| BlockchainError::Storage(format!("Failed to begin transaction: {}", e)))?;
            let db = txn.create_db(Some("blocks"), DatabaseFlags::empty())
                .map_err(|e| BlockchainError::Storage(format!("Failed to create blocks database: {}", e)))?;
            txn.commit()
                .map_err(|e| BlockchainError::Storage(format!("Failed to commit transaction: {}", e)))?;
            db
        };

        let metadata_db = {
            let txn = env.begin_rw_txn()
                .map_err(|e| BlockchainError::Storage(format!("Failed to begin transaction: {}", e)))?;
            let db = txn.create_db(Some("metadata"), DatabaseFlags::empty())
                .map_err(|e| BlockchainError::Storage(format!("Failed to create metadata database: {}", e)))?;
            txn.commit()
                .map_err(|e| BlockchainError::Storage(format!("Failed to commit transaction: {}", e)))?;
            db
        };

        let transactions_db = {
            let txn = env.begin_rw_txn()
                .map_err(|e| BlockchainError::Storage(format!("Failed to begin transaction: {}", e)))?;
            let db = txn.create_db(Some("transactions"), DatabaseFlags::empty())
                .map_err(|e| BlockchainError::Storage(format!("Failed to create transactions database: {}", e)))?;
            txn.commit()
                .map_err(|e| BlockchainError::Storage(format!("Failed to commit transaction: {}", e)))?;
            db
        };

        Ok(Self {
            env,
            blocks_db,
            metadata_db,
            transactions_db,
        })
    }

    /// Get data by key from blocks database
    pub async fn get(&self, key: &Blake2bHash) -> Result<Option<Vec<u8>>> {
        let env = self.env.clone();
        let db = self.blocks_db;
        let key_bytes = key.as_bytes().to_vec();

        // Run database operation in blocking task to avoid blocking async runtime
        tokio::task::spawn_blocking(move || {
            let txn = env.begin_ro_txn()
                .map_err(|e| BlockchainError::Storage(format!("Failed to begin read transaction: {}", e)))?;

            match txn.get(&db, &key_bytes) {
                Ok(data) => Ok(Some(data.to_vec())),
                Err(MdbxError::NotFound) => Ok(None),
                Err(e) => Err(BlockchainError::Storage(format!("Failed to get data: {}", e))),
            }
        }).await
        .map_err(|e| BlockchainError::Storage(format!("Task join error: {}", e)))?
    }

    /// Put data by key into blocks database
    pub async fn put(&self, key: &Blake2bHash, value: Vec<u8>) -> Result<()> {
        let env = self.env.clone();
        let db = self.blocks_db;
        let key_bytes = key.as_bytes().to_vec();

        tokio::task::spawn_blocking(move || {
            let mut txn = env.begin_rw_txn()
                .map_err(|e| BlockchainError::Storage(format!("Failed to begin write transaction: {}", e)))?;

            txn.put(&db, &key_bytes, &value, WriteFlags::empty())
                .map_err(|e| BlockchainError::Storage(format!("Failed to put data: {}", e)))?;

            txn.commit()
                .map_err(|e| BlockchainError::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok(())
        }).await
        .map_err(|e| BlockchainError::Storage(format!("Task join error: {}", e)))?
    }

    /// Get metadata by string key
    pub async fn get_metadata(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let env = self.env.clone();
        let db = self.metadata_db;
        let key_bytes = key.as_bytes().to_vec();

        tokio::task::spawn_blocking(move || {
            let txn = env.begin_ro_txn()
                .map_err(|e| BlockchainError::Storage(format!("Failed to begin read transaction: {}", e)))?;

            match txn.get(&db, &key_bytes) {
                Ok(data) => Ok(Some(data.to_vec())),
                Err(MdbxError::NotFound) => Ok(None),
                Err(e) => Err(BlockchainError::Storage(format!("Failed to get metadata: {}", e))),
            }
        }).await
        .map_err(|e| BlockchainError::Storage(format!("Task join error: {}", e)))?
    }

    /// Put metadata by string key
    pub async fn put_metadata(&self, key: &str, value: Vec<u8>) -> Result<()> {
        let env = self.env.clone();
        let db = self.metadata_db;
        let key_bytes = key.as_bytes().to_vec();

        tokio::task::spawn_blocking(move || {
            let mut txn = env.begin_rw_txn()
                .map_err(|e| BlockchainError::Storage(format!("Failed to begin write transaction: {}", e)))?;

            txn.put(&db, &key_bytes, &value, WriteFlags::empty())
                .map_err(|e| BlockchainError::Storage(format!("Failed to put metadata: {}", e)))?;

            txn.commit()
                .map_err(|e| BlockchainError::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok(())
        }).await
        .map_err(|e| BlockchainError::Storage(format!("Task join error: {}", e)))?
    }

    /// Delete data by key
    pub async fn delete(&self, key: &Blake2bHash) -> Result<bool> {
        let env = self.env.clone();
        let db = self.blocks_db;
        let key_bytes = key.as_bytes().to_vec();

        tokio::task::spawn_blocking(move || {
            let mut txn = env.begin_rw_txn()
                .map_err(|e| BlockchainError::Storage(format!("Failed to begin write transaction: {}", e)))?;

            match txn.del(&db, &key_bytes, None) {
                Ok(()) => {
                    txn.commit()
                        .map_err(|e| BlockchainError::Storage(format!("Failed to commit transaction: {}", e)))?;
                    Ok(true)
                }
                Err(MdbxError::NotFound) => Ok(false),
                Err(e) => Err(BlockchainError::Storage(format!("Failed to delete data: {}", e))),
            }
        }).await
        .map_err(|e| BlockchainError::Storage(format!("Task join error: {}", e)))?
    }

    /// Get statistics about the database
    pub async fn stats(&self) -> Result<DatabaseStats> {
        let env = self.env.clone();
        let blocks_db = self.blocks_db;
        let metadata_db = self.metadata_db;
        let transactions_db = self.transactions_db;

        tokio::task::spawn_blocking(move || {
            let txn = env.begin_ro_txn()
                .map_err(|e| BlockchainError::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let blocks_stat = txn.db_stat(&blocks_db)
                .map_err(|e| BlockchainError::Storage(format!("Failed to get blocks stats: {}", e)))?;

            let metadata_stat = txn.db_stat(&metadata_db)
                .map_err(|e| BlockchainError::Storage(format!("Failed to get metadata stats: {}", e)))?;

            let transactions_stat = txn.db_stat(&transactions_db)
                .map_err(|e| BlockchainError::Storage(format!("Failed to get transactions stats: {}", e)))?;

            Ok(DatabaseStats {
                blocks_entries: blocks_stat.entries(),
                metadata_entries: metadata_stat.entries(),
                transactions_entries: transactions_stat.entries(),
                page_size: blocks_stat.page_size(),
            })
        }).await
        .map_err(|e| BlockchainError::Storage(format!("Task join error: {}", e)))?
    }

    /// Sync database to disk
    pub async fn sync(&self) -> Result<()> {
        let env = self.env.clone();

        tokio::task::spawn_blocking(move || {
            env.sync(true)
                .map_err(|e| BlockchainError::Storage(format!("Failed to sync database: {}", e)))?;
            Ok(())
        }).await
        .map_err(|e| BlockchainError::Storage(format!("Task join error: {}", e)))?
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub blocks_entries: usize,
    pub metadata_entries: usize,
    pub transactions_entries: usize,
    pub page_size: u32,
}