// Transaction types for CDR blockchain
use serde::{Deserialize, Serialize};
use crate::primitives::primitives::{Blake2bHash, Timestamp};
use crate::primitives::cdr::{CDRBatch, CDRStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Transaction {
    CDRRecord(CDRTransaction),
    Settlement(SettlementTransaction),
    NetworkJoin(NetworkJoinTransaction),
}

impl Transaction {
    pub fn hash(&self) -> Blake2bHash {
        let data = serde_json::to_vec(self).unwrap();
        crate::primitives::primitives::hash_data(&data)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CDRTransaction {
    pub batch_id: Blake2bHash,
    pub home_network: String,
    pub visited_network: String,
    pub record_count: u32,
    pub total_charges: u64,
    pub encrypted_data: Vec<u8>,
    pub privacy_proof: Vec<u8>,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementTransaction {
    pub settlement_id: Blake2bHash,
    pub creditor_network: String,
    pub debtor_network: String,
    pub amount: u64,
    pub currency: String,
    pub exchange_rate: u32,
    pub settlement_proof: Vec<u8>,
    pub batch_references: Vec<Blake2bHash>,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkJoinTransaction {
    pub network_name: String,
    pub public_key: Vec<u8>,
    pub country_code: String,
    pub operator_license: Vec<u8>,
    pub timestamp: Timestamp,
}