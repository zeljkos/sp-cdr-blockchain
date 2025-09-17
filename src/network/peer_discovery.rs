// Peer discovery for SP CDR reconciliation network
use libp2p::{Multiaddr, PeerId};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, debug, error};
use serde::{Deserialize, Serialize};

use crate::primitives::{NetworkId, Blake2bHash, BlockchainError};

fn default_peer_id() -> PeerId {
    PeerId::random()
}

/// SP operator node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SPOperatorInfo {
    #[serde(skip)]
    #[serde(default = "default_peer_id")]
    pub peer_id: PeerId,
    pub network_id: NetworkId,
    pub operator_name: String,
    pub country_code: String,
    pub endpoints: Vec<Multiaddr>,
    pub validator_stake: u64,
    pub supported_currencies: Vec<String>,
    pub is_validator: bool,
    pub last_seen: u64, // timestamp
}

/// Known SP operators in the consortium
#[derive(Debug)]
pub struct PeerDiscovery {
    /// Known operators by peer ID
    operators: RwLock<HashMap<PeerId, SPOperatorInfo>>,

    /// Network ID to peer ID mapping
    network_to_peer: RwLock<HashMap<NetworkId, PeerId>>,

    /// Bootstrap nodes for initial discovery
    bootstrap_nodes: Vec<Multiaddr>,
}

impl PeerDiscovery {
    /// Create new peer discovery with bootstrap nodes
    pub fn new(bootstrap_nodes: Vec<Multiaddr>) -> Self {
        Self {
            operators: RwLock::new(HashMap::new()),
            network_to_peer: RwLock::new(HashMap::new()),
            bootstrap_nodes,
        }
    }

    /// Initialize with known SP consortium members
    pub async fn with_sp_consortium() -> std::result::Result<Self, BlockchainError> {
        let bootstrap_nodes = vec![
            // Production SP consortium bootstrap nodes would be here
            "/ip4/127.0.0.1/tcp/8000".parse()?,
            "/ip4/127.0.0.1/tcp/8001".parse()?,
            "/ip4/127.0.0.1/tcp/8002".parse()?,
        ];

        let discovery = Self::new(bootstrap_nodes);

        // Pre-populate with known major operators for demo
        discovery.add_known_operators().await;

        Ok(discovery)
    }

    /// Add known operators to the discovery table
    async fn add_known_operators(&self) {
        let known_operators = vec![
            SPOperatorInfo {
                peer_id: PeerId::random(), // In real implementation, these would be fixed
                network_id: NetworkId::new("T-Mobile", "DE"),
                operator_name: "T-Mobile Deutschland".to_string(),
                country_code: "DE".to_string(),
                endpoints: vec!["/ip4/127.0.0.1/tcp/8000".parse().unwrap()],
                validator_stake: 1_000_000,
                supported_currencies: vec!["EUR".to_string()],
                is_validator: true,
                last_seen: chrono::Utc::now().timestamp() as u64,
            },
            SPOperatorInfo {
                peer_id: PeerId::random(),
                network_id: NetworkId::new("Vodafone", "UK"),
                operator_name: "Vodafone UK".to_string(),
                country_code: "UK".to_string(),
                endpoints: vec!["/ip4/127.0.0.1/tcp/8001".parse().unwrap()],
                validator_stake: 800_000,
                supported_currencies: vec!["GBP".to_string(), "EUR".to_string()],
                is_validator: true,
                last_seen: chrono::Utc::now().timestamp() as u64,
            },
            SPOperatorInfo {
                peer_id: PeerId::random(),
                network_id: NetworkId::new("Orange", "FR"),
                operator_name: "Orange France".to_string(),
                country_code: "FR".to_string(),
                endpoints: vec!["/ip4/127.0.0.1/tcp/8002".parse().unwrap()],
                validator_stake: 900_000,
                supported_currencies: vec!["EUR".to_string()],
                is_validator: true,
                last_seen: chrono::Utc::now().timestamp() as u64,
            },
        ];

        let mut operators = self.operators.write().await;
        let mut network_to_peer = self.network_to_peer.write().await;

        for operator in known_operators {
            network_to_peer.insert(operator.network_id.clone(), operator.peer_id);
            operators.insert(operator.peer_id, operator);
        }

        info!("Initialized with {} known SP operators", operators.len());
    }

    /// Register a new operator
    pub async fn register_operator(&self, operator: SPOperatorInfo) -> std::result::Result<(), BlockchainError> {
        let peer_id = operator.peer_id;
        let network_id = operator.network_id.clone();

        info!("Registering operator: {} ({:?})", operator.operator_name, network_id);

        let mut operators = self.operators.write().await;
        let mut network_to_peer = self.network_to_peer.write().await;

        operators.insert(peer_id, operator);
        network_to_peer.insert(network_id, peer_id);

        Ok(())
    }

    /// Update operator information
    pub async fn update_operator(&self, peer_id: PeerId, update_fn: impl FnOnce(&mut SPOperatorInfo)) -> std::result::Result<(), BlockchainError> {
        let mut operators = self.operators.write().await;

        if let Some(operator) = operators.get_mut(&peer_id) {
            update_fn(operator);
            operator.last_seen = chrono::Utc::now().timestamp() as u64;
            debug!("Updated operator: {}", operator.operator_name);
        }

        Ok(())
    }

