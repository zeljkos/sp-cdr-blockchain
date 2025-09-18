// Signature schemes for SP CDR reconciliation blockchain
// Multi-signature and threshold signature support for validator consensus

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::primitives::{Blake2bHash, hash_data};
use super::{
    PublicKey, Signature, AggregateSignature, AggregatePublicKey,
    CryptoError, Result
};

/// Multi-signature threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    /// Minimum number of signatures required
    pub threshold: usize,
    /// Total number of possible signers
    pub total_signers: usize,
}

impl ThresholdConfig {
    /// Create new threshold configuration
    pub fn new(threshold: usize, total_signers: usize) -> Result<Self> {
        if threshold == 0 || threshold > total_signers {
            return Err(CryptoError::VerificationFailed(
                "Invalid threshold configuration".to_string()
            ));
        }

        Ok(Self {
            threshold,
            total_signers,
        })
    }

    /// Check if number of signatures meets threshold
    pub fn meets_threshold(&self, signature_count: usize) -> bool {
        signature_count >= self.threshold
    }

    /// Calculate percentage of total signers required
    pub fn threshold_percentage(&self) -> f64 {
        (self.threshold as f64 / self.total_signers as f64) * 100.0
    }
}

/// Multi-signature data for validator consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSignature {
    /// The aggregate signature
    pub signature: AggregateSignature,
    /// Bitmap indicating which validators signed
    pub signer_bitmap: Vec<u8>,
    /// Number of signers
    pub signer_count: usize,
    /// The message that was signed
    pub message_hash: Blake2bHash,
}

impl MultiSignature {
    /// Create multi-signature from individual signatures
    pub fn create(
        signatures: &[(usize, Signature)], // (validator_index, signature)
        message: &[u8],
        total_validators: usize,
    ) -> Result<Self> {
        if signatures.is_empty() {
            return Err(CryptoError::AggregationFailed(
                "No signatures to aggregate".to_string()
            ));
        }

        let message_hash = hash_data(message);
        
        // Extract just the signatures for aggregation
        let sigs: Vec<Signature> = signatures.iter().map(|(_, sig)| sig.clone()).collect();
        let signature = AggregateSignature::aggregate(&sigs)?;

        // Create signer bitmap
        let mut signer_bitmap = vec![0u8; (total_validators + 7) / 8]; // Round up to byte boundary
        for &(validator_index, _) in signatures {
            if validator_index >= total_validators {
                return Err(CryptoError::AggregationFailed(
                    format!("Validator index {} out of bounds", validator_index)
                ));
            }
            
            let byte_index = validator_index / 8;
            let bit_index = validator_index % 8;
            signer_bitmap[byte_index] |= 1u8 << bit_index;
        }

        Ok(Self {
            signature,
            signer_bitmap,
            signer_count: signatures.len(),
            message_hash,
        })
    }

    /// Verify multi-signature against validator public keys
    pub fn verify(
        &self,
        validator_public_keys: &[PublicKey],
        message: &[u8],
        threshold_config: &ThresholdConfig,
    ) -> Result<bool> {
        let message_hash = hash_data(message);
        
        // Verify message hash matches
        if message_hash != self.message_hash {
            return Ok(false);
        }

        // Check threshold
        if !threshold_config.meets_threshold(self.signer_count) {
            return Ok(false);
        }

        // Extract signer public keys based on bitmap
        let mut signer_public_keys = Vec::new();
        for (validator_index, public_key) in validator_public_keys.iter().enumerate() {
            if self.is_signer(validator_index) {
                signer_public_keys.push(public_key.clone());
            }
        }

        // Verify we have the expected number of signers
        if signer_public_keys.len() != self.signer_count {
            return Ok(false);
        }

        // Create aggregate public key and verify
        let agg_public_key = AggregatePublicKey::aggregate(&signer_public_keys)?;
        Ok(agg_public_key.verify(&self.signature, &self.message_hash))
    }

    /// Check if validator at given index signed
    pub fn is_signer(&self, validator_index: usize) -> bool {
        let byte_index = validator_index / 8;
        let bit_index = validator_index % 8;
        
        if byte_index >= self.signer_bitmap.len() {
            return false;
        }
        
        (self.signer_bitmap[byte_index] & (1u8 << bit_index)) != 0
    }

