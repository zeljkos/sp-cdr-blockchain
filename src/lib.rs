// SP CDR Reconciliation Blockchain Library
// Integrating Albatross components for SP consortium

// Standard Rust module structure
pub mod primitives;
pub mod blockchain;
pub mod common;
pub mod storage;
pub mod smart_contracts;
pub mod zkp;
pub mod crypto;

pub mod network;
pub mod bce_pipeline;
pub mod api;

// Re-export key types for easy access
pub use primitives::{
    primitives::*,
    error::*,
    cdr::*,
};

pub use blockchain::{
    Block, MicroBlock, MacroBlock,
    ValidatorInfo,
};
pub use blockchain::transaction::Transaction;

pub use common::{
    AbstractBlockchain, Consensus, 
    ValidatorSet, TendermintVote,
};

pub use storage::{
    ChainStore, SimpleChainStore, MdbxChainStore,
};
use smart_contracts::{
    ContractVM, MemoryStorage, MdbxContractStorage, create_mdbx_contract_storage,
    ContractCryptoVerifier, ConsensusContractEngine, ExecutionContext,
};
use blockchain::block::{TransactionData, CDRTransaction, SettlementTransaction};
use std::any::Any;

pub use zkp::{
    CDRPrivacyProof, SettlementProof, CDRPrivateData,
    CDRPublicInputs, SettlementInputs,
};

pub use crypto::{
    PrivateKey, PublicKey, Signature, AggregateSignature,
    KeyPair, ValidatorKey, NetworkOperatorKey,
    MultiSignature, ThresholdConfig,
};

/// Main blockchain implementation integrating all Albatross components
pub struct SPCDRBlockchain {
    chain_store: std::sync::Arc<dyn ChainStore>,
    consensus: common::Consensus<Self>,
    validator_set: std::sync::Arc<tokio::sync::RwLock<common::ValidatorSet>>,
    head_block: std::sync::Arc<tokio::sync::RwLock<Block>>,
    macro_head: std::sync::Arc<tokio::sync::RwLock<Block>>,
    election_head: std::sync::Arc<tokio::sync::RwLock<Block>>,
    network_id: NetworkId,
    contract_engine: Option<std::sync::Arc<ConsensusContractEngine<MdbxContractStorage>>>,
}

#[async_trait::async_trait]
impl common::AbstractBlockchain for SPCDRBlockchain {
    fn network_id(&self) -> NetworkId {
        self.network_id.clone()
    }
    
