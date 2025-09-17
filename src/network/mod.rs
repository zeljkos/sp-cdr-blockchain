// P2P networking layer for SP CDR reconciliation blockchain
use libp2p::{
    gossipsub::{self, Behaviour as Gossipsub, Event as GossipsubEvent, IdentTopic, MessageAuthenticity},
    identify::{self, Behaviour as Identify},
    mdns::{self, tokio::Behaviour as Mdns},
    noise,
    swarm::{NetworkBehaviour, SwarmEvent, ConnectionDenied, ConnectionId},
    tcp,
    yamux,
    Multiaddr, PeerId, Swarm, Transport,
};
use std::collections::HashSet;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, info, warn, error};
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

use crate::primitives::{Blake2bHash, NetworkId, BlockchainError};
use crate::blockchain::{Block, Transaction};

pub mod peer_discovery;
pub mod consensus_networking;
pub mod settlement_messaging;

pub use peer_discovery::PeerDiscovery;
pub use consensus_networking::ConsensusNetwork;
pub use settlement_messaging::SettlementMessaging;

/// SP-specific network messages for telecom operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SPNetworkMessage {
    /// Consensus messages
    BlockProposal {
        block: Block,
        #[serde(serialize_with = "serialize_peer_id", deserialize_with = "deserialize_peer_id")]
        proposer: PeerId,
        signature: Vec<u8>,
    },
    BlockVote {
        block_hash: Blake2bHash,
        #[serde(serialize_with = "serialize_peer_id", deserialize_with = "deserialize_peer_id")]
        voter: PeerId,
        approve: bool,
        signature: Vec<u8>,
    },

    /// Settlement negotiation
    SettlementProposal {
        creditor: NetworkId,
        debtor: NetworkId,
        amount_cents: u64,
        period_hash: Blake2bHash,
        nonce: u64,
    },
    SettlementAccept {
        proposal_hash: Blake2bHash,
        signature: Vec<u8>,
    },
    SettlementReject {
        proposal_hash: Blake2bHash,
        reason: String,
    },

    /// CDR batch coordination
    CDRBatchReady {
        batch_id: Blake2bHash,
        network_pair: (NetworkId, NetworkId),
        record_count: u32,
        total_amount: u64,
    },
    CDRBatchRequest {
        batch_id: Blake2bHash,
        requester: NetworkId,
    },

    /// ZK proof sharing
    ZKProofGenerated {
        proof_type: String, // "cdr_privacy" or "settlement"
        proof_data: Vec<u8>,
        public_inputs: Vec<u8>,
        network_id: NetworkId,
    },

    /// Validator coordination
    ValidatorAnnouncement {
        #[serde(serialize_with = "serialize_peer_id", deserialize_with = "deserialize_peer_id")]
        validator_id: PeerId,
        network_ids: Vec<NetworkId>,
        stake_amount: u64,
        endpoint: Multiaddr,
    },
}

/// Network event types for the application layer
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    PeerConnected(PeerId),
    PeerDisconnected(PeerId),
    MessageReceived {
        peer: PeerId,
        message: SPNetworkMessage,
    },
    GossipReceived {
        topic: String,
        message: SPNetworkMessage,
        source: PeerId,
    },
}

#[derive(NetworkBehaviour)]
pub struct SPNetworkBehaviour {
    pub gossipsub: Gossipsub,
    pub mdns: Mdns,
    pub identify: Identify,
}


/// Core P2P network manager for SP CDR blockchain
pub struct SPNetworkManager {
    swarm: Swarm<SPNetworkBehaviour>,
    event_sender: broadcast::Sender<NetworkEvent>,
    command_receiver: mpsc::Receiver<NetworkCommand>,

    // SP-specific topics
    consensus_topic: IdentTopic,
    settlement_topic: IdentTopic,
    cdr_topic: IdentTopic,
    zkp_topic: IdentTopic,

    // Network state
    connected_peers: HashSet<PeerId>,
    network_id: NetworkId,
}

/// Commands that can be sent to the network manager
#[derive(Debug)]
pub enum NetworkCommand {
    Connect(Multiaddr),
    Disconnect(PeerId),
    SendMessage {
        peer: PeerId,
        message: SPNetworkMessage,
    },
    Broadcast {
        topic: String,
        message: SPNetworkMessage,
    },
    JoinTopic(String),
    LeaveTopic(String),
}

