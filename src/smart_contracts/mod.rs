// Smart contracts module for SP CDR reconciliation blockchain
pub mod settlement;
pub mod vm;
pub mod crypto_verifier;
pub mod consensus_integration;
pub mod settlement_contract;
pub mod mdbx_storage;  // Non-breaking addition

// Legacy settlement data structures (keeping for compatibility)
pub use settlement::{
    SettlementContract,
    CDRBatchContract,
    SettlementExecutionContract,
    SettlementEngine,
    SettlementPeriod,
    SettlementStatus
};

// Real smart contract components
pub use vm::{ContractVM, ExecutionContext, ExecutionResult, Instruction, ContractStorage, MemoryStorage};
pub use crypto_verifier::{ZKProofVerifier, BLSVerifier, ContractCryptoVerifier, SettlementProofInputs, CDRPrivacyInputs};
pub use consensus_integration::{ConsensusContractEngine, ContractTransaction, ContractDeployment, ContractReceipt};
pub use settlement_contract::{ExecutableSettlementContract, SettlementContractCompiler, SettlementContractFactory};
pub use mdbx_storage::{MdbxContractStorage, create_mdbx_contract_storage};  // Non-breaking addition

use serde::{Deserialize, Serialize};
use crate::primitives::{Blake2bHash, NetworkId};

/// Smart contract error types
#[derive(Debug, thiserror::Error)]
pub enum SmartContractError {
    #[error("Contract not found: {0}")]
    ContractNotFound(Blake2bHash),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    #[error("Invalid contract code")]
    InvalidCode,
}

/// Smart contract state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartContract {
    pub contract_id: Blake2bHash,
    pub code_hash: Blake2bHash,
    pub creator: Blake2bHash,
    pub created_at: u64,
    pub network: NetworkId,
}

/// Smart contract execution environment (placeholder)
pub struct ContractExecutor {
    network_id: NetworkId,
}

impl ContractExecutor {
    pub fn new(network_id: NetworkId) -> Self {
        Self { network_id }
    }

    /// Execute contract (placeholder implementation)
    pub fn execute(&self, _contract: &SmartContract, _input: &[u8]) -> std::result::Result<Vec<u8>, SmartContractError> {
        // Placeholder - in a real implementation this would:
        // 1. Load contract bytecode
        // 2. Set up execution environment
        // 3. Execute contract with input
        // 4. Return result
        
        Ok(b"contract_execution_result".to_vec())
    }
}

pub type Result<T> = std::result::Result<T, SmartContractError>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::primitives::hash_data;

    #[test]
    fn test_smart_contract_creation() {
        let contract = SmartContract {
            contract_id: hash_data(b"contract_1"),
            code_hash: hash_data(b"contract_code"),
            creator: hash_data(b"creator_address"),
            created_at: 1640995200,
            network: NetworkId::SPConsortium,
        };

        assert_eq!(contract.network, NetworkId::SPConsortium);
        assert_ne!(contract.contract_id, Blake2bHash::zero());
    }

    #[test]
    fn test_contract_executor() {
        let executor = ContractExecutor::new(NetworkId::SPConsortium);
        
        let contract = SmartContract {
            contract_id: hash_data(b"test_contract"),
            code_hash: hash_data(b"test_code"),
            creator: hash_data(b"test_creator"),
            created_at: 1640995200,
            network: NetworkId::SPConsortium,
        };

        let result = executor.execute(&contract, b"test_input");
        assert!(result.is_ok());
    }
}