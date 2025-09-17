// API integration layer between components
use crate::primitives::{Result, Blake2bHash};
use crate::primitives::cdr::{CDRBatch, CDRStatus};
use crate::blockchain::{Block, Transaction};
use crate::smart_contracts::{ContractVM, ContractTransaction, ExecutionResult};
use crate::storage::ChainStore;
use std::sync::Arc;

/// Core API for integrating CDR processing with blockchain and smart contracts
pub struct CDRBlockchainAPI<S: ChainStore> {
    chain_store: Arc<S>,
    contract_vm: Option<ContractVM<crate::smart_contracts::MemoryStorage>>,
}

impl<S: ChainStore> CDRBlockchainAPI<S> {
    pub fn new(chain_store: Arc<S>) -> Self {
        Self {
            chain_store,
            contract_vm: None,
        }
    }

    /// Initialize smart contract integration
    pub fn with_contracts(mut self) -> Self {
        let storage = crate::smart_contracts::MemoryStorage::new();
        self.contract_vm = Some(ContractVM::new(storage));
        self
    }

    /// Process CDR batch through the full pipeline
    pub async fn process_cdr_batch(
        &mut self,
        batch: CDRBatch,
        privacy_proof: Vec<u8>,
        network_signatures: Vec<(String, Vec<u8>)>,
    ) -> Result<Blake2bHash> {
        // 1. Validate CDR batch
        self.validate_cdr_batch(&batch)?;

        // 2. Execute smart contract validation if available
        if let Some(vm) = &mut self.contract_vm {
            let validation_result = Self::execute_cdr_validation_contract(
                vm, &batch, &privacy_proof, &network_signatures
            )?;

            if !validation_result {
                return Err(crate::primitives::BlockchainError::InvalidTransaction(
                    "Smart contract validation failed".to_string()
                ));
            }
        }

        // 3. Create blockchain transaction
        let transaction = Transaction::CDRRecord(crate::blockchain::CDRTransaction {
            batch_id: batch.batch_id,
            home_network: batch.home_network.clone(),
            visited_network: batch.visited_network.clone(),
            record_count: batch.total_records,
            total_charges: batch.total_charges,
            encrypted_data: privacy_proof,
            privacy_proof: vec![], // Separate from encrypted data
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        // 4. Store transaction (would be included in next block)
        let tx_hash = transaction.hash();

        // In a real implementation, this would be added to transaction pool
        // For now, we'll store it directly

        Ok(tx_hash)
    }

    /// Execute settlement between two networks
    pub async fn execute_settlement(
        &mut self,
        creditor_network: String,
        debtor_network: String,
        net_amount: u64,
        currency: String,
        batch_references: Vec<Blake2bHash>,
        settlement_proof: Vec<u8>,
        multi_signature: Vec<u8>,
    ) -> Result<Blake2bHash> {
        // 1. Execute settlement smart contract if available
        if let Some(vm) = &mut self.contract_vm {
            let settlement_result = Self::execute_settlement_contract(
                vm,
                &creditor_network,
                &debtor_network,
                net_amount,
                &settlement_proof,
                &multi_signature
            )?;

            if !settlement_result {
                return Err(crate::primitives::BlockchainError::InvalidTransaction(
                    "Settlement contract execution failed".to_string()
                ));
            }
        }

        // 2. Create settlement transaction
        let settlement_tx = Transaction::Settlement(crate::blockchain::SettlementTransaction {
            settlement_id: crate::primitives::primitives::hash_data(
                &format!("{}:{}:{}", creditor_network, debtor_network, net_amount).as_bytes()
            ),
            creditor_network,
            debtor_network,
            amount: net_amount,
            currency,
            exchange_rate: 100, // 1.00 default
            settlement_proof,
            batch_references,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        let tx_hash = settlement_tx.hash();
        Ok(tx_hash)
    }

    /// Get CDR batch status from blockchain
    pub async fn get_cdr_batch_status(&self, batch_id: &Blake2bHash) -> Result<Option<CDRStatus>> {
        // Query blockchain for CDR batch transaction
        // This would search through blocks for the CDR transaction

        // Placeholder implementation
        Ok(Some(CDRStatus::Pending))
    }

    /// Query settlement history between networks
    pub async fn get_settlement_history(
        &self,
        network1: &str,
        network2: &str,
        from_timestamp: u64,
        to_timestamp: u64,
    ) -> Result<Vec<(Blake2bHash, u64, String)>> {
        // Query blockchain for settlement transactions between networks
        // Returns: (settlement_id, amount, currency)

        // Placeholder implementation
        Ok(vec![])
    }

    // Private helper methods
    fn validate_cdr_batch(&self, batch: &CDRBatch) -> Result<()> {
        // Basic validation
        if batch.home_network.is_empty() || batch.visited_network.is_empty() {
            return Err(crate::primitives::BlockchainError::InvalidTransaction(
                "Network names cannot be empty".to_string()
            ));
        }

        if batch.total_charges == 0 {
            return Err(crate::primitives::BlockchainError::InvalidTransaction(
                "Total charges must be greater than zero".to_string()
            ));
        }

        // Network validation
        crate::primitives::cdr::network::validate_network_id(&batch.home_network)
            .map_err(|_| crate::primitives::BlockchainError::InvalidTransaction(
                format!("Invalid home network: {}", batch.home_network)
            ))?;

        crate::primitives::cdr::network::validate_network_id(&batch.visited_network)
            .map_err(|_| crate::primitives::BlockchainError::InvalidTransaction(
                format!("Invalid visited network: {}", batch.visited_network)
            ))?;

        Ok(())
    }

    fn execute_cdr_validation_contract(
        vm: &mut ContractVM<crate::smart_contracts::MemoryStorage>,
        batch: &CDRBatch,
        privacy_proof: &[u8],
        signatures: &[(String, Vec<u8>)],
    ) -> Result<bool> {
        // Deploy CDR validation contract if not exists
        let contract_addr = crate::primitives::primitives::hash_data(b"cdr_validator");

        let bytecode = crate::smart_contracts::SettlementContractCompiler::compile_cdr_batch_validator();
        vm.deploy_contract(contract_addr, bytecode)?;

        // Prepare input data
        let mut input_data = Vec::new();
        input_data.extend_from_slice(batch.batch_id.as_bytes());
        input_data.extend_from_slice(privacy_proof);

        // Add signatures
        for (network, sig) in signatures {
            input_data.extend_from_slice(network.as_bytes());
            input_data.extend_from_slice(sig);
        }

        // Execute contract
        let context = crate::smart_contracts::ExecutionContext {
            contract_address: contract_addr,
            caller: crate::primitives::primitives::hash_data(batch.home_network.as_bytes()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            gas_limit: 1_000_000,
            gas_used: 0,
            value: 0,
        };

        let result = vm.execute(context, &input_data)?;
        Ok(result.success && result.return_value == Some(1))
    }

    fn execute_settlement_contract(
        vm: &mut ContractVM<crate::smart_contracts::MemoryStorage>,
        creditor: &str,
        debtor: &str,
        amount: u64,
        proof: &[u8],
        signature: &[u8],
    ) -> Result<bool> {
        let contract_addr = crate::primitives::primitives::hash_data(b"settlement_executor");

        let bytecode = crate::smart_contracts::SettlementContractCompiler::compile_settlement_executor();
        vm.deploy_contract(contract_addr, bytecode)?;

        let mut input_data = Vec::new();
        input_data.extend_from_slice(proof);
        input_data.extend_from_slice(signature);
        input_data.extend_from_slice(&amount.to_le_bytes());

        let context = crate::smart_contracts::ExecutionContext {
            contract_address: contract_addr,
            caller: crate::primitives::primitives::hash_data(creditor.as_bytes()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            gas_limit: 2_000_000,
            gas_used: 0,
            value: 0,
        };

        let result = vm.execute(context, &input_data)?;
        Ok(result.success && result.return_value == Some(1))
    }
}

/// High-level API for CDR reconciliation operations
pub struct CDRReconciliationAPI {
    blockchain_api: CDRBlockchainAPI<crate::storage::SimpleChainStore>,
}

impl CDRReconciliationAPI {
    /// Create new CDR reconciliation API with full integration
    pub fn new() -> Self {
        let chain_store = Arc::new(crate::storage::SimpleChainStore::new());
        let blockchain_api = CDRBlockchainAPI::new(chain_store).with_contracts();

        Self {
            blockchain_api,
        }
    }

    /// Submit CDR batch for processing and settlement
    pub async fn submit_cdr_batch(
        &mut self,
        home_network: String,
        visited_network: String,
        period_start: u64,
        period_end: u64,
        total_charges: u64,
        privacy_proof: Vec<u8>,
        network_signatures: Vec<(String, Vec<u8>)>,
    ) -> Result<Blake2bHash> {
        // Create CDR batch
        let mut batch = crate::primitives::cdr::CDRBatch::new(
            home_network,
            visited_network,
            period_start,
            period_end,
        );

        // Add charges
        batch.total_charges = total_charges;
        batch.mark_validated();

        // Process through blockchain
        self.blockchain_api.process_cdr_batch(batch, privacy_proof, network_signatures).await
    }

    /// Execute settlement between networks
    pub async fn execute_settlement(
        &mut self,
        creditor_network: String,
        debtor_network: String,
        net_amount: u64,
        currency: String,
        batch_references: Vec<Blake2bHash>,
        settlement_proof: Vec<u8>,
        multi_signature: Vec<u8>,
    ) -> Result<Blake2bHash> {
        self.blockchain_api.execute_settlement(
            creditor_network,
            debtor_network,
            net_amount,
            currency,
            batch_references,
            settlement_proof,
            multi_signature,
        ).await
    }

    /// Get status of CDR batch
    pub async fn get_batch_status(&self, batch_id: &Blake2bHash) -> Result<Option<CDRStatus>> {
        self.blockchain_api.get_cdr_batch_status(batch_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cdr_batch_processing() {
        let mut api = CDRReconciliationAPI::new();

        let batch_id = api.submit_cdr_batch(
            "T-Mobile-DE".to_string(),
            "Vodafone-UK".to_string(),
            1640995200,
            1641081600,
            125000, // €1,250.00
            b"privacy_proof_data".to_vec(),
            vec![
                ("T-Mobile-DE".to_string(), b"signature1".to_vec()),
                ("Vodafone-UK".to_string(), b"signature2".to_vec()),
            ],
        ).await;

        assert!(batch_id.is_ok());
    }

    #[tokio::test]
    async fn test_settlement_execution() {
        let mut api = CDRReconciliationAPI::new();

        let settlement_id = api.execute_settlement(
            "T-Mobile-DE".to_string(),
            "Vodafone-UK".to_string(),
            85000, // €850.00 net amount
            "EUR".to_string(),
            vec![Blake2bHash::zero()],
            b"settlement_proof".to_vec(),
            b"multi_signature".to_vec(),
        ).await;

        assert!(settlement_id.is_ok());
    }
}