    /// Get list of validator indices that signed
    pub fn get_signers(&self) -> Vec<usize> {
        let mut signers = Vec::new();
        
        for byte_index in 0..self.signer_bitmap.len() {
            for bit_index in 0..8 {
                let validator_index = byte_index * 8 + bit_index;
                if self.is_signer(validator_index) {
                    signers.push(validator_index);
                }
            }
        }
        
        signers
    }
}

/// Network operator signature for CDR transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSignature {
    /// The network operator that signed
    pub network_id: String,
    /// The signature
    pub signature: Signature,
    /// Public key used for signing (for verification)
    pub public_key: PublicKey,
    /// Timestamp when signature was created
    pub timestamp: u64,
}

impl NetworkSignature {
    /// Create network signature
    pub fn create(
        network_id: String,
        signature: Signature,
        public_key: PublicKey,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            network_id,
            signature,
            public_key,
            timestamp,
        }
    }

    /// Verify network signature
    pub fn verify(&self, message: &[u8]) -> bool {
        self.public_key.verify(&self.signature, hash_data(message).as_bytes())
    }

    /// Check if signature is within valid time window
    pub fn is_within_time_window(&self, max_age_seconds: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        now.saturating_sub(self.timestamp) <= max_age_seconds
    }
}

/// Settlement signature for inter-network settlements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementSignature {
    /// Creditor network signature
    pub creditor_signature: NetworkSignature,
    /// Debtor network signature  
    pub debtor_signature: NetworkSignature,
    /// Settlement amount agreed upon
    pub settlement_amount: u64,
    /// Settlement period
    pub period: String,
    /// Settlement transaction hash
    pub settlement_hash: Blake2bHash,
}

impl SettlementSignature {
    /// Create settlement signature from both network signatures
    pub fn create(
        creditor_signature: NetworkSignature,
        debtor_signature: NetworkSignature,
        settlement_amount: u64,
        period: String,
    ) -> Result<Self> {
        // Compute settlement hash from all components
        let settlement_data = format!(
            "{}:{}:{}:{}:{}",
            creditor_signature.network_id,
            debtor_signature.network_id,
            settlement_amount,
            period,
            creditor_signature.timestamp
        );
        let settlement_hash = hash_data(settlement_data.as_bytes());

        Ok(Self {
            creditor_signature,
            debtor_signature,
            settlement_amount,
            period,
            settlement_hash,
        })
    }

    /// Verify both signatures in the settlement
    pub fn verify(&self, settlement_message: &[u8]) -> bool {
        self.creditor_signature.verify(settlement_message) &&
        self.debtor_signature.verify(settlement_message)
    }

    /// Verify settlement integrity
    pub fn verify_settlement_integrity(&self) -> bool {
        // Verify networks are different
        if self.creditor_signature.network_id == self.debtor_signature.network_id {
            return false;
        }

        // Verify signatures are within reasonable time window (1 hour)
        let max_age = 3600;
        if !self.creditor_signature.is_within_time_window(max_age) ||
           !self.debtor_signature.is_within_time_window(max_age) {
            return false;
        }

        // Verify settlement hash
        let settlement_data = format!(
            "{}:{}:{}:{}:{}",
            self.creditor_signature.network_id,
            self.debtor_signature.network_id,
            self.settlement_amount,
            self.period,
            self.creditor_signature.timestamp
        );
        let computed_hash = hash_data(settlement_data.as_bytes());
        
        computed_hash == self.settlement_hash
    }
}

/// Signature manager for coordinating multi-signatures
#[derive(Debug)]
pub struct SignatureManager {
    /// Pending multi-signatures indexed by message hash
    pending_multisigs: HashMap<Blake2bHash, Vec<(usize, Signature)>>,
    /// Threshold configurations for different message types
    threshold_configs: HashMap<String, ThresholdConfig>,
}

impl SignatureManager {
    /// Create new signature manager
    pub fn new() -> Self {
        Self {
            pending_multisigs: HashMap::new(),
            threshold_configs: HashMap::new(),
        }
    }

