// Production Sled database wrapper for blockchain storage
use crate::primitives::{Blake2bHash, Result, BlockchainError};
use sled::{Db, Tree};
use std::path::Path;
use std::sync::Arc;

/// Production Sled storage with real persistence
pub struct SledStore {
    db: Arc<Db>,
    blocks_tree: Tree,
    metadata_tree: Tree,
    transactions_tree: Tree,
}

impl SledStore {
    pub fn new(path: &str) -> Result<Self> {
        // Create the database directory if it doesn't exist
        std::fs::create_dir_all(path)
            .map_err(|e| BlockchainError::Storage(format!("Failed to create directory: {}", e)))?;

        // Initialize Sled database
        let db = sled::open(Path::new(path))
            .map_err(|e| BlockchainError::Storage(format!("Failed to open Sled database: {}", e)))?;

        let db = Arc::new(db);

        // Create trees for different data types
        let blocks_tree = db.open_tree("blocks")
            .map_err(|e| BlockchainError::Storage(format!("Failed to open blocks tree: {}", e)))?;

        let metadata_tree = db.open_tree("metadata")
            .map_err(|e| BlockchainError::Storage(format!("Failed to open metadata tree: {}", e)))?;

        let transactions_tree = db.open_tree("transactions")
            .map_err(|e| BlockchainError::Storage(format!("Failed to open transactions tree: {}", e)))?;

        Ok(Self {
            db,
            blocks_tree,
            metadata_tree,
            transactions_tree,
        })
    }

    /// Get data by key from blocks tree
    pub async fn get(&self, key: &Blake2bHash) -> Result<Option<Vec<u8>>> {
        let blocks_tree = self.blocks_tree.clone();
        let key_bytes = key.as_bytes().to_vec();

        // Run database operation in blocking task to avoid blocking async runtime
        tokio::task::spawn_blocking(move || {
            match blocks_tree.get(&key_bytes) {
                Ok(Some(data)) => Ok(Some(data.to_vec())),
                Ok(None) => Ok(None),
                Err(e) => Err(BlockchainError::Storage(format!("Failed to get data: {}", e))),
            }
        }).await
        .map_err(|e| BlockchainError::Storage(format!("Task join error: {}", e)))?
    }

    /// Put data by key into blocks tree
    pub async fn put(&self, key: &Blake2bHash, value: Vec<u8>) -> Result<()> {
        let blocks_tree = self.blocks_tree.clone();
        let key_bytes = key.as_bytes().to_vec();

        tokio::task::spawn_blocking(move || {
            blocks_tree.insert(&key_bytes, value)
                .map_err(|e| BlockchainError::Storage(format!("Failed to put data: {}", e)))?;

            Ok(())
        }).await
        .map_err(|e| BlockchainError::Storage(format!("Task join error: {}", e)))?
    }

    /// Get metadata by string key
    pub async fn get_metadata(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let metadata_tree = self.metadata_tree.clone();
        let key_bytes = key.as_bytes().to_vec();

        tokio::task::spawn_blocking(move || {
            match metadata_tree.get(&key_bytes) {
                Ok(Some(data)) => Ok(Some(data.to_vec())),
                Ok(None) => Ok(None),
                Err(e) => Err(BlockchainError::Storage(format!("Failed to get metadata: {}", e))),
            }
        }).await
        .map_err(|e| BlockchainError::Storage(format!("Task join error: {}", e)))?
    }

    /// Put metadata by string key
    pub async fn put_metadata(&self, key: &str, value: Vec<u8>) -> Result<()> {
        let metadata_tree = self.metadata_tree.clone();
        let key_bytes = key.as_bytes().to_vec();

        tokio::task::spawn_blocking(move || {
            metadata_tree.insert(&key_bytes, value)
                .map_err(|e| BlockchainError::Storage(format!("Failed to put metadata: {}", e)))?;

            Ok(())
        }).await
        .map_err(|e| BlockchainError::Storage(format!("Task join error: {}", e)))?
    }

