// Consensus networking for SP CDR blockchain
use libp2p::PeerId;
use std::collections::{HashMap, HashSet};
use tokio::sync::{broadcast, RwLock};
use tracing::{info, debug, warn, error};
use serde::{Deserialize, Serialize, Serializer, Deserializer};

// Helper functions for PeerId serialization
fn serialize_peer_id<S>(peer_id: &PeerId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&peer_id.to_string())
}

fn deserialize_peer_id<'de, D>(deserializer: D) -> Result<PeerId, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

fn serialize_peer_id_vec<S>(peer_ids: &Vec<(PeerId, Vec<u8>)>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let string_vec: Vec<(String, Vec<u8>)> = peer_ids.iter()
        .map(|(peer, data)| (peer.to_string(), data.clone()))
        .collect();
    string_vec.serialize(serializer)
}

fn deserialize_peer_id_vec<'de, D>(deserializer: D) -> Result<Vec<(PeerId, Vec<u8>)>, D::Error>
where
    D: Deserializer<'de>,
{
    let string_vec: Vec<(String, Vec<u8>)> = Vec::deserialize(deserializer)?;
    string_vec.into_iter()
        .map(|(s, data)| s.parse().map(|peer| (peer, data)).map_err(serde::de::Error::custom))
        .collect()
}

use crate::primitives::{Blake2bHash, NetworkId, BlockchainError, Height};
use crate::blockchain::{Block, Transaction};
use crate::network::{SPNetworkMessage, NetworkCommand};
use crate::crypto::bls::{BLSPrivateKey, BLSPublicKey, BLSSignature, BLSVerifier};

