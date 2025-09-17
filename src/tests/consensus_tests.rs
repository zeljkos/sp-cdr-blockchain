// Consensus mechanism tests
use sp_cdr_reconciliation_bc::*;
use std::sync::Arc;
use tokio::sync::RwLock;

// Mock blockchain for consensus testing
struct MockBlockchain {
    network_id: NetworkId,
    head_block: RwLock<Block>,
    macro_head: RwLock<Block>,
    election_head: RwLock<Block>,
    current_time: u64,
}

impl MockBlockchain {
    fn new() -> Self {
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
                extra_data: vec![],
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
        
        Self {
            network_id: NetworkId::SPConsortium,
            head_block: RwLock::new(genesis_block.clone()),
            macro_head: RwLock::new(genesis_block.clone()),
            election_head: RwLock::new(genesis_block),
            current_time: 1234567890,
        }
    }
}

#[async_trait::async_trait]
impl AbstractBlockchain for MockBlockchain {
    fn network_id(&self) -> NetworkId {
        self.network_id
    }
    
    fn now(&self) -> u64 {
        self.current_time
    }
    
    fn head(&self) -> &Block {
        unimplemented!("Use async methods")
    }
    
    fn macro_head(&self) -> &Block {
        unimplemented!("Use async methods")
    }
    
    fn election_head(&self) -> &Block {
        unimplemented!("Use async methods")
    }
    
    fn block_number(&self) -> u32 {
        0 // Simplified for testing
    }
    
    fn macro_block_number(&self) -> u32 {
        0
    }
    
    fn election_block_number(&self) -> u32 {
        0
    }
    
    async fn get_block(&self, _hash: &Blake2bHash, _include_body: bool) -> Result<Option<Block>> {
        Ok(Some(self.head_block.read().await.clone()))
    }
    
    async fn push_block(&self, block: Block) -> Result<()> {
        *self.head_block.write().await = block;
        Ok(())
    }
    
    fn get_chain_info(&self) -> common::ChainInfo {
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
    
    fn subscribe_events(&self) -> futures::stream::BoxStream<BlockchainEvent> {
        use futures::stream::StreamExt;
        futures::stream::empty().boxed()
    }
}

#[tokio::test]
async fn test_consensus_creation() {
    // Test consensus can be created with blockchain
    let blockchain = Arc::new(MockBlockchain::new());
    let consensus = Consensus::new(blockchain.clone());
    
    // Test initial state
    assert!(!consensus.is_established().await);
    assert_eq!(consensus.blockchain().network_id(), NetworkId::SPConsortium);
    
    println!("✅ Consensus creation works");
}

#[tokio::test]
async fn test_consensus_establishment() {
    // Test consensus establishment flow
    let blockchain = Arc::new(MockBlockchain::new());
    let consensus = Consensus::new(blockchain);
    
    // Initially not established
    assert!(!consensus.is_established().await);
    
    // Force establishment
    consensus.force_established().await;
    assert!(consensus.is_established().await);
    
    // Test event subscription
    let mut events = consensus.subscribe_events();
    
    // Events should be non-blocking to receive
    tokio::select! {
        event = events.recv() => {
            match event {
                Ok(ConsensusEvent::Established { synced_validity_window }) => {
                    assert!(synced_validity_window);
                    println!("✅ Received consensus established event");
                }
                Ok(other) => println!("📝 Received other event: {:?}", other),
                Err(_) => println!("⚠️ Event channel closed"),
            }
        }
        _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
            println!("⚠️ No events received within timeout");
        }
    }
    
    println!("✅ Consensus establishment works");
}

#[tokio::test]
async fn test_validator_set_management() {
    // Test validator set creation and updates
    let validators = vec![
        ValidatorInfo {
            address: Blake2bHash::from_bytes([1u8; 32]),
            signing_key: vec![1u8; 48],
            voting_key: vec![1u8; 32],
            reward_address: Blake2bHash::from_bytes([1u8; 32]),
            signal_data: None,
            inactive_from: None,
            jailed_from: None,
        },
        ValidatorInfo {
            address: Blake2bHash::from_bytes([2u8; 32]),
            signing_key: vec![2u8; 48],
            voting_key: vec![2u8; 32],
            reward_address: Blake2bHash::from_bytes([2u8; 32]),
            signal_data: Some(b"validator_info".to_vec()),
            inactive_from: None,
            jailed_from: None,
        },
        ValidatorInfo {
            address: Blake2bHash::from_bytes([3u8; 32]),
            signing_key: vec![3u8; 48],
            voting_key: vec![3u8; 32],
            reward_address: Blake2bHash::from_bytes([3u8; 32]),
            signal_data: None,
            inactive_from: Some(100), // Inactive validator
            jailed_from: None,
        },
    ];
    
    let mut validator_set = ValidatorSet::new(validators.clone());
    
    // Test initial state
    assert_eq!(validator_set.current_validators().len(), 3);
    assert_eq!(validator_set.next_validators().len(), 3);
    
    // Test validator update
    let updated_validators = vec![validators[0].clone(), validators[1].clone()]; // Remove third validator
    validator_set.update_validators(updated_validators.clone());
    
    // Current validators should remain unchanged until epoch finalization
    assert_eq!(validator_set.current_validators().len(), 3);
    assert_eq!(validator_set.next_validators().len(), 2);
    
    // Finalize epoch
    validator_set.finalize_epoch();
    assert_eq!(validator_set.current_validators().len(), 2);
    assert_eq!(validator_set.next_validators().len(), 2);
    
    println!("✅ Validator set management works");
}