    /// Set threshold configuration for message type
    pub fn set_threshold_config(&mut self, message_type: String, config: ThresholdConfig) {
        self.threshold_configs.insert(message_type, config);
    }

    /// Add signature to pending multi-signature
    pub fn add_signature(
        &mut self,
        message: &[u8],
        validator_index: usize,
        signature: Signature,
    ) -> Result<()> {
        let message_hash = hash_data(message);
        
        let signatures = self.pending_multisigs.entry(message_hash).or_insert_with(Vec::new);
        
        // Check if validator already signed
        for (existing_index, _) in signatures.iter() {
            if *existing_index == validator_index {
                return Err(CryptoError::AggregationFailed(
                    "Validator already signed this message".to_string()
                ));
            }
        }
        
        signatures.push((validator_index, signature));
        Ok(())
    }

    /// Try to create multi-signature if threshold is met
    pub fn try_create_multisig(
        &mut self,
        message: &[u8],
        message_type: &str,
        total_validators: usize,
    ) -> Result<Option<MultiSignature>> {
        let message_hash = hash_data(message);
        
        let threshold_config = self.threshold_configs.get(message_type)
            .ok_or_else(|| CryptoError::VerificationFailed(
                format!("No threshold config for message type: {}", message_type)
            ))?;

        if let Some(signatures) = self.pending_multisigs.get(&message_hash) {
            if threshold_config.meets_threshold(signatures.len()) {
                let multisig = MultiSignature::create(signatures, message, total_validators)?;
                
                // Remove from pending once created
                self.pending_multisigs.remove(&message_hash);
                
                return Ok(Some(multisig));
            }
        }
        
        Ok(None)
    }

    /// Get number of signatures for a message
    pub fn get_signature_count(&self, message: &[u8]) -> usize {
        let message_hash = hash_data(message);
        self.pending_multisigs.get(&message_hash)
            .map(|sigs| sigs.len())
            .unwrap_or(0)
    }

    /// Clear old pending signatures (cleanup)
    pub fn cleanup_old_signatures(&mut self, message_hashes_to_remove: &[Blake2bHash]) {
        for hash in message_hashes_to_remove {
            self.pending_multisigs.remove(hash);
        }
    }
}