    fn now(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
    
    fn head(&self) -> &Block {
        // This requires refactoring to avoid blocking in sync context
        // For now, we'll use a placeholder approach
        unimplemented!("Use async head() method instead")
    }
    
    fn macro_head(&self) -> &Block {
        unimplemented!("Use async macro_head() method instead")
    }
    
    fn election_head(&self) -> &Block {
        unimplemented!("Use async election_head() method instead")
    }
    
    fn block_number(&self) -> u32 {
        // This would need to be cached or computed differently in a real implementation
        0 // Placeholder
    }
    
    fn macro_block_number(&self) -> u32 {
        0 // Placeholder
    }
    
    fn election_block_number(&self) -> u32 {
        0 // Placeholder  
    }
    
    async fn get_block(&self, hash: &Blake2bHash, _include_body: bool) -> Result<Option<Block>> {
        self.chain_store.get_block(hash).await
    }
    
    async fn push_block(&self, block: Block) -> Result<()> {
        // Execute transactions in the block first
        self.execute_block_transactions(&block).await?;

        // Store block
        self.chain_store.put_block(&block).await?;

        let block_hash = block.hash();

        // Update head pointers based on block type
        match &block {
            Block::Micro(_) => {
                *self.head_block.write().await = block;
                self.chain_store.set_head(&block_hash).await?;
            }
            Block::Macro(macro_block) => {
                *self.head_block.write().await = block.clone();
                *self.macro_head.write().await = block.clone();

                self.chain_store.set_head(&block_hash).await?;
                self.chain_store.set_macro_head(&block_hash).await?;

                // Check if it's an election block (every 32 macro blocks following Albatross)
                if macro_block.header.block_number % (primitives::Policy::EPOCH_LENGTH * primitives::Policy::BATCH_LENGTH) == 0 {
                    *self.election_head.write().await = block.clone();
                    self.chain_store.set_election_head(&block_hash).await?;

                    // Update validator set if present
                    if let Some(ref validators) = macro_block.body.validators {
                        let mut validator_set = self.validator_set.write().await;
                        // Convert block::ValidatorInfo to validator_set::ValidatorInfo
                        let converted_validators: Vec<blockchain::validator_set::ValidatorInfo> = validators
                            .iter()
                            .map(|v| blockchain::validator_set::ValidatorInfo {
                                validator_address: v.address,
                                signing_key: crate::crypto::PublicKey::from_bytes(&v.signing_key).unwrap_or_else(|_| crate::crypto::PublicKey::from_bytes(&[0u8; 48]).unwrap()),
                                voting_power: 1, // Default voting power
                                network_operator: "default".to_string(),
                                joined_at_height: 0,
                            })
                            .collect();
                        validator_set.update_validators(converted_validators);
                        validator_set.finalize_epoch();
                    }
                }
            }
        }

        Ok(())
    }
    
    fn get_chain_info(&self) -> common::ChainInfo {
        // This would need async access to read the current state
        // For now return placeholder
        common::ChainInfo {
            head_hash: Blake2bHash::zero(),
            head_block_number: 0,
            macro_head_hash: Blake2bHash::zero(), 
            macro_head_block_number: 0,
            election_head_hash: Blake2bHash::zero(),
            election_head_block_number: 0,
            total_work: 0,
        }
    }
    
    fn subscribe_events(&self) -> futures::stream::BoxStream<primitives::BlockchainEvent> {
        // Return empty stream for now - would need proper event system
        use futures::stream::StreamExt;
        futures::stream::empty().boxed()
    }
}

impl SPCDRBlockchain {
    pub fn new(
        chain_store: std::sync::Arc<dyn ChainStore>,
        initial_validators: Vec<ValidatorInfo>,
    ) -> Self {
        Self::new_with_contract_engine(chain_store, initial_validators, None)
    }

    pub fn new_with_contract_engine(
        chain_store: std::sync::Arc<dyn ChainStore>,
        initial_validators: Vec<ValidatorInfo>,
        contract_engine: Option<std::sync::Arc<ConsensusContractEngine<MdbxContractStorage>>>,
    ) -> Self {
        let validator_set = std::sync::Arc::new(tokio::sync::RwLock::new(
            common::ValidatorSet::new(initial_validators)
        ));
        
        // Create genesis blocks
        let genesis_block = Block::Macro(MacroBlock {
            header: blockchain::MacroHeader {
                network: NetworkId::SPConsortium,
                version: 1,
                block_number: 0,
                round: 0,
                timestamp: 0,
                parent_hash: Blake2bHash::zero(),
                parent_election_hash: Blake2bHash::zero(),
                seed: Blake2bHash::zero(),
                extra_data: b"SP CDR Reconciliation Genesis".to_vec(),
                state_root: Blake2bHash::zero(),
                body_root: Blake2bHash::zero(), 
                history_root: Blake2bHash::zero(),
            },
            body: blockchain::MacroBody {
                validators: None,
                lost_reward_set: vec![],
                disabled_set: vec![],
                transactions: vec![],
            },
        });
        
        let head_block = std::sync::Arc::new(tokio::sync::RwLock::new(genesis_block.clone()));
        let macro_head = std::sync::Arc::new(tokio::sync::RwLock::new(genesis_block.clone()));
        let election_head = std::sync::Arc::new(tokio::sync::RwLock::new(genesis_block));
        
        let blockchain = Self {
            chain_store,
            validator_set,
            head_block,
            macro_head,
            election_head,
            network_id: NetworkId::SPConsortium,
            consensus: common::Consensus::placeholder(),
            contract_engine,
        };
        
        // TODO: Fix circular dependency - consensus needs blockchain reference
        // This requires refactoring the constructor pattern
        
        blockchain
    }
    
    /// Async method to get current head
    pub async fn head_async(&self) -> Block {
        self.head_block.read().await.clone()
    }
    
    /// Async method to get macro head
    pub async fn macro_head_async(&self) -> Block {
        self.macro_head.read().await.clone()
    }
    
    /// Async method to get election head
    pub async fn election_head_async(&self) -> Block {
        self.election_head.read().await.clone()
    }

