// Consensus module following Albatross patterns and integrations
use futures::stream::BoxStream;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use crate::primitives::{Result, BlockchainError, ConsensusEvent, Blake2bHash};
use crate::blockchain::Block;
use crate::blockchain::validator_set::ValidatorInfo;

/// Abstract blockchain interface following Albatross AbstractBlockchain trait
#[async_trait::async_trait]
pub trait AbstractBlockchain: Send + Sync {
    /// Returns the network id
    fn network_id(&self) -> crate::primitives::NetworkId;
    
    /// Returns the current time
    fn now(&self) -> u64;
    
    /// Returns the head of the main chain
    fn head(&self) -> &Block;
    
    /// Returns the last macro block
    fn macro_head(&self) -> &Block;
    
    /// Returns the last election macro block  
    fn election_head(&self) -> &Block;
    
    /// Returns block number at head
    fn block_number(&self) -> u32;
    
    /// Returns macro block number
    fn macro_block_number(&self) -> u32;
    
    /// Returns election block number
    fn election_block_number(&self) -> u32;
    
    /// Get block by hash
    async fn get_block(&self, hash: &Blake2bHash, include_body: bool) -> Result<Option<Block>>;
    
    /// Push block to blockchain
    async fn push_block(&self, block: Block) -> Result<()>;
    
    /// Get chain info
    fn get_chain_info(&self) -> ChainInfo;
    
    /// Subscribe to blockchain events
    fn subscribe_events(&self) -> BoxStream<crate::primitives::BlockchainEvent>;
}

/// Chain information
#[derive(Debug, Clone)]
pub struct ChainInfo {
    pub head_hash: Blake2bHash,
    pub head_block_number: u32,
    pub macro_head_hash: Blake2bHash,
    pub macro_head_block_number: u32,
    pub election_head_hash: Blake2bHash,
    pub election_head_block_number: u32,
    pub total_work: u64,
}

/// Consensus manager following Albatross Consensus pattern
pub struct Consensus<B: AbstractBlockchain> {
    blockchain: Arc<B>,
    established: Arc<RwLock<bool>>,
    events: broadcast::Sender<ConsensusEvent>,
}

impl<B: AbstractBlockchain> Consensus<B> {
    pub fn new(blockchain: Arc<B>) -> Self {
        let (events, _) = broadcast::channel::<ConsensusEvent>(256);

        Self {
            blockchain,
            established: Arc::new(RwLock::new(false)),
            events,
        }
    }

    pub fn placeholder() -> Self {
        let (events, _) = broadcast::channel::<ConsensusEvent>(256);

        // This is a temporary placeholder to avoid circular dependencies
        // In production, this would be properly initialized
        todo!("Consensus placeholder not implemented for production use")
    }
    
    /// Check if consensus is established
    pub async fn is_established(&self) -> bool {
        *self.established.read().await
    }
    
    /// Force establish consensus (for testing)
    pub async fn force_established(&self) {
        *self.established.write().await = true;
        let _ = self.events.send(ConsensusEvent::Established { synced_validity_window: true });
    }
    
    /// Subscribe to consensus events
    pub fn subscribe_events(&self) -> broadcast::Receiver<ConsensusEvent> {
        self.events.subscribe()
    }
    
    /// Get blockchain reference
    pub fn blockchain(&self) -> &Arc<B> {
        &self.blockchain
    }
}

/// Validator management following Albatross patterns
pub struct ValidatorSet {
    validators: Vec<ValidatorInfo>,
    current_validators: Vec<ValidatorInfo>,
    next_validators: Vec<ValidatorInfo>,
}

impl ValidatorSet {
    pub fn new(validators: Vec<ValidatorInfo>) -> Self {
        Self {
            current_validators: validators.clone(),
            next_validators: validators.clone(), 
            validators,
        }
    }
    
    pub fn current_validators(&self) -> &[ValidatorInfo] {
        &self.current_validators
    }
    
    pub fn next_validators(&self) -> &[ValidatorInfo] {
        &self.next_validators
    }
    
    pub fn update_validators(&mut self, validators: Vec<ValidatorInfo>) {
        self.next_validators = validators;
    }
    
    pub fn finalize_epoch(&mut self) {
        self.current_validators = self.next_validators.clone();
    }
}

/// Tendermint-style vote for macro block production
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TendermintVote {
    pub proposal_hash: Option<Blake2bHash>,
    pub round: u32,
    pub step: TendermintStep,
    pub validator_idx: u16,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum TendermintStep {
    Propose = 1,
    Prevote = 2,  
    Precommit = 3,
}

/// Tendermint identifier for consensus rounds
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TendermintIdentifier {
    pub network: u8,
    pub block_number: u32,
    pub round_number: u32,
    pub step: TendermintStep,
}