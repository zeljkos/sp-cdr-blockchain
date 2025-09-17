// Real MDBX chain store extracted from Albatross
use std::sync::Arc;
use libmdbx::{Environment, Database, Transaction as MdbxTransaction, WriteFlags, Mode};
use crate::primitives::{Result, BlockchainError, Blake2bHash};
use crate::blockchain::Block;
use super::ChainStore;

/// Real MDBX chain store implementation from Albatross
pub struct MdbxChainStore {
    env: Environment,
    blocks_db: Database,
    chain_db: Database,
    macro_db: Database,
}

impl MdbxChainStore {
    /// Create new MDBX chain store
    pub fn new(path: &str) -> Result<Self> {
        // Create MDBX environment with Albatross settings
        let env = Environment::new()
            .set_max_dbs(16)
            .open(path)
            .map_err(|e| BlockchainError::Storage(format!("MDBX environment error: {}", e)))?;

        // Create databases
        let txn = env.begin_rw_txn()
            .map_err(|e| BlockchainError::Storage(format!("Transaction begin error: {}", e)))?;

        let blocks_db = env.create_db(Some("blocks"))
            .map_err(|e| BlockchainError::Storage(format!("Blocks DB creation error: {}", e)))?;

        let chain_db = env.create_db(Some("chain"))
            .map_err(|e| BlockchainError::Storage(format!("Chain DB creation error: {}", e)))?;

        let macro_db = env.create_db(Some("macro"))
            .map_err(|e| BlockchainError::Storage(format!("Macro DB creation error: {}", e)))?;

        txn.commit()
            .map_err(|e| BlockchainError::Storage(format!("Transaction commit error: {}", e)))?;

        Ok(Self {
            env,
            blocks_db,
            chain_db,
            macro_db,
        })
    }

    /// Get block by hash from MDBX
    fn get_block_raw(&self, hash: &Blake2bHash) -> Result<Option<Vec<u8>>> {
        let txn = self.env.begin_ro_txn()
            .map_err(|e| BlockchainError::Storage(format!("Read transaction error: {}", e)))?;

        let key = hash.as_bytes();
        match txn.get(&self.blocks_db, key) {
            Ok(data) => Ok(Some(data.to_vec())),
            Err(libmdbx::Error::NotFound) => Ok(None),
            Err(e) => Err(BlockchainError::Storage(format!("MDBX read error: {}", e))),
        }
    }

    /// Put block into MDBX
    fn put_block_raw(&self, hash: &Blake2bHash, data: &[u8]) -> Result<()> {
        let txn = self.env.begin_rw_txn()
            .map_err(|e| BlockchainError::Storage(format!("Write transaction error: {}", e)))?;

        let key = hash.as_bytes();
        txn.put(&self.blocks_db, key, data, WriteFlags::default())
            .map_err(|e| BlockchainError::Storage(format!("MDBX write error: {}", e)))?;

        txn.commit()
            .map_err(|e| BlockchainError::Storage(format!("Transaction commit error: {}", e)))?;

        Ok(())
    }

    /// Get chain metadata
    fn get_chain_data(&self, key: &str) -> Result<Option<Blake2bHash>> {
        let txn = self.env.begin_ro_txn()
            .map_err(|e| BlockchainError::Storage(format!("Read transaction error: {}", e)))?;

        match txn.get(&self.chain_db, key.as_bytes()) {
            Ok(data) => {
                if data.len() == 32 {
                    let mut hash_bytes = [0u8; 32];
                    hash_bytes.copy_from_slice(&data);
                    Ok(Some(Blake2bHash::from_bytes(hash_bytes)))
                } else {
                    Err(BlockchainError::Storage("Invalid hash length".to_string()))
                }
            },
            Err(libmdbx::Error::NotFound) => Ok(None),
            Err(e) => Err(BlockchainError::Storage(format!("MDBX read error: {}", e))),
        }
    }