impl SPNetworkManager {
    /// Create a new SP network manager
    pub async fn new(
        network_id: NetworkId,
        listen_addr: Multiaddr,
    ) -> std::result::Result<(Self, mpsc::Sender<NetworkCommand>, broadcast::Receiver<NetworkEvent>), BlockchainError> {
        // Generate keypair for this node
        let local_key = libp2p::identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        info!("SP Node Peer ID: {}", local_peer_id);
        info!("Network ID: {:?}", network_id);

        // Create transport
        let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
            .upgrade(libp2p::core::upgrade::Version::V1Lazy)
            .authenticate(noise::Config::new(&local_key)?)
            .multiplex(yamux::Config::default())
            .boxed();

        // Configure gossipsub
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(std::time::Duration::from_secs(10))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .message_id_fn(|message| {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                message.data.hash(&mut hasher);
                libp2p::gossipsub::MessageId::from(hasher.finish().to_be_bytes().to_vec())
            })
            .build()
            .map_err(|e| crate::primitives::BlockchainError::NetworkError(e.to_string()))?;

        let gossipsub = Gossipsub::new(
            MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        ).map_err(|e| crate::primitives::BlockchainError::NetworkError(e.to_string()))?;

        // Create other behaviors
        let mdns = Mdns::new(mdns::Config::default(), local_peer_id)
            .map_err(|e| crate::primitives::BlockchainError::NetworkError(e.to_string()))?;

        let identify = Identify::new(identify::Config::new(
            "/sp-cdr-blockchain/1.0.0".to_string(),
            local_key.public(),
        ));

        // Combine behaviors
        let behavior = SPNetworkBehaviour {
            gossipsub,
            mdns,
            identify,
        };

        // Create swarm
        let mut swarm = Swarm::new(transport, behavior, local_peer_id, libp2p::swarm::Config::with_tokio_executor());

        // Listen on the provided address
        swarm.listen_on(listen_addr)?;

        // Create communication channels
        let (event_sender, event_receiver) = broadcast::channel(1024);
        let (command_sender, command_receiver) = mpsc::channel(256);

        // Define SP-specific topics
        let consensus_topic = IdentTopic::new("sp-consensus");
        let settlement_topic = IdentTopic::new("sp-settlement");
        let cdr_topic = IdentTopic::new("sp-cdr");
        let zkp_topic = IdentTopic::new("sp-zkp");

        // Subscribe to topics
        swarm.behaviour_mut().gossipsub.subscribe(&consensus_topic)?;
        swarm.behaviour_mut().gossipsub.subscribe(&settlement_topic)?;
        swarm.behaviour_mut().gossipsub.subscribe(&cdr_topic)?;
        swarm.behaviour_mut().gossipsub.subscribe(&zkp_topic)?;

        let manager = SPNetworkManager {
            swarm,
            event_sender,
            command_receiver,
            consensus_topic,
            settlement_topic,
            cdr_topic,
            zkp_topic,
            connected_peers: HashSet::new(),
            network_id,
        };

        Ok((manager, command_sender, event_receiver))
    }

    /// Start the network event loop
    pub async fn run(mut self) {
        info!("Starting SP Network Manager for {:?}", self.network_id);

        loop {
            tokio::select! {
                // Handle swarm events
                event = futures::StreamExt::select_next_some(&mut self.swarm) => {
                    if let Err(e) = self.handle_swarm_event(event).await {
                        error!("Error handling swarm event: {}", e);
                    }
                }

                // Handle commands
                command = self.command_receiver.recv() => {
                    match command {
                        Some(cmd) => {
                            if let Err(e) = self.handle_command(cmd).await {
                                error!("Error handling command: {}", e);
                            }
                        }
                        None => {
                            warn!("Command channel closed, shutting down network manager");
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Handle swarm events
    async fn handle_swarm_event(&mut self, event: SwarmEvent<SPNetworkBehaviourEvent>) -> std::result::Result<(), BlockchainError> {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on: {}", address);
            }

            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                info!("Connected to peer: {}", peer_id);
                self.connected_peers.insert(peer_id);

                let _ = self.event_sender.send(NetworkEvent::PeerConnected(peer_id));
            }

            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                info!("Disconnected from peer: {}", peer_id);
                self.connected_peers.remove(&peer_id);

                let _ = self.event_sender.send(NetworkEvent::PeerDisconnected(peer_id));
            }

            SwarmEvent::Behaviour(SPNetworkBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source: source,
                message_id: _,
                message,
            })) => {
                self.handle_gossip_message(source, message).await?;
            }

            SwarmEvent::Behaviour(SPNetworkBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                for (peer_id, multiaddr) in list {
                    debug!("Discovered peer via mDNS: {} at {}", peer_id, multiaddr);

                    // Auto-connect to discovered SP nodes
                    if let Err(e) = self.swarm.dial(multiaddr) {
                        debug!("Failed to dial discovered peer: {}", e);
                    }
                }
            }

            SwarmEvent::Behaviour(SPNetworkBehaviourEvent::Identify(identify::Event::Received {
                peer_id,
                info,
            })) => {
                debug!("Identified peer {}: {}", peer_id, info.protocol_version);

                // Check if this is an SP node
                if info.protocol_version.contains("sp-cdr-blockchain") {
                    info!("Connected to SP CDR node: {}", peer_id);
                }
            }

            _ => {}
        }

        Ok(())
    }