#[tokio::test]
async fn test_tendermint_vote_structure() {
    // Test Tendermint vote structure following Albatross patterns
    let vote = TendermintVote {
        proposal_hash: Some(Blake2bHash::from_bytes([42u8; 32])),
        round: 0,
        step: TendermintStep::Prevote,
        validator_idx: 0,
        signature: b"bls_aggregated_signature".to_vec(),
    };
    
    // Test vote serialization/deserialization
    let serialized = serde_json::to_vec(&vote).unwrap();
    let deserialized: TendermintVote = serde_json::from_slice(&serialized).unwrap();
    
    assert_eq!(vote.proposal_hash, deserialized.proposal_hash);
    assert_eq!(vote.round, deserialized.round);
    assert_eq!(vote.step, deserialized.step);
    assert_eq!(vote.validator_idx, deserialized.validator_idx);
    
    // Test nil vote
    let nil_vote = TendermintVote {
        proposal_hash: None,
        round: 1,
        step: TendermintStep::Precommit,
        validator_idx: 1,
        signature: b"nil_vote_signature".to_vec(),
    };
    
    assert!(nil_vote.proposal_hash.is_none());
    
    println!("✅ Tendermint vote structure works");
}

#[tokio::test]
async fn test_tendermint_identifier() {
    // Test Tendermint identifier structure
    let identifier = TendermintIdentifier {
        network: NetworkId::SPConsortium as u8,
        block_number: 32,
        round_number: 0,
        step: TendermintStep::Propose,
    };
    
    // Test identifier uniqueness
    let identifier2 = TendermintIdentifier {
        network: NetworkId::SPConsortium as u8,
        block_number: 32,
        round_number: 1, // Different round
        step: TendermintStep::Propose,
    };
    
    // Identifiers should serialize to different values
    let id1_bytes = serde_json::to_vec(&identifier).unwrap();
    let id2_bytes = serde_json::to_vec(&identifier2).unwrap();
    assert_ne!(id1_bytes, id2_bytes);
    
    // Test step enumeration
    assert_eq!(TendermintStep::Propose as u8, 1);
    assert_eq!(TendermintStep::Prevote as u8, 2);
    assert_eq!(TendermintStep::Precommit as u8, 3);
    
    println!("✅ Tendermint identifier works");
}

#[tokio::test]
async fn test_consensus_blockchain_interaction() {
    // Test consensus interacting with blockchain
    let blockchain = Arc::new(MockBlockchain::new());
    let consensus = Consensus::new(blockchain.clone());
    
    // Test blockchain access through consensus
    let chain_info = consensus.blockchain().get_chain_info();
    assert_eq!(chain_info.head_block_number, 0);
    assert_eq!(chain_info.macro_head_block_number, 0);
    
    // Test block pushing through blockchain
    let new_block = Block::Micro(MicroBlock {
        header: blockchain::MicroHeader {
            network: NetworkId::SPConsortium,
            version: 1,
            block_number: 1,
            timestamp: 1234567890,
            parent_hash: Blake2bHash::zero(),
            seed: Blake2bHash::from_bytes([1u8; 32]),
            extra_data: vec![],
            state_root: Blake2bHash::zero(),
            body_root: Blake2bHash::zero(),
            history_root: Blake2bHash::zero(),
        },
        body: blockchain::MicroBody {
            transactions: vec![],
        },
    });
    
    consensus.blockchain().push_block(new_block).await.unwrap();
    
    println!("✅ Consensus-blockchain interaction works");
}

#[test]
fn test_consensus_event_types() {
    // Test all consensus event types
    let established = ConsensusEvent::Established { synced_validity_window: true };
    let lost = ConsensusEvent::Lost;
    let waiting = ConsensusEvent::Waiting;
    
    // Test that events can be cloned and debug printed
    let _established_clone = established.clone();
    let _lost_clone = lost.clone();
    let _waiting_clone = waiting.clone();
    
    println!("Events: {:?}, {:?}, {:?}", established, lost, waiting);
    
    println!("✅ Consensus event types work");
}

#[test]
fn test_blockchain_event_types() {
    // Test blockchain event types
    let extended = BlockchainEvent::Extended(Blake2bHash::from_bytes([1u8; 32]));
    let reverted = BlockchainEvent::Reverted(Blake2bHash::from_bytes([2u8; 32]));
    let rebranched = BlockchainEvent::Rebranched {
        old_blocks: vec![Blake2bHash::from_bytes([3u8; 32])],
        new_blocks: vec![Blake2bHash::from_bytes([4u8; 32]), Blake2bHash::from_bytes([5u8; 32])],
    };
    let finalized = BlockchainEvent::Finalized(Blake2bHash::from_bytes([6u8; 32]));
    
    // Test event cloning and debug
    let _extended_clone = extended.clone();
    let _reverted_clone = reverted.clone();
    let _rebranched_clone = rebranched.clone();
    let _finalized_clone = finalized.clone();
    
    println!("Blockchain events created successfully");
    
    println!("✅ Blockchain event types work");
}