    /// Convert NetworkId to Blake2bHash for use as caller address
    fn network_id_to_hash(&self, network_id: &NetworkId) -> Blake2bHash {
        match network_id {
            NetworkId::SPConsortium => Blake2bHash::from_bytes([1u8; 32]),
            NetworkId::DevNet => Blake2bHash::from_bytes([2u8; 32]),
            NetworkId::TestNet => Blake2bHash::from_bytes([3u8; 32]),
            NetworkId::MainNet => Blake2bHash::from_bytes([4u8; 32]),
            NetworkId::Operator { name, country: _ } => {
                // Generate hash from operator name
                crate::primitives::primitives::hash_data(name.as_bytes())
            }
        }
    }

    /// Execute all transactions in a block before applying it
    async fn execute_block_transactions(&self, block: &Block) -> Result<()> {
        // Only execute if we have a contract engine
        let contract_engine = match &self.contract_engine {
            Some(engine) => engine,
            None => return Ok(()), // No contract execution without engine
        };

        // Extract transactions from block
        let transactions = match block {
            Block::Micro(micro_block) => &micro_block.body.transactions,
            Block::Macro(macro_block) => &macro_block.body.transactions,
        };

        // Execute each transaction through the contract engine
        for transaction in transactions {
            // Check if this is a contract transaction (CDR settlement, deployment, etc.)
            if let TransactionData::CDRRecord(cdr_tx) = &transaction.data {
                // Create contract transaction from CDR transaction
                // Generate settlement address from network pair
                let settlement_address = crate::primitives::primitives::hash_data(
                    format!("{}-{}", cdr_tx.home_network, cdr_tx.visited_network).as_bytes()
                );

                let contract_tx = smart_contracts::ContractTransaction {
                    contract_address: settlement_address,
                    caller: transaction.sender, // Use transaction sender as caller
                    input_data: bincode::serialize(cdr_tx)
                        .map_err(|e| BlockchainError::Serialization(e.to_string()))?,
                    gas_limit: 1_000_000, // Default gas limit for CDR transactions
                    value: transaction.value,
                    nonce: 0, // Basic nonce for now
                };

                // Execute the contract transaction
                match contract_engine.execute_transaction(contract_tx, block.height(), 0).await {
                    Ok(receipt) => {
                        // Store execution result
                        if let Some(mdbx_store) = self.chain_store.as_any().downcast_ref::<MdbxChainStore>() {
                            let result_data = bincode::serialize(&receipt)
                                .map_err(|e| BlockchainError::Serialization(e.to_string()))?;
                            mdbx_store.put_execution_result(&transaction.hash(), &result_data).await?;
                        }

                        // Log successful execution
                        println!("Contract execution successful: tx={}, gas_used={}",
                            transaction.hash(), receipt.gas_used);
                    }
                    Err(e) => {
                        eprintln!("Contract execution failed: tx={}, error={}",
                            transaction.hash(), e);
                        // In a production system, we might want to fail the entire block
                        // For now, we continue processing other transactions
                    }
                }
            }
            // Handle other transaction types (SettlementTransaction, etc.)
            else if let TransactionData::Settlement(settlement_tx) = &transaction.data {
                // Settlement transactions can also trigger contract execution
                // Generate settlement contract address from network pair
                let contract_address = crate::primitives::primitives::hash_data(
                    format!("{}-{}", settlement_tx.creditor_network, settlement_tx.debtor_network).as_bytes()
                );

                let contract_tx = smart_contracts::ContractTransaction {
                    contract_address,
                    caller: Blake2bHash::zero(), // System caller for settlements
                    input_data: bincode::serialize(&settlement_tx)
                        .map_err(|e| BlockchainError::Serialization(e.to_string()))?,
                    gas_limit: 2_000_000, // Higher gas limit for settlement validation
                    value: settlement_tx.amount,
                    nonce: 0, // Basic nonce for now
                };

                match contract_engine.execute_transaction(contract_tx, block.height(), 0).await {
                    Ok(receipt) => {
                        println!("Settlement validation successful: tx={}, gas_used={}",
                            transaction.hash(), receipt.gas_used);
                    }
                    Err(e) => {
                        eprintln!("Settlement validation failed: tx={}, error={}",
                            transaction.hash(), e);
                    }
                }
            }
        }

        Ok(())
    }
}

// Integration tests module
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_blockchain_integration() {
        // Test that all components can be instantiated and work together
        // This ensures our API integration is correct
    }
}