impl Default for SignatureManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{PrivateKey, KeyPair};

    #[test]
    fn test_threshold_config() {
        let config = ThresholdConfig::new(3, 5).unwrap();
        assert_eq!(config.threshold, 3);
        assert_eq!(config.total_signers, 5);
        assert_eq!(config.threshold_percentage(), 60.0);
        
        assert!(!config.meets_threshold(2));
        assert!(config.meets_threshold(3));
        assert!(config.meets_threshold(4));
        
        // Invalid configs
        assert!(ThresholdConfig::new(0, 5).is_err());
        assert!(ThresholdConfig::new(6, 5).is_err());
    }

    #[test]
    fn test_multi_signature_creation() {
        let message = b"SP CDR multi-signature test";
        
        // Create validator key pairs
        let keypair1 = KeyPair::generate().unwrap();
        let keypair2 = KeyPair::generate().unwrap();
        let keypair3 = KeyPair::generate().unwrap();
        
        // Create signatures
        let sig1 = keypair1.sign(message).unwrap();
        let sig2 = keypair2.sign(message).unwrap();
        let sig3 = keypair3.sign(message).unwrap();
        
        let signatures = vec![(0, sig1), (1, sig2), (2, sig3)];
        let multisig = MultiSignature::create(&signatures, message, 5).unwrap();
        
        assert_eq!(multisig.signer_count, 3);
        assert!(multisig.is_signer(0));
        assert!(multisig.is_signer(1));
        assert!(multisig.is_signer(2));
        assert!(!multisig.is_signer(3));
        assert!(!multisig.is_signer(4));
        
        let signers = multisig.get_signers();
        assert_eq!(signers, vec![0, 1, 2]);
    }

    #[test]
    fn test_multi_signature_verification() {
        let message = b"SP CDR verification test";
        
        // Create validator key pairs
        let keypairs: Vec<_> = (0..5).map(|_| KeyPair::generate().unwrap()).collect();
        let public_keys: Vec<_> = keypairs.iter().map(|kp| kp.public_key.clone()).collect();
        
        // Create signatures from first 3 validators
        let signatures: Vec<_> = keypairs[0..3]
            .iter()
            .enumerate()
            .map(|(i, kp)| (i, kp.sign(message).unwrap()))
            .collect();
            
        let multisig = MultiSignature::create(&signatures, message, 5).unwrap();
        
        // Create threshold config (3 out of 5)
        let threshold_config = ThresholdConfig::new(3, 5).unwrap();
        
        // Should verify successfully
        assert!(multisig.verify(&public_keys, message, &threshold_config).unwrap());
        
        // Wrong message should fail
        assert!(!multisig.verify(&public_keys, b"wrong message", &threshold_config).unwrap());
        
        // Threshold config requiring 4 signatures should fail
        let strict_threshold = ThresholdConfig::new(4, 5).unwrap();
        assert!(!multisig.verify(&public_keys, message, &strict_threshold).unwrap());
    }

    #[test]
    fn test_network_signature() {
        let keypair = KeyPair::generate().unwrap();
        let message = b"CDR transaction data";
        let signature = keypair.sign(message).unwrap();
        
        let network_sig = NetworkSignature::create(
            "T-Mobile-DE".to_string(),
            signature,
            keypair.public_key.clone(),
        );
        
        assert_eq!(network_sig.network_id, "T-Mobile-DE");
        assert!(network_sig.verify(message));
        assert!(!network_sig.verify(b"different message"));
        assert!(network_sig.is_within_time_window(3600)); // 1 hour
    }

    #[test]
    fn test_settlement_signature() {
        let creditor_keypair = KeyPair::generate().unwrap();
        let debtor_keypair = KeyPair::generate().unwrap();
        let settlement_message = b"settlement_data_123";
        
        let creditor_sig = NetworkSignature::create(
            "Vodafone-UK".to_string(),
            creditor_keypair.sign(settlement_message).unwrap(),
            creditor_keypair.public_key.clone(),
        );
        
        let debtor_sig = NetworkSignature::create(
            "T-Mobile-DE".to_string(),
            debtor_keypair.sign(settlement_message).unwrap(),
            debtor_keypair.public_key.clone(),
        );
        
        let settlement = SettlementSignature::create(
            creditor_sig,
            debtor_sig,
            125000, // €1,250.00
            "2024-01-15-daily".to_string(),
        ).unwrap();
        
        assert_eq!(settlement.creditor_signature.network_id, "Vodafone-UK");
        assert_eq!(settlement.debtor_signature.network_id, "T-Mobile-DE");
        assert_eq!(settlement.settlement_amount, 125000);
        assert!(settlement.verify(settlement_message));
        assert!(settlement.verify_settlement_integrity());
    }

    #[test]
    fn test_signature_manager() {
        let mut sig_manager = SignatureManager::new();
        let message = b"consensus message";
        
        // Set threshold config
        let threshold_config = ThresholdConfig::new(2, 3).unwrap();
        sig_manager.set_threshold_config("consensus".to_string(), threshold_config);
        
        // Add signatures
        let keypair1 = KeyPair::generate().unwrap();
        let keypair2 = KeyPair::generate().unwrap();
        
        let sig1 = keypair1.sign(message).unwrap();
        let sig2 = keypair2.sign(message).unwrap();
        
        sig_manager.add_signature(message, 0, sig1).unwrap();
        assert_eq!(sig_manager.get_signature_count(message), 1);
        
        // Should not have enough signatures yet
        let multisig = sig_manager.try_create_multisig(message, "consensus", 3).unwrap();
        assert!(multisig.is_none());
        
        // Add second signature
        sig_manager.add_signature(message, 1, sig2).unwrap();
        assert_eq!(sig_manager.get_signature_count(message), 2);
        
        // Now should have enough signatures
        let multisig = sig_manager.try_create_multisig(message, "consensus", 3).unwrap();
        assert!(multisig.is_some());
        
        // Should be removed from pending after creation
        assert_eq!(sig_manager.get_signature_count(message), 0);
    }
}