    /// Delete data by key
    pub async fn delete(&self, key: &Blake2bHash) -> Result<bool> {
        let blocks_tree = self.blocks_tree.clone();
        let key_bytes = key.as_bytes().to_vec();

        tokio::task::spawn_blocking(move || {
            match blocks_tree.remove(&key_bytes) {
                Ok(Some(_)) => Ok(true),
                Ok(None) => Ok(false),
                Err(e) => Err(BlockchainError::Storage(format!("Failed to delete data: {}", e))),
            }
        }).await
        .map_err(|e| BlockchainError::Storage(format!("Task join error: {}", e)))?
    }

    /// Get statistics about the database
    pub async fn stats(&self) -> Result<DatabaseStats> {
        let blocks_tree = self.blocks_tree.clone();
        let metadata_tree = self.metadata_tree.clone();
        let transactions_tree = self.transactions_tree.clone();

        tokio::task::spawn_blocking(move || {
            let blocks_entries = blocks_tree.len();
            let metadata_entries = metadata_tree.len();
            let transactions_entries = transactions_tree.len();

            Ok(DatabaseStats {
                blocks_entries,
                metadata_entries,
                transactions_entries,
                page_size: 4096, // Sled default page size
            })
        }).await
        .map_err(|e| BlockchainError::Storage(format!("Task join error: {}", e)))?
    }

    /// Sync database to disk
    pub async fn sync(&self) -> Result<()> {
        let db = self.db.clone();

        tokio::task::spawn_blocking(move || {
            db.flush()
                .map_err(|e| BlockchainError::Storage(format!("Failed to sync database: {}", e)))?;
            Ok(())
        }).await
        .map_err(|e| BlockchainError::Storage(format!("Task join error: {}", e)))?
    }

    /// Get raw database reference for advanced operations
    pub fn raw_db(&self) -> &Db {
        &self.db
    }

    /// Get blocks tree for batch operations
    pub fn blocks_tree(&self) -> &Tree {
        &self.blocks_tree
    }

    /// Get metadata tree for batch operations
    pub fn metadata_tree(&self) -> &Tree {
        &self.metadata_tree
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sled_store_basic_operations() {
        let temp_dir = "/tmp/test_sled_store";
        let _ = std::fs::remove_dir_all(temp_dir);

        let store = SledStore::new(temp_dir).unwrap();

        // Test basic put/get
        let key = Blake2bHash::from_data(b"test_key");
        let value = b"test_value".to_vec();

        store.put(&key, value.clone()).await.unwrap();
        let retrieved = store.get(&key).await.unwrap();
        assert_eq!(retrieved, Some(value));

        // Test metadata operations
        store.put_metadata("head", b"head_hash".to_vec()).await.unwrap();
        let head = store.get_metadata("head").await.unwrap();
        assert_eq!(head, Some(b"head_hash".to_vec()));

        // Test deletion
        let deleted = store.delete(&key).await.unwrap();
        assert!(deleted);

        let after_delete = store.get(&key).await.unwrap();
        assert_eq!(after_delete, None);

        // Test stats
        let stats = store.stats().await.unwrap();
        assert_eq!(stats.metadata_entries, 1); // head metadata

        // Test sync
        store.sync().await.unwrap();

        // Cleanup
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[tokio::test]
    async fn test_sled_store_persistence() {
        let temp_dir = "/tmp/test_sled_persistence";
        let _ = std::fs::remove_dir_all(temp_dir);

        let test_key = Blake2bHash::from_data(b"persistence_test");
        let test_value = b"persistent_value".to_vec();

        // Store data
        {
            let store = SledStore::new(temp_dir).unwrap();
            store.put(&test_key, test_value.clone()).await.unwrap();
            store.sync().await.unwrap();
        } // store goes out of scope, database should persist

        // Retrieve data from new instance
        {
            let store2 = SledStore::new(temp_dir).unwrap();
            let retrieved = store2.get(&test_key).await.unwrap();
            assert_eq!(retrieved, Some(test_value));
        }

        // Cleanup
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}