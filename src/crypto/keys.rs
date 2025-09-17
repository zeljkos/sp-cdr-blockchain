// Key management for SP CDR reconciliation validators
// Handles validator keys, network operator keys, and key rotation

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::primitives::{Blake2bHash, hash_data};
use super::{
    PrivateKey, PublicKey, CompressedPublicKey, 
    CryptoError, Result
};

/// Key pair for validators and network operators
#[derive(Clone, Debug)]
pub struct KeyPair {
    pub private_key: PrivateKey,
    pub public_key: PublicKey,
    pub key_id: Blake2bHash,
}

impl KeyPair {
    /// Generate a new key pair
    pub fn generate() -> Result<Self> {
        let private_key = PrivateKey::generate()?;
        let public_key = private_key.public_key()?;
        let key_id = Self::compute_key_id(&public_key);

        Ok(Self {
            private_key,
            public_key,
            key_id,
        })
    }

    /// Create key pair from existing private key
    pub fn from_private_key(private_key: PrivateKey) -> Result<Self> {
        let public_key = private_key.public_key()?;
        let key_id = Self::compute_key_id(&public_key);

        Ok(Self {
            private_key,
            public_key,
            key_id,
        })
    }

    /// Compute unique identifier for a public key
    fn compute_key_id(public_key: &PublicKey) -> Blake2bHash {
        hash_data(public_key.as_bytes())
    }

    /// Get the public key component
    pub fn public(&self) -> &PublicKey {
        &self.public_key
    }

    /// Get the private key component (use carefully!)
    pub fn private(&self) -> &PrivateKey {
        &self.private_key
    }

    /// Sign a message with this key pair
    pub fn sign(&self, message: &[u8]) -> Result<super::Signature> {
        let message_hash = hash_data(message);
        self.private_key.sign(&message_hash)
    }

    /// Verify a signature with this key pair's public key
    pub fn verify(&self, signature: &super::Signature, message: &[u8]) -> bool {
        let message_hash = hash_data(message);
        self.public_key.verify(signature, &message_hash)
    }
}

/// Validator key information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorKey {
    /// Validator address/identifier
    pub validator_address: Blake2bHash,
    /// BLS signing key for consensus
    pub signing_key: CompressedPublicKey,
    /// Ed25519 voting key for Tendermint consensus
    pub voting_key: Vec<u8>, // 32 bytes for Ed25519
    /// Reward address for this validator
    pub reward_address: Blake2bHash,
    /// Key activation epoch
    pub active_from_epoch: u32,
    /// Key deactivation epoch (if any)
    pub inactive_from_epoch: Option<u32>,
}

impl ValidatorKey {
    /// Create new validator key info
    pub fn new(
        validator_address: Blake2bHash,
        signing_key: CompressedPublicKey,
        voting_key: Vec<u8>,
        reward_address: Blake2bHash,
        active_from_epoch: u32,
    ) -> Result<Self> {
        // Validate Ed25519 voting key size
        if voting_key.len() != 32 {
            return Err(CryptoError::InvalidPublicKey);
        }

        Ok(Self {
            validator_address,
            signing_key,
            voting_key,
            reward_address,
            active_from_epoch,
            inactive_from_epoch: None,
        })
    }

    /// Check if key is active at given epoch
    pub fn is_active_at_epoch(&self, epoch: u32) -> bool {
        if epoch < self.active_from_epoch {
            return false;
        }

        if let Some(inactive_epoch) = self.inactive_from_epoch {
            if epoch >= inactive_epoch {
                return false;
            }
        }

        true
    }

    /// Deactivate key at given epoch
    pub fn deactivate_at_epoch(&mut self, epoch: u32) {
        self.inactive_from_epoch = Some(epoch);
    }
}

/// Network operator key information for SP consortium
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkOperatorKey {
    /// Network operator identifier (e.g., "T-Mobile-DE", "Vodafone-UK")
    pub network_id: String,
    /// Primary signing key for CDR transactions
    pub primary_key: CompressedPublicKey,
    /// Backup signing key for failover
    pub backup_key: Option<CompressedPublicKey>,
    /// Settlement signing key for financial transactions
    pub settlement_key: CompressedPublicKey,
    /// Country code for roaming regulations
    pub country_code: String,
    /// Key creation timestamp
    pub created_at: u64,
    /// Key expiration timestamp
    pub expires_at: Option<u64>,
}