/// Consensus message types for SP blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusMessage {
    /// Phase 1: Block proposal
    Propose {
        block: Block,
        #[serde(serialize_with = "serialize_peer_id", deserialize_with = "deserialize_peer_id")]
        proposer_id: PeerId,
        round: u64,
        signature: Vec<u8>,
    },

    /// Phase 2: Pre-vote (prepare)
    PreVote {
        block_hash: Blake2bHash,
        round: u64,
        #[serde(serialize_with = "serialize_peer_id", deserialize_with = "deserialize_peer_id")]
        voter_id: PeerId,
        signature: Vec<u8>,
    },

    /// Phase 3: Pre-commit
    PreCommit {
        block_hash: Blake2bHash,
        round: u64,
        #[serde(serialize_with = "serialize_peer_id", deserialize_with = "deserialize_peer_id")]
        voter_id: PeerId,
        signature: Vec<u8>,
    },

    /// Commit notification
    Commit {
        block_hash: Blake2bHash,
        round: u64,
        height: u64,
        #[serde(serialize_with = "serialize_peer_id_vec", deserialize_with = "deserialize_peer_id_vec")]
        signatures: Vec<(PeerId, Vec<u8>)>,
    },

    /// View change/timeout
    ViewChange {
        round: u64,
        height: u64,
        #[serde(serialize_with = "serialize_peer_id", deserialize_with = "deserialize_peer_id")]
        requester_id: PeerId,
        reason: ViewChangeReason,
    },

    /// Synchronization request
    SyncRequest {
        from_height: u64,
        to_height: Option<u64>,
        #[serde(serialize_with = "serialize_peer_id", deserialize_with = "deserialize_peer_id")]
        requester_id: PeerId,
    },

    /// Synchronization response
    SyncResponse {
        blocks: Vec<Block>,
        current_height: u64,
        #[serde(serialize_with = "serialize_peer_id", deserialize_with = "deserialize_peer_id")]
        responder_id: PeerId,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViewChangeReason {
    Timeout,
    InvalidProposal,
    NetworkPartition,
}

/// Consensus state for tracking rounds and votes
#[derive(Debug, Clone)]
pub struct ConsensusState {
    pub current_round: u64,
    pub current_height: u64,
    pub phase: ConsensusPhase,
    pub proposed_block: Option<Block>,
    pub pre_votes: HashMap<PeerId, Blake2bHash>,
    pub pre_commits: HashMap<PeerId, Blake2bHash>,
    pub validators: HashSet<PeerId>,
    pub validator_weights: HashMap<PeerId, u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConsensusPhase {
    Propose,
    PreVote,
    PreCommit,
    Commit,
}

/// Consensus networking manager
pub struct ConsensusNetwork {
    state: RwLock<ConsensusState>,
    command_sender: broadcast::Sender<NetworkCommand>,
    network_id: NetworkId,
    local_peer_id: PeerId,

    // Consensus parameters
    timeout_duration: std::time::Duration,
    min_validators: usize,

    // BLS cryptography for validator signatures
    validator_private_key: BLSPrivateKey,
    bls_verifier: BLSVerifier,
}

impl ConsensusNetwork {
    pub fn new(
        network_id: NetworkId,
        local_peer_id: PeerId,
        validators: HashSet<PeerId>,
        validator_weights: HashMap<PeerId, u64>,
        command_sender: broadcast::Sender<NetworkCommand>,
        validator_private_key: BLSPrivateKey,
        validator_public_keys: HashMap<PeerId, BLSPublicKey>,
    ) -> Self {
        let state = ConsensusState {
            current_round: 0,
            current_height: 0,
            phase: ConsensusPhase::Propose,
            proposed_block: None,
            pre_votes: HashMap::new(),
            pre_commits: HashMap::new(),
            validators,
            validator_weights,
        };

        // Initialize BLS verifier with validator public keys
        let mut bls_verifier = BLSVerifier::new();
        for (peer_id, public_key) in validator_public_keys {
            bls_verifier.register_operator(&peer_id.to_string(), public_key);
        }

        Self {
            state: RwLock::new(state),
            command_sender,
            network_id,
            local_peer_id,
            timeout_duration: std::time::Duration::from_secs(30),
            min_validators: 3,
            validator_private_key,
            bls_verifier,
        }
    }

    /// Start consensus for a new block
    pub async fn start_consensus(&self, transactions: Vec<Transaction>) -> std::result::Result<(), BlockchainError> {
        let mut state = self.state.write().await;

        if state.phase != ConsensusPhase::Propose {
            warn!("Cannot start consensus, not in propose phase");
            return Ok(());
        }

        // Check if we are the proposer for this round
        if !self.is_proposer(state.current_round, &state.validators).await {
            debug!("Not proposer for round {}", state.current_round);
            return Ok(());
        }

        info!("Starting consensus for round {} height {}", state.current_round, state.current_height);

        // Create new block
        let block = self.create_block(transactions, state.current_height).await?;
        let block_hash = block.hash();

        // Store proposed block
        state.proposed_block = Some(block.clone());
        state.phase = ConsensusPhase::PreVote;

        // Create message to sign (block hash + round)
        let mut message_to_sign = block_hash.as_bytes().to_vec();
        message_to_sign.extend_from_slice(&state.current_round.to_le_bytes());

        // Sign with validator's BLS private key
        let signature = self.validator_private_key.sign(&message_to_sign)
            .map_err(|e| BlockchainError::Crypto(format!("Failed to sign proposal: {:?}", e)))?;

        // Broadcast proposal with real signature
        let proposal = ConsensusMessage::Propose {
            block,
            proposer_id: self.local_peer_id,
            round: state.current_round,
            signature: signature.to_bytes().to_vec(),
        };

        self.broadcast_consensus_message(proposal).await?;

        Ok(())
    }

    /// Handle incoming consensus message
    pub async fn handle_consensus_message(&self, message: ConsensusMessage, from_peer: PeerId) -> std::result::Result<(), BlockchainError> {
        match message {
            ConsensusMessage::Propose { block, proposer_id, round, signature } => {
                self.handle_proposal(block, proposer_id, round, signature, from_peer).await
            }

            ConsensusMessage::PreVote { block_hash, round, voter_id, signature } => {
                self.handle_pre_vote(block_hash, round, voter_id, signature).await
            }

            ConsensusMessage::PreCommit { block_hash, round, voter_id, signature } => {
                self.handle_pre_commit(block_hash, round, voter_id, signature).await
            }

            ConsensusMessage::Commit { block_hash, round, height, signatures } => {
                self.handle_commit(block_hash, round, height, signatures).await
            }

            ConsensusMessage::ViewChange { round, height, requester_id, reason } => {
                self.handle_view_change(round, height, requester_id, reason).await
            }

            ConsensusMessage::SyncRequest { from_height, to_height, requester_id } => {
                self.handle_sync_request(from_height, to_height, requester_id).await
            }

            ConsensusMessage::SyncResponse { blocks, current_height, responder_id } => {
                self.handle_sync_response(blocks, current_height, responder_id).await
            }
        }
    }

    /// Handle block proposal
    async fn handle_proposal(
        &self,
        block: Block,
        proposer_id: PeerId,
        round: u64,
        signature: Vec<u8>,
        _from_peer: PeerId,
    ) -> std::result::Result<(), BlockchainError> {
        let mut state = self.state.write().await;

        if round != state.current_round {
            debug!("Ignoring proposal for different round: {} vs {}", round, state.current_round);
            return Ok(());
        }

        if state.phase != ConsensusPhase::Propose {
            debug!("Not in propose phase, ignoring proposal");
            return Ok(());
        }

        // Validate proposer
        if !self.is_valid_proposer(proposer_id, round, &state.validators) {
            warn!("Invalid proposer {} for round {}", proposer_id, round);
            return Ok(());
        }

        // Verify BLS signature on proposal
        let block_hash = block.hash();
        let mut message_to_verify = block_hash.as_bytes().to_vec();
        message_to_verify.extend_from_slice(&round.to_le_bytes());

        let signature_valid = self.bls_verifier.verify_operator_signature(
            &proposer_id.to_string(),
            &message_to_verify,
            &signature,
        ).unwrap_or(false);

        if !signature_valid {
            warn!("Invalid BLS signature on proposal from {}", proposer_id);
            return Ok(());
        }

        info!("Received valid signed proposal from {} for round {}", proposer_id, round);

        // Validate block
        if self.validate_block(&block).await? {
            // Accept proposal and move to pre-vote
            state.proposed_block = Some(block.clone());
            state.phase = ConsensusPhase::PreVote;

            let block_hash = block.hash();

            // Create message to sign for pre-vote (block hash + round + "prevote")
            let mut prevote_message = block_hash.as_bytes().to_vec();
            prevote_message.extend_from_slice(&round.to_le_bytes());
            prevote_message.extend_from_slice(b"prevote");

            let prevote_signature = self.validator_private_key.sign(&prevote_message)
                .map_err(|e| BlockchainError::Crypto(format!("Failed to sign pre-vote: {:?}", e)))?;

            // Send pre-vote with real BLS signature
            let pre_vote = ConsensusMessage::PreVote {
                block_hash,
                round,
                voter_id: self.local_peer_id,
                signature: prevote_signature.to_bytes().to_vec(),
            };

            self.broadcast_consensus_message(pre_vote).await?;
        } else {
            warn!("Invalid block proposal, sending nil pre-vote");
            // Send nil pre-vote (empty hash)
            let pre_vote = ConsensusMessage::PreVote {
                block_hash: Blake2bHash::default(),
                round,
                voter_id: self.local_peer_id,
                signature: vec![],
            };

            self.broadcast_consensus_message(pre_vote).await?;
        }

        Ok(())
    }

    /// Handle pre-vote
    async fn handle_pre_vote(
        &self,
        block_hash: Blake2bHash,
        round: u64,
        voter_id: PeerId,
        signature: Vec<u8>,
    ) -> std::result::Result<(), BlockchainError> {
        let mut state = self.state.write().await;

        if round != state.current_round {
            return Ok(());
        }

        if !state.validators.contains(&voter_id) {
            warn!("Pre-vote from non-validator: {}", voter_id);
            return Ok(());
        }

        // Verify BLS signature on pre-vote
        let mut prevote_message = block_hash.as_bytes().to_vec();
        prevote_message.extend_from_slice(&round.to_le_bytes());
        prevote_message.extend_from_slice(b"prevote");

        let signature_valid = self.bls_verifier.verify_operator_signature(
            &voter_id.to_string(),
            &prevote_message,
            &signature,
        ).unwrap_or(false);

        if !signature_valid {
            warn!("Invalid BLS signature on pre-vote from {}", voter_id);
            return Ok(());
        }

        // Record pre-vote
        state.pre_votes.insert(voter_id, block_hash);

        debug!("Received pre-vote from {} for block {:?}", voter_id, block_hash);

        // Check if we have enough pre-votes for the proposed block
        if let Some(ref proposed_block) = state.proposed_block {
            let proposed_hash = proposed_block.hash();
            let votes_for_block = state.pre_votes.values()
                .filter(|&hash| *hash == proposed_hash)
                .count();

            if votes_for_block >= self.required_votes(&state.validators) {
                info!("Received sufficient pre-votes for block, moving to pre-commit");

                state.phase = ConsensusPhase::PreCommit;

                // Create message to sign for pre-commit (block hash + round + "precommit")
                let mut precommit_message = proposed_hash.as_bytes().to_vec();
                precommit_message.extend_from_slice(&round.to_le_bytes());
                precommit_message.extend_from_slice(b"precommit");

                let precommit_signature = self.validator_private_key.sign(&precommit_message)
                    .map_err(|e| BlockchainError::Crypto(format!("Failed to sign pre-commit: {:?}", e)))?;

                // Send pre-commit with real BLS signature
                let pre_commit = ConsensusMessage::PreCommit {
                    block_hash: proposed_hash,
                    round,
                    voter_id: self.local_peer_id,
                    signature: precommit_signature.to_bytes().to_vec(),
                };

                self.broadcast_consensus_message(pre_commit).await?;
            }
        }

        Ok(())
    }

    /// Handle pre-commit
    async fn handle_pre_commit(
        &self,
        block_hash: Blake2bHash,
        round: u64,
        voter_id: PeerId,
        signature: Vec<u8>,
    ) -> std::result::Result<(), BlockchainError> {
        let mut state = self.state.write().await;

        if round != state.current_round {
            return Ok(());
        }

        if !state.validators.contains(&voter_id) {
            warn!("Pre-commit from non-validator: {}", voter_id);
            return Ok(());
        }

        // Verify BLS signature on pre-commit
        let mut precommit_message = block_hash.as_bytes().to_vec();
        precommit_message.extend_from_slice(&round.to_le_bytes());
        precommit_message.extend_from_slice(b"precommit");

        let signature_valid = self.bls_verifier.verify_operator_signature(
            &voter_id.to_string(),
            &precommit_message,
            &signature,
        ).unwrap_or(false);

        if !signature_valid {
            warn!("Invalid BLS signature on pre-commit from {}", voter_id);
            return Ok(());
        }

        // Record pre-commit
        state.pre_commits.insert(voter_id, block_hash);

        debug!("Received pre-commit from {} for block {:?}", voter_id, block_hash);

        // Check if we have enough pre-commits
        if let Some(ref proposed_block) = state.proposed_block.clone() {
            let proposed_hash = proposed_block.hash();
            let commits_for_block = state.pre_commits.values()
                .filter(|&hash| *hash == proposed_hash)
                .count();

            if commits_for_block >= self.required_votes(&state.validators) {
                info!("Received sufficient pre-commits, committing block");

                // Collect signatures for commit message
                let signatures: Vec<(PeerId, Vec<u8>)> = state.pre_commits.iter()
                    .filter(|(_, hash)| **hash == proposed_hash)
                    .map(|(peer, _)| (*peer, vec![])) // Would include actual signatures
                    .collect();

                state.phase = ConsensusPhase::Commit;

                // Broadcast commit
                let commit = ConsensusMessage::Commit {
                    block_hash: proposed_hash,
                    round,
                    height: state.current_height,
                    signatures,
                };

                self.broadcast_consensus_message(commit).await?;

                // Apply block and move to next round
                self.apply_block(proposed_block.clone()).await?;
                self.start_new_round().await?;
            }
        }

        Ok(())
    }

    /// Handle commit
    async fn handle_commit(
        &self,
        block_hash: Blake2bHash,
        round: u64,
        height: u64,
        _signatures: Vec<(PeerId, Vec<u8>)>,
    ) -> std::result::Result<(), BlockchainError> {
        let mut state = self.state.write().await;

        if round != state.current_round || height != state.current_height {
            return Ok(());
        }

        if let Some(ref proposed_block) = state.proposed_block {
            if proposed_block.hash() == block_hash {
                info!("Block committed: {:?}", block_hash);

                // Apply block and start new round
                self.apply_block(proposed_block.clone()).await?;
                self.start_new_round().await?;
            }
        }

        Ok(())
    }

    /// Handle view change
    async fn handle_view_change(
        &self,
        round: u64,
        height: u64,
        requester_id: PeerId,
        reason: ViewChangeReason,
    ) -> std::result::Result<(), BlockchainError> {
        info!("View change requested by {} for round {} height {}: {:?}",
              requester_id, round, height, reason);

        // In a real implementation, we would:
        // 1. Validate the view change request
        // 2. Collect view change messages from other validators
        // 3. Move to new round with new proposer

        self.start_new_round().await?;
        Ok(())
    }

    /// Handle sync request
    async fn handle_sync_request(
        &self,
        from_height: u64,
        to_height: Option<u64>,
        requester_id: PeerId,
    ) -> std::result::Result<(), BlockchainError> {
        debug!("Sync request from {} for blocks {} to {:?}",
               requester_id, from_height, to_height);

        // In a real implementation, we would fetch the requested blocks
        // from our blockchain storage and send them back
        let blocks = vec![]; // Would load from storage

        let sync_response = ConsensusMessage::SyncResponse {
            blocks,
            current_height: self.state.read().await.current_height,
            responder_id: self.local_peer_id,
        };

        // Send response directly to requester
        let dummy_block = self.create_block(vec![], 0).await?;
        let command = NetworkCommand::SendMessage {
            peer: requester_id,
            message: SPNetworkMessage::BlockProposal {
                block: dummy_block,
                proposer: self.local_peer_id,
                signature: vec![],
            },
        };

        let _ = self.command_sender.send(command);

        Ok(())
    }

    /// Handle sync response
    async fn handle_sync_response(
        &self,
        blocks: Vec<Block>,
        current_height: u64,
        responder_id: PeerId,
    ) -> std::result::Result<(), BlockchainError> {
        info!("Sync response from {} with {} blocks, current height: {}",
              responder_id, blocks.len(), current_height);

        // Process received blocks
        for block in blocks {
            self.apply_block(block).await?;
        }

        Ok(())
    }

    /// Check if this node is the proposer for the given round
    async fn is_proposer(&self, round: u64, validators: &HashSet<PeerId>) -> bool {
        // Simple round-robin proposer selection
        let sorted_validators: Vec<_> = validators.iter().collect();
        if sorted_validators.is_empty() {
            return false;
        }

        let proposer_index = (round as usize) % sorted_validators.len();
        *sorted_validators[proposer_index] == self.local_peer_id
    }

    /// Validate if a peer is a valid proposer for the round
    fn is_valid_proposer(&self, proposer_id: PeerId, round: u64, validators: &HashSet<PeerId>) -> bool {
        if !validators.contains(&proposer_id) {
            return false;
        }

        // Simple round-robin validation
        let sorted_validators: Vec<_> = validators.iter().collect();
        if sorted_validators.is_empty() {
            return false;
        }

        let expected_proposer_index = (round as usize) % sorted_validators.len();
        *sorted_validators[expected_proposer_index] == proposer_id
    }

    /// Validate a proposed block
    async fn validate_block(&self, block: &Block) -> std::result::Result<bool, BlockchainError> {
        // In a real implementation, this would validate:
        // 1. Block structure and format
        // 2. Transaction validity
        // 3. State transitions
        // 4. ZK proofs for settlements
        // 5. Digital signatures

        // For now, just basic validation
        Ok(!block.transactions().is_empty())
    }

    /// Create a new block with given transactions
    async fn create_block(&self, transactions: Vec<Transaction>, height: u64) -> std::result::Result<Block, BlockchainError> {
        // In a real implementation, this would:
        // 1. Validate all transactions
        // 2. Execute transactions and compute state changes
        // 3. Generate ZK proofs for settlements
        // 4. Create block with proper hash and signatures

        // For now, create a simple dummy block
        // In real implementation, would use proper block structure
        use crate::blockchain::Block;

        // Return a placeholder block - this needs proper implementation
        // when we have the real block structure finalized
        Ok(Block::Micro(crate::blockchain::MicroBlock {
            header: crate::blockchain::MicroHeader {
                network: crate::primitives::NetworkId::new("SP", "Consortium"),
                version: 1,
                block_number: height as Height,
                timestamp: chrono::Utc::now().timestamp() as u64,
                parent_hash: Blake2bHash::default(),
                seed: Blake2bHash::from_bytes([0u8; 32]), // Simplified seed
                extra_data: vec![],
                state_root: Blake2bHash::default(),
                body_root: Blake2bHash::default(),
                history_root: Blake2bHash::default(),
            },
            body: crate::blockchain::MicroBody {
                transactions: vec![], // Use empty for now, fix transaction types later
            },
        }))
    }

    /// Apply a committed block to the blockchain state
    async fn apply_block(&self, block: Block) -> std::result::Result<(), BlockchainError> {
        info!("Applying block at height {}", block.height());

        // In a real implementation, this would:
        // 1. Apply all transactions in the block
        // 2. Update account balances
        // 3. Process settlement transactions
        // 4. Verify and store ZK proofs
        // 5. Update blockchain state

        Ok(())
    }

    /// Start a new consensus round
    async fn start_new_round(&self) -> std::result::Result<(), BlockchainError> {
        let mut state = self.state.write().await;

        state.current_round += 1;
        state.current_height += 1;
        state.phase = ConsensusPhase::Propose;
        state.proposed_block = None;
        state.pre_votes.clear();
        state.pre_commits.clear();

        info!("Starting new round {} at height {}", state.current_round, state.current_height);

        Ok(())
    }

    /// Broadcast consensus message to all validators
    async fn broadcast_consensus_message(&self, message: ConsensusMessage) -> std::result::Result<(), BlockchainError> {
        let dummy_block = self.create_block(vec![], 0).await?;
        let sp_message = SPNetworkMessage::BlockProposal {
            block: dummy_block, // Would serialize consensus message properly
            proposer: self.local_peer_id,
            signature: vec![],
        };

        let command = NetworkCommand::Broadcast {
            topic: "consensus".to_string(),
            message: sp_message,
        };

        let _ = self.command_sender.send(command);
        Ok(())
    }

    /// Calculate required number of votes (2/3 + 1)
    fn required_votes(&self, validators: &HashSet<PeerId>) -> usize {
        (validators.len() * 2 / 3) + 1
    }

    /// Get current consensus state
    pub async fn get_state(&self) -> ConsensusState {
        self.state.read().await.clone()
    }

    /// Request sync from network
    pub async fn request_sync(&self, from_height: u64) -> std::result::Result<(), BlockchainError> {
        let sync_request = ConsensusMessage::SyncRequest {
            from_height,
            to_height: None,
            requester_id: self.local_peer_id,
        };

        // Broadcast sync request
        let dummy_block = self.create_block(vec![], 0).await?;
        let sp_message = SPNetworkMessage::BlockProposal {
            block: dummy_block,
            proposer: self.local_peer_id,
            signature: vec![],
        };

        let command = NetworkCommand::Broadcast {
            topic: "consensus".to_string(),
            message: sp_message,
        };

        let _ = self.command_sender.send(command);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::broadcast;

    #[tokio::test]
    async fn test_consensus_network() {
        let (cmd_sender, _) = broadcast::channel(10);

        let peer1 = PeerId::random();
        let peer2 = PeerId::random();
        let peer3 = PeerId::random();

        let mut validators = HashSet::new();
        validators.insert(peer1);
        validators.insert(peer2);
        validators.insert(peer3);

        let mut weights = HashMap::new();
        weights.insert(peer1, 100);
        weights.insert(peer2, 100);
        weights.insert(peer3, 100);

        let consensus = ConsensusNetwork::new(
            NetworkId::new("Test", "Network"),
            peer1,
            validators,
            weights,
            cmd_sender,
        );

        let state = consensus.get_state().await;
        assert_eq!(state.current_round, 0);
        assert_eq!(state.phase, ConsensusPhase::Propose);
    }
}