    /// Set chain metadata
    fn set_chain_data(&self, key: &str, hash: &Blake2bHash) -> Result<()> {
        let txn = self.env.begin_rw_txn()
            .map_err(|e| BlockchainError::Storage(format!("Write transaction error: {}", e)))?;

        txn.put(&self.chain_db, key.as_bytes(), hash.as_bytes(), WriteFlags::default())
            .map_err(|e| BlockchainError::Storage(format!("MDBX write error: {}", e)))?;

        txn.commit()
            .map_err(|e| BlockchainError::Storage(format!("Transaction commit error: {}", e)))?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl ChainStore for MdbxChainStore {
    async fn get_block(&self, hash: &Blake2bHash) -> Result<Option<Block>> {
        let data = self.get_block_raw(hash)?;
        match data {
            Some(bytes) => {
                // Deserialize block from bytes
                let block: Block = serde_json::from_slice(&bytes)
                    .map_err(|e| BlockchainError::Storage(format!("Block deserialization error: {}", e)))?;
                Ok(Some(block))
            },
            None => Ok(None),
        }
    }

    async fn get_block_at(&self, _block_number: u32) -> Result<Option<Block>> {
        // Would need block number index for full implementation
        // For now, return None
        Ok(None)
    }

    async fn put_block(&self, block: &Block) -> Result<()> {
        // Serialize block to bytes
        let data = serde_json::to_vec(block)
            .map_err(|e| BlockchainError::Storage(format!("Block serialization error: {}", e)))?;

        let hash = block.hash();
        self.put_block_raw(&hash, &data)?;

        Ok(())
    }

    async fn get_head_hash(&self) -> Result<Blake2bHash> {
        self.get_chain_data("head")?.ok_or_else(||
            BlockchainError::Storage("No head hash found".to_string())
        )
    }

    async fn set_head(&self, hash: &Blake2bHash) -> Result<()> {
        self.set_chain_data("head", hash)
    }

    async fn get_macro_head_hash(&self) -> Result<Blake2bHash> {
        self.get_chain_data("macro_head")?.ok_or_else(||
            BlockchainError::Storage("No macro head hash found".to_string())
        )
    }

    async fn set_macro_head(&self, hash: &Blake2bHash) -> Result<()> {
        self.set_chain_data("macro_head", hash)
    }

    async fn get_election_head_hash(&self) -> Result<Blake2bHash> {
        self.get_chain_data("election_head")?.ok_or_else(||
            BlockchainError::Storage("No election head hash found".to_string())
        )
    }

    async fn set_election_head(&self, hash: &Blake2bHash) -> Result<()> {
        self.set_chain_data("election_head", hash)
    }
}

/// MDBX-based storage for contract state
pub struct MdbxContractStorage {
    env: Environment,
    contracts_db: Database,
    state_db: Database,
}

impl MdbxContractStorage {
    pub fn new(path: &str) -> Result<Self> {
        let env = Environment::new()
            .set_max_dbs(8)
            .open(path)
            .map_err(|e| BlockchainError::Storage(format!("Contract storage error: {}", e)))?;

        let txn = env.begin_rw_txn()
            .map_err(|e| BlockchainError::Storage(format!("Transaction begin error: {}", e)))?;

        let contracts_db = env.create_db(Some("contracts"))
            .map_err(|e| BlockchainError::Storage(format!("Contracts DB error: {}", e)))?;

        let state_db = env.create_db(Some("state"))
            .map_err(|e| BlockchainError::Storage(format!("State DB error: {}", e)))?;

        txn.commit()
            .map_err(|e| BlockchainError::Storage(format!("Transaction commit error: {}", e)))?;

        Ok(Self {
            env,
            contracts_db,
            state_db,
        })
    }
}

impl crate::smart_contracts::ContractStorage for MdbxContractStorage {
    fn get(&self, contract: &Blake2bHash, key: &Blake2bHash) -> Result<Option<Vec<u8>>> {
        let txn = self.env.begin_ro_txn()
            .map_err(|e| BlockchainError::Storage(format!("Read transaction error: {}", e)))?;

        // Create composite key: contract_address + state_key
        let mut composite_key = Vec::new();
        composite_key.extend_from_slice(contract.as_bytes());
        composite_key.extend_from_slice(key.as_bytes());

        match txn.get(&self.state_db, &composite_key) {
            Ok(data) => Ok(Some(data.to_vec())),
            Err(libmdbx::Error::NotFound) => Ok(None),
            Err(e) => Err(BlockchainError::Storage(format!("MDBX read error: {}", e))),
        }
    }

    fn set(&mut self, contract: &Blake2bHash, key: &Blake2bHash, value: Vec<u8>) -> Result<()> {
        let txn = self.env.begin_rw_txn()
            .map_err(|e| BlockchainError::Storage(format!("Write transaction error: {}", e)))?;

        let mut composite_key = Vec::new();
        composite_key.extend_from_slice(contract.as_bytes());
        composite_key.extend_from_slice(key.as_bytes());

        txn.put(&self.state_db, &composite_key, &value, WriteFlags::default())
            .map_err(|e| BlockchainError::Storage(format!("MDBX write error: {}", e)))?;

        txn.commit()
            .map_err(|e| BlockchainError::Storage(format!("Transaction commit error: {}", e)))?;

        Ok(())
    }

    fn get_code(&self, contract: &Blake2bHash) -> Result<Option<Vec<crate::smart_contracts::Instruction>>> {
        let txn = self.env.begin_ro_txn()
            .map_err(|e| BlockchainError::Storage(format!("Read transaction error: {}", e)))?;

        match txn.get(&self.contracts_db, contract.as_bytes()) {
            Ok(data) => {
                let instructions: Vec<crate::smart_contracts::Instruction> = serde_json::from_slice(&data)
                    .map_err(|e| BlockchainError::Storage(format!("Code deserialization error: {}", e)))?;
                Ok(Some(instructions))
            },
            Err(libmdbx::Error::NotFound) => Ok(None),
            Err(e) => Err(BlockchainError::Storage(format!("MDBX read error: {}", e))),
        }
    }

    fn set_code(&mut self, contract: &Blake2bHash, code: Vec<crate::smart_contracts::Instruction>) -> Result<()> {
        let txn = self.env.begin_rw_txn()
            .map_err(|e| BlockchainError::Storage(format!("Write transaction error: {}", e)))?;

        let data = serde_json::to_vec(&code)
            .map_err(|e| BlockchainError::Storage(format!("Code serialization error: {}", e)))?;

        txn.put(&self.contracts_db, contract.as_bytes(), &data, WriteFlags::default())
            .map_err(|e| BlockchainError::Storage(format!("MDBX write error: {}", e)))?;

        txn.commit()
            .map_err(|e| BlockchainError::Storage(format!("Transaction commit error: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn test_mdbx_chain_store() {
        let temp_dir = "/tmp/test_mdbx_chain";
        let _ = fs::remove_dir_all(temp_dir);
        fs::create_dir_all(temp_dir).unwrap();

        let store = MdbxChainStore::new(temp_dir).unwrap();

        // Test head operations
        let test_hash = crate::primitives::primitives::hash_data(b"test_block");
        store.set_head(&test_hash).await.unwrap();
        let retrieved_hash = store.get_head_hash().await.unwrap();
        assert_eq!(test_hash, retrieved_hash);

        // Cleanup
        let _ = fs::remove_dir_all(temp_dir);
    }
}