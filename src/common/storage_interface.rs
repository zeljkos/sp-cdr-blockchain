// Storage interface abstraction
use crate::primitives::{Blake2bHash, Result};

#[async_trait::async_trait]
pub trait StorageInterface: Send + Sync {
    async fn get(&self, key: &Blake2bHash) -> Result<Option<Vec<u8>>>;
    async fn put(&self, key: &Blake2bHash, value: Vec<u8>) -> Result<()>;
    async fn delete(&self, key: &Blake2bHash) -> Result<()>;
}