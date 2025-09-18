// MDBX-based smart contract storage (non-breaking addition)
use std::sync::Arc;
use crate::primitives::{Blake2bHash, Result};
use crate::storage::MdbxChainStore;
use crate::smart_contracts::vm::{ContractStorage, Instruction};

/// MDBX-backed contract storage implementation
/// This is an ADDITION to MemoryStorage, not a replacement
pub struct MdbxContractStorage {
    mdbx_store: Arc<MdbxChainStore>,
}

impl MdbxContractStorage {
    pub fn new(mdbx_store: Arc<MdbxChainStore>) -> Self {
        Self { mdbx_store }
    }
}

impl ContractStorage for MdbxContractStorage {
    fn get(&self, contract: &Blake2bHash, key: &Blake2bHash) -> Result<Option<Vec<u8>>> {
        // Use tokio runtime to handle async in sync context
        let rt = tokio::runtime::Handle::current();
        rt.block_on(self.mdbx_store.get_contract_state(contract, key))
    }

    fn set(&mut self, contract: &Blake2bHash, key: &Blake2bHash, value: Vec<u8>) -> Result<()> {
        // Use tokio runtime to handle async in sync context
        let rt = tokio::runtime::Handle::current();
        rt.block_on(self.mdbx_store.put_contract_state(contract, key, &value))
    }

    fn get_code(&self, contract: &Blake2bHash) -> Result<Option<Vec<Instruction>>> {
        // Get bytecode from MDBX
        let rt = tokio::runtime::Handle::current();
        let bytecode_opt = rt.block_on(self.mdbx_store.get_contract_code(contract))?;

        match bytecode_opt {
            Some(bytecode) => {
                // Deserialize bytecode to instructions
                let instructions: Vec<Instruction> = bincode::deserialize(&bytecode)
                    .map_err(|e| crate::primitives::BlockchainError::Serialization(
                        format!("Failed to deserialize contract bytecode: {}", e)
                    ))?;
                Ok(Some(instructions))
            }
            None => Ok(None),
        }
    }

    fn set_code(&mut self, contract: &Blake2bHash, code: Vec<Instruction>) -> Result<()> {
        // Serialize instructions to bytecode
        let bytecode = bincode::serialize(&code)
            .map_err(|e| crate::primitives::BlockchainError::Serialization(
                format!("Failed to serialize contract bytecode: {}", e)
            ))?;

        // Store in MDBX
        let rt = tokio::runtime::Handle::current();
        rt.block_on(self.mdbx_store.put_contract_code(contract, &bytecode))
    }
}

/// Factory function to create MDBX storage for contracts
/// This allows easy switching between MemoryStorage and MdbxContractStorage
pub fn create_mdbx_contract_storage(mdbx_store: Arc<MdbxChainStore>) -> MdbxContractStorage {
    MdbxContractStorage::new(mdbx_store)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MdbxChainStore;
    use crate::smart_contracts::vm::Instruction;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_mdbx_contract_storage() {
        // Create temporary MDBX store
        let temp_dir = TempDir::new().unwrap();
        let store = Arc::new(MdbxChainStore::new(temp_dir.path()).unwrap());

        // Create MDBX contract storage
        let mut contract_storage = MdbxContractStorage::new(store);

        // Test contract address and key
        let contract_addr = Blake2bHash::from_bytes([1; 32]);
        let state_key = Blake2bHash::from_bytes([2; 32]);
        let test_value = b"test_contract_state".to_vec();

        // Test state storage
        contract_storage.set(&contract_addr, &state_key, test_value.clone()).unwrap();
        let retrieved_value = contract_storage.get(&contract_addr, &state_key).unwrap();
        assert_eq!(retrieved_value, Some(test_value));

        // Test contract code storage
        let test_code = vec![
            Instruction::Push(42),
            Instruction::Push(100),
            Instruction::Add,
            Instruction::Halt,
        ];

        contract_storage.set_code(&contract_addr, test_code.clone()).unwrap();
        let retrieved_code = contract_storage.get_code(&contract_addr).unwrap();
        assert_eq!(retrieved_code, Some(test_code));

        println!("✅ MDBX contract storage tests passed");
    }
}