    /// Handle gossipsub messages
    async fn handle_gossip_message(
        &mut self,
        source: PeerId,
        message: gossipsub::Message,
    ) -> std::result::Result<(), BlockchainError> {
        // Deserialize SP network message
        let sp_message: SPNetworkMessage = bincode::deserialize(&message.data)
            .map_err(|e| crate::primitives::BlockchainError::NetworkError(format!("Failed to deserialize message: {}", e)))?;

        debug!("Received gossip message from {}: {:?}", source, sp_message);

        let topic = message.topic.to_string();

        // Send to application layer
        let _ = self.event_sender.send(NetworkEvent::GossipReceived {
            topic,
            message: sp_message,
            source,
        });

        Ok(())
    }

    /// Handle network commands
    async fn handle_command(&mut self, command: NetworkCommand) -> std::result::Result<(), BlockchainError> {
        match command {
            NetworkCommand::Connect(addr) => {
                info!("Connecting to: {}", addr);
                self.swarm.dial(addr)?;
            }

            NetworkCommand::Disconnect(peer_id) => {
                info!("Disconnecting from: {}", peer_id);
                // libp2p doesn't have a direct disconnect method, we'd need to close the connection
                // For now, we just remove from our tracking
                self.connected_peers.remove(&peer_id);
            }

            NetworkCommand::SendMessage { peer, message } => {
                debug!("Sending direct message to {}: {:?}", peer, message);
                // For direct messaging, we'd need to implement a custom protocol
                // For now, we'll use gossip with a specific topic
                let serialized = bincode::serialize(&message)
                    .map_err(|e| crate::primitives::BlockchainError::NetworkError(format!("Serialization error: {}", e)))?;

                // Use a peer-specific topic for direct messaging
                let direct_topic = IdentTopic::new(format!("direct-{}", peer));
                self.swarm.behaviour_mut().gossipsub.subscribe(&direct_topic)?;
                self.swarm.behaviour_mut().gossipsub.publish(direct_topic, serialized)?;
            }

            NetworkCommand::Broadcast { topic, message } => {
                debug!("Broadcasting to topic {}: {:?}", topic, message);

                let serialized = bincode::serialize(&message)
                    .map_err(|e| crate::primitives::BlockchainError::NetworkError(format!("Serialization error: {}", e)))?;

                let gossip_topic = match topic.as_str() {
                    "consensus" => &self.consensus_topic,
                    "settlement" => &self.settlement_topic,
                    "cdr" => &self.cdr_topic,
                    "zkp" => &self.zkp_topic,
                    _ => {
                        warn!("Unknown topic: {}", topic);
                        return Ok(());
                    }
                };

                self.swarm.behaviour_mut().gossipsub.publish(gossip_topic.clone(), serialized)?;
            }

            NetworkCommand::JoinTopic(topic) => {
                debug!("Joining topic: {}", topic);
                let gossip_topic = IdentTopic::new(topic);
                self.swarm.behaviour_mut().gossipsub.subscribe(&gossip_topic)?;
            }

            NetworkCommand::LeaveTopic(topic) => {
                debug!("Leaving topic: {}", topic);
                let gossip_topic = IdentTopic::new(topic);
                self.swarm.behaviour_mut().gossipsub.unsubscribe(&gossip_topic)?;
            }
        }

        Ok(())
    }

    /// Get list of connected peers
    pub fn connected_peers(&self) -> Vec<PeerId> {
        self.connected_peers.iter().copied().collect()
    }

    /// Get network statistics
    pub fn network_stats(&self) -> NetworkStats {
        NetworkStats {
            connected_peers: self.connected_peers.len(),
            listening_addresses: self.swarm.listeners().cloned().collect(),
            local_peer_id: *self.swarm.local_peer_id(),
            network_id: self.network_id.clone(),
        }
    }
}

/// Network statistics
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub connected_peers: usize,
    pub listening_addresses: Vec<Multiaddr>,
    pub local_peer_id: PeerId,
    pub network_id: NetworkId,
}

/// Convenience functions for creating specific message types
impl SPNetworkMessage {
    pub fn block_proposal(block: Block, proposer: PeerId, signature: Vec<u8>) -> Self {
        Self::BlockProposal { block, proposer, signature }
    }

    pub fn settlement_proposal(
        creditor: NetworkId,
        debtor: NetworkId,
        amount_cents: u64,
        period_hash: Blake2bHash,
        nonce: u64,
    ) -> Self {
        Self::SettlementProposal {
            creditor,
            debtor,
            amount_cents,
            period_hash,
            nonce,
        }
    }

    pub fn cdr_batch_ready(
        batch_id: Blake2bHash,
        network_pair: (NetworkId, NetworkId),
        record_count: u32,
        total_amount: u64,
    ) -> Self {
        Self::CDRBatchReady {
            batch_id,
            network_pair,
            record_count,
            total_amount,
        }
    }

    pub fn zkp_generated(
        proof_type: String,
        proof_data: Vec<u8>,
        public_inputs: Vec<u8>,
        network_id: NetworkId,
    ) -> Self {
        Self::ZKProofGenerated {
            proof_type,
            proof_data,
            public_inputs,
            network_id,
        }
    }
}