impl NetworkOperatorKey {
    /// Create new network operator key
    pub fn new(
        network_id: String,
        primary_key: CompressedPublicKey,
        settlement_key: CompressedPublicKey,
        country_code: String,
    ) -> Self {
        Self {
            network_id,
            primary_key,
            backup_key: None,
            settlement_key,
            country_code,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            expires_at: None,
        }
    }

    /// Check if key is currently valid
    pub fn is_valid(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            if now >= expires_at {
                return false;
            }
        }

        true
    }

    /// Set backup key
    pub fn set_backup_key(&mut self, backup_key: CompressedPublicKey) {
        self.backup_key = Some(backup_key);
    }

    /// Set expiration time
    pub fn set_expiration(&mut self, expires_at: u64) {
        self.expires_at = Some(expires_at);
    }
}

/// Key management system for the SP CDR reconciliation blockchain
#[derive(Debug)]
pub struct KeyManager {
    /// Validator keys indexed by validator address
    validator_keys: HashMap<Blake2bHash, ValidatorKey>,
    /// Network operator keys indexed by network ID
    network_operator_keys: HashMap<String, NetworkOperatorKey>,
    /// Current epoch for key validation
    current_epoch: u32,
}

impl KeyManager {
    /// Create new key manager
    pub fn new() -> Self {
        Self {
            validator_keys: HashMap::new(),
            network_operator_keys: HashMap::new(),
            current_epoch: 0,
        }
    }

    /// Add validator key
    pub fn add_validator_key(&mut self, validator_key: ValidatorKey) {
        self.validator_keys.insert(validator_key.validator_address.clone(), validator_key);
    }

    /// Get validator key by address
    pub fn get_validator_key(&self, address: &Blake2bHash) -> Option<&ValidatorKey> {
        self.validator_keys.get(address)
    }

    /// Get active validator keys for current epoch
    pub fn get_active_validator_keys(&self) -> Vec<&ValidatorKey> {
        self.validator_keys
            .values()
            .filter(|key| key.is_active_at_epoch(self.current_epoch))
            .collect()
    }

    /// Add network operator key
    pub fn add_network_operator_key(&mut self, operator_key: NetworkOperatorKey) {
        self.network_operator_keys.insert(operator_key.network_id.clone(), operator_key);
    }

    /// Get network operator key by network ID
    pub fn get_network_operator_key(&self, network_id: &str) -> Option<&NetworkOperatorKey> {
        self.network_operator_keys.get(network_id)
    }

    /// Get all valid network operator keys
    pub fn get_valid_network_operator_keys(&self) -> Vec<&NetworkOperatorKey> {
        self.network_operator_keys
            .values()
            .filter(|key| key.is_valid())
            .collect()
    }

    /// Update current epoch
    pub fn update_epoch(&mut self, epoch: u32) {
        self.current_epoch = epoch;
    }

    /// Rotate validator key
    pub fn rotate_validator_key(
        &mut self, 
        validator_address: &Blake2bHash,
        new_key: ValidatorKey,
        deactivate_at_epoch: u32,
    ) -> Result<()> {
        // Deactivate old key
        if let Some(old_key) = self.validator_keys.get_mut(validator_address) {
            old_key.deactivate_at_epoch(deactivate_at_epoch);
        }

        // Add new key
        self.add_validator_key(new_key);
        Ok(())
    }

    /// Rotate network operator key
    pub fn rotate_network_operator_key(
        &mut self,
        network_id: &str,
        new_key: NetworkOperatorKey,
        old_key_expiration: u64,
    ) -> Result<()> {
        // Set expiration on old key
        if let Some(old_key) = self.network_operator_keys.get_mut(network_id) {
            old_key.set_expiration(old_key_expiration);
        }

        // Add new key
        self.add_network_operator_key(new_key);
        Ok(())
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = KeyPair::generate().unwrap();
        
        assert_eq!(keypair.private_key.as_bytes().len(), 32);
        assert_eq!(keypair.public_key.as_bytes().len(), 48);
        assert_ne!(keypair.key_id, Blake2bHash::zero());
    }

