// Block structures following Albatross patterns
use serde::{Deserialize, Serialize};
use crate::primitives::{Blake2bHash, Height, Timestamp, NetworkId, hash_json};

/// Block types following Albatross micro/macro pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Block {
    Micro(MicroBlock),
    Macro(MacroBlock),
}

impl Block {
    pub fn hash(&self) -> Blake2bHash {
        match self {
            Block::Micro(block) => hash_json(&block.header),
            Block::Macro(block) => hash_json(&block.header),
        }
    }

    pub fn block_number(&self) -> Height {
        match self {
            Block::Micro(block) => block.header.block_number,
            Block::Macro(block) => block.header.block_number,
        }
    }

    pub fn timestamp(&self) -> Timestamp {
        match self {
            Block::Micro(block) => block.header.timestamp,
            Block::Macro(block) => block.header.timestamp,
        }
    }

    pub fn parent_hash(&self) -> &Blake2bHash {
        match self {
            Block::Micro(block) => &block.header.parent_hash,
            Block::Macro(block) => &block.header.parent_hash,
        }
    }

    pub fn transactions(&self) -> &[Transaction] {
        match self {
            Block::Micro(block) => &block.body.transactions,
            Block::Macro(block) => &block.body.transactions,
        }
    }

    pub fn height(&self) -> Height {
        self.block_number()
    }
}

/// Micro block for CDR transactions (following Albatross micro blocks)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroBlock {
    pub header: MicroHeader,
    pub body: MicroBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroHeader {
    pub network: NetworkId,
    pub version: u16,
    pub block_number: Height,
    pub timestamp: Timestamp,
    pub parent_hash: Blake2bHash,
    pub seed: Blake2bHash,
    pub extra_data: Vec<u8>,
    pub state_root: Blake2bHash,
    pub body_root: Blake2bHash,
    pub history_root: Blake2bHash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroBody {
    pub transactions: Vec<Transaction>,
}

/// Macro block for epoch changes and validator set updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroBlock {
    pub header: MacroHeader,
    pub body: MacroBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroHeader {
    pub network: NetworkId,
    pub version: u16,
    pub block_number: Height,
    pub round: u32,
    pub timestamp: Timestamp,
    pub parent_hash: Blake2bHash,
    pub parent_election_hash: Blake2bHash,
    pub seed: Blake2bHash,
    pub extra_data: Vec<u8>,
    pub state_root: Blake2bHash,
    pub body_root: Blake2bHash,
    pub history_root: Blake2bHash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroBody {
    pub validators: Option<Vec<ValidatorInfo>>, // Only in election blocks
    pub lost_reward_set: Vec<Blake2bHash>,
    pub disabled_set: Vec<Blake2bHash>,
    pub transactions: Vec<Transaction>,
}

/// Transaction structure for CDR data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: Blake2bHash,
    pub recipient: Blake2bHash,
    pub value: u64,
    pub fee: u64,
    pub validity_start_height: Height,
    pub data: TransactionData,
    pub signature: Vec<u8>,
    pub signature_proof: Vec<u8>,
}

/// CDR-specific transaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionData {
    Basic,
    CDRRecord(CDRTransaction),
    Settlement(SettlementTransaction),
    ValidatorUpdate(ValidatorTransaction),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CDRTransaction {
    pub record_type: CDRType,
    pub home_network: String,
    pub visited_network: String,
    pub encrypted_data: Vec<u8>, // Privacy-protected CDR data
    pub zk_proof: Vec<u8>, // Zero-knowledge proof
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CDRType {
    VoiceCall,
    DataSession, 
    SMS,
    Roaming,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementTransaction {
    pub creditor_network: String,
    pub debtor_network: String,
    pub amount: u64,
    pub currency: String,
    pub period: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorTransaction {
    pub action: ValidatorAction,
    pub validator_address: Blake2bHash,
    pub stake: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidatorAction {
    CreateValidator,
    UpdateValidator,
    DeactivateValidator,
    ReactivateValidator,
}

/// Validator info following Albatross patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub address: Blake2bHash,
    pub signing_key: Vec<u8>, // BLS public key
    pub voting_key: Vec<u8>,  // Ed25519 public key  
    pub reward_address: Blake2bHash,
    pub signal_data: Option<Vec<u8>>,
    pub inactive_from: Option<Height>,
    pub jailed_from: Option<Height>,
}

impl Transaction {
    pub fn hash(&self) -> Blake2bHash {
        hash_json(self)
    }
    
    pub fn is_valid(&self) -> bool {
        // Basic validation
        !self.signature.is_empty() && self.fee > 0
    }
}