    /// Find operator by network ID
    pub async fn find_by_network(&self, network_id: &NetworkId) -> Option<SPOperatorInfo> {
        let network_to_peer = self.network_to_peer.read().await;
        let peer_id = network_to_peer.get(network_id)?;

        let operators = self.operators.read().await;
        operators.get(peer_id).cloned()
    }

    /// Find operator by peer ID
    pub async fn find_by_peer(&self, peer_id: &PeerId) -> Option<SPOperatorInfo> {
        let operators = self.operators.read().await;
        operators.get(peer_id).cloned()
    }

    /// Get all validators
    pub async fn get_validators(&self) -> Vec<SPOperatorInfo> {
        let operators = self.operators.read().await;
        operators.values()
            .filter(|op| op.is_validator)
            .cloned()
            .collect()
    }

    /// Get all operators in a country
    pub async fn get_operators_by_country(&self, country_code: &str) -> Vec<SPOperatorInfo> {
        let operators = self.operators.read().await;
        operators.values()
            .filter(|op| op.country_code == country_code)
            .cloned()
            .collect()
    }

    /// Get settlement partners for a network
    pub async fn get_settlement_partners(&self, network_id: &NetworkId) -> Vec<SPOperatorInfo> {
        let operators = self.operators.read().await;
        operators.values()
            .filter(|op| {
                // Settlement partners are operators that:
                // 1. Are not the same network
                // 2. Share at least one currency
                // 3. Are active (seen recently)
                op.network_id != *network_id &&
                self.has_common_currency(network_id, &op.network_id).unwrap_or(false) &&
                self.is_recently_active(op)
            })
            .cloned()
            .collect()
    }

    /// Check if two networks have common currencies
    fn has_common_currency(&self, network1: &NetworkId, network2: &NetworkId) -> Option<bool> {
        // This would check supported currencies - simplified for demo
        Some(true)
    }

    /// Check if operator is recently active (within last hour)
    fn is_recently_active(&self, operator: &SPOperatorInfo) -> bool {
        let now = chrono::Utc::now().timestamp() as u64;
        now - operator.last_seen < 3600 // 1 hour
    }

    /// Get bootstrap nodes for initial connection
    pub fn bootstrap_nodes(&self) -> &[Multiaddr] {
        &self.bootstrap_nodes
    }

    /// Get all known operators
    pub async fn all_operators(&self) -> Vec<SPOperatorInfo> {
        let operators = self.operators.read().await;
        operators.values().cloned().collect()
    }

    /// Remove operator (e.g., when they go offline)
    pub async fn remove_operator(&self, peer_id: PeerId) -> std::result::Result<(), BlockchainError> {
        let mut operators = self.operators.write().await;
        let mut network_to_peer = self.network_to_peer.write().await;

        if let Some(operator) = operators.remove(&peer_id) {
            network_to_peer.remove(&operator.network_id);
            info!("Removed operator: {}", operator.operator_name);
        }

        Ok(())
    }

    /// Get network topology for routing optimization
    pub async fn get_network_topology(&self) -> NetworkTopology {
        let operators = self.operators.read().await;

        let mut countries = HashMap::new();
        let mut validator_count = 0;
        let mut total_stake = 0;

        for operator in operators.values() {
            let country_ops = countries.entry(operator.country_code.clone()).or_insert(Vec::new());
            country_ops.push(operator.clone());

            if operator.is_validator {
                validator_count += 1;
                total_stake += operator.validator_stake;
            }
        }

        NetworkTopology {
            total_operators: operators.len(),
            validators: validator_count,
            total_stake,
            countries,
            last_updated: chrono::Utc::now().timestamp() as u64,
        }
    }
}

/// Network topology information
#[derive(Debug, Clone)]
pub struct NetworkTopology {
    pub total_operators: usize,
    pub validators: usize,
    pub total_stake: u64,
    pub countries: HashMap<String, Vec<SPOperatorInfo>>,
    pub last_updated: u64,
}

impl NetworkTopology {
    /// Check if network has sufficient validators for consensus
    pub fn has_sufficient_validators(&self) -> bool {
        self.validators >= 3 // Minimum for Byzantine fault tolerance
    }

    /// Get geographical distribution
    pub fn geographical_distribution(&self) -> HashMap<String, usize> {
        self.countries.iter()
            .map(|(country, ops)| (country.clone(), ops.len()))
            .collect()
    }

    /// Calculate total stake percentage for a country
    pub fn country_stake_percentage(&self, country_code: &str) -> f64 {
        if self.total_stake == 0 {
            return 0.0;
        }

        let country_stake: u64 = self.countries
            .get(country_code)
            .map(|ops| ops.iter().filter(|op| op.is_validator).map(|op| op.validator_stake).sum())
            .unwrap_or(0);

        (country_stake as f64 / self.total_stake as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_peer_discovery() {
        let discovery = PeerDiscovery::with_sp_consortium().await.unwrap();

        // Test finding by network
        let tmobile = discovery.find_by_network(&NetworkId::new("T-Mobile", "DE")).await;
        assert!(tmobile.is_some());

        // Test validators
        let validators = discovery.get_validators().await;
        assert_eq!(validators.len(), 3);

        // Test network topology
        let topology = discovery.get_network_topology().await;
        assert!(topology.has_sufficient_validators());
        assert_eq!(topology.total_operators, 3);
    }
}