    #[test]
    fn test_validator_key_lifecycle() {
        let keypair = KeyPair::generate().unwrap();
        let compressed_public = keypair.public_key.compress();
        let voting_key = vec![1u8; 32];
        
        let mut validator_key = ValidatorKey::new(
            hash_data(b"validator_1"),
            compressed_public,
            voting_key,
            hash_data(b"reward_addr"),
            10, // Active from epoch 10
        ).unwrap();

        // Should not be active before epoch 10
        assert!(!validator_key.is_active_at_epoch(5));
        
        // Should be active at epoch 10 and later
        assert!(validator_key.is_active_at_epoch(10));
        assert!(validator_key.is_active_at_epoch(20));
        
        // Deactivate at epoch 30
        validator_key.deactivate_at_epoch(30);
        
        // Should be active before deactivation
        assert!(validator_key.is_active_at_epoch(25));
        
        // Should not be active after deactivation
        assert!(!validator_key.is_active_at_epoch(30));
        assert!(!validator_key.is_active_at_epoch(35));
    }

    #[test]
    fn test_network_operator_key() {
        let keypair1 = KeyPair::generate().unwrap();
        let keypair2 = KeyPair::generate().unwrap();
        
        let mut network_key = NetworkOperatorKey::new(
            "T-Mobile-DE".to_string(),
            keypair1.public_key.compress(),
            keypair2.public_key.compress(),
            "DE".to_string(),
        );

        assert!(network_key.is_valid());
        assert_eq!(network_key.network_id, "T-Mobile-DE");
        assert_eq!(network_key.country_code, "DE");
        
        // Set expiration in the past
        let past_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() - 3600; // 1 hour ago
            
        network_key.set_expiration(past_time);
        assert!(!network_key.is_valid());
    }

    #[test]
    fn test_key_manager() {
        let mut key_manager = KeyManager::new();
        
        // Add validator key
        let keypair = KeyPair::generate().unwrap();
        let validator_address = hash_data(b"validator_test");
        let validator_key = ValidatorKey::new(
            validator_address.clone(),
            keypair.public_key.compress(),
            vec![1u8; 32],
            hash_data(b"reward"),
            0,
        ).unwrap();
        
        key_manager.add_validator_key(validator_key);
        
        // Should be able to retrieve it
        let retrieved = key_manager.get_validator_key(&validator_address);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().validator_address, validator_address);
        
        // Add network operator key
        let operator_keypair = KeyPair::generate().unwrap();
        let settlement_keypair = KeyPair::generate().unwrap();
        let network_key = NetworkOperatorKey::new(
            "Vodafone-UK".to_string(),
            operator_keypair.public_key.compress(),
            settlement_keypair.public_key.compress(),
            "GB".to_string(),
        );
        
        key_manager.add_network_operator_key(network_key);
        
        // Should be able to retrieve it
        let retrieved_network = key_manager.get_network_operator_key("Vodafone-UK");
        assert!(retrieved_network.is_some());
        assert_eq!(retrieved_network.unwrap().country_code, "GB");
    }

    #[test]
    fn test_key_rotation() {
        let mut key_manager = KeyManager::new();
        
        // Add initial validator key
        let keypair1 = KeyPair::generate().unwrap();
        let validator_address = hash_data(b"validator_rotation_test");
        let validator_key1 = ValidatorKey::new(
            validator_address.clone(),
            keypair1.public_key.compress(),
            vec![1u8; 32],
            hash_data(b"reward"),
            0,
        ).unwrap();
        
        key_manager.add_validator_key(validator_key1);
        key_manager.update_epoch(5);
        
        // Should have active key
        let active_keys = key_manager.get_active_validator_keys();
        assert_eq!(active_keys.len(), 1);
        
        // Rotate to new key
        let keypair2 = KeyPair::generate().unwrap();
        let validator_key2 = ValidatorKey::new(
            validator_address.clone(),
            keypair2.public_key.compress(),
            vec![2u8; 32],
            hash_data(b"reward"),
            10, // New key active from epoch 10
        ).unwrap();
        
        key_manager.rotate_validator_key(&validator_address, validator_key2, 10).unwrap();
        
        // At epoch 9, old key should still be active
        key_manager.update_epoch(9);
        let active_keys = key_manager.get_active_validator_keys();
        assert_eq!(active_keys.len(), 1);
        
        // At epoch 10, new key should be active (old key deactivated)
        key_manager.update_epoch(10);
        let active_keys = key_manager.get_active_validator_keys();
        assert_eq!(active_keys.len(), 1);
    }
}