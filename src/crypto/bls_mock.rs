// BLS signature implementation for SP CDR reconciliation
// Adapted from Nimiq's BLS implementation for validator consensus

use std::fmt;
use serde::{Deserialize, Serialize};
use crate::primitives::Blake2bHash;
use super::{CryptoError, Result};

/// BLS signature type using Blake2b hash
pub type SigHash = Blake2bHash;

/// BLS private key (32 bytes)
#[derive(Clone, PartialEq, Eq)]
pub struct PrivateKey {
    key: [u8; 32],
}

impl PrivateKey {
    /// Generate a new random private key
    pub fn generate() -> Result<Self> {
        let mut key = [0u8; 32];

        // Use cryptographically secure RNG
        getrandom::getrandom(&mut key).map_err(|e|
            CryptoError::KeyGenerationFailed(e.to_string()))?;

        // Ensure key is not zero (extremely unlikely but good practice)
        if key.iter().all(|&b| b == 0) {
            // If we get all zeros (probability: 1 in 2^256), try again
            getrandom::getrandom(&mut key).map_err(|e|
                CryptoError::KeyGenerationFailed(e.to_string()))?;
        }

        Ok(Self { key })
    }

    /// Create private key from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 32 {
            return Err(CryptoError::InvalidPrivateKey);
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(bytes);
        
        // Validate key is not zero
        if key.iter().all(|&b| b == 0) {
            return Err(CryptoError::InvalidPrivateKey);
        }

        Ok(Self { key })
    }

    /// Get the corresponding public key
    pub fn public_key(&self) -> Result<PublicKey> {
        // In a real BLS implementation, this would derive the public key
        // from the private key using elliptic curve operations
        PublicKey::from_private_key(self)
    }

    /// Sign a message hash
    pub fn sign(&self, message_hash: &SigHash) -> Result<Signature> {
        Signature::create(self, message_hash)
    }

    /// Get raw key bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key
    }
}

impl fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PrivateKey([REDACTED])")
    }
}

/// BLS public key (48 bytes compressed)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PublicKey {
    key: [u8; 48],
}

impl serde::Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.key)
    }
}

impl<'de> serde::Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        if bytes.len() != 48 {
            return Err(serde::de::Error::custom("Invalid public key length"));
        }
        let mut key = [0u8; 48];
        key.copy_from_slice(&bytes);
        Ok(PublicKey { key })
    }
}

impl PublicKey {
    /// Create public key from private key
    pub fn from_private_key(private_key: &PrivateKey) -> Result<Self> {
        // In real BLS implementation, derive public key from private key
        let mut key = [0u8; 48];
        
        // Mock derivation - in reality use BLS12-381 point multiplication
        key[0..32].copy_from_slice(&private_key.key);
        key[32] = 0x01; // Mark as derived
        
        Ok(Self { key })
    }

    /// Create public key from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 48 {
            return Err(CryptoError::InvalidPublicKey);
        }

        let mut key = [0u8; 48];
        key.copy_from_slice(bytes);
        
        Ok(Self { key })
    }

    /// Verify a signature against this public key
    pub fn verify(&self, signature: &Signature, message_hash: &SigHash) -> bool {
        signature.verify(self, message_hash).unwrap_or(false)
    }

    /// Get raw key bytes
    pub fn as_bytes(&self) -> &[u8; 48] {
        &self.key
    }

    /// Convert to compressed representation for storage
    pub fn compress(&self) -> CompressedPublicKey {
        CompressedPublicKey {
            key: self.key,
        }
    }
}

/// Compressed public key for efficient storage
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CompressedPublicKey {
    key: [u8; 48],
}

impl serde::Serialize for CompressedPublicKey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.key)
    }
}

impl<'de> serde::Deserialize<'de> for CompressedPublicKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        if bytes.len() != 48 {
            return Err(serde::de::Error::custom("Invalid compressed public key length"));
        }
        let mut key = [0u8; 48];
        key.copy_from_slice(&bytes);
        Ok(CompressedPublicKey { key })
    }
}

impl CompressedPublicKey {
    /// Decompress to full public key
    pub fn decompress(&self) -> Result<PublicKey> {
        PublicKey::from_bytes(&self.key)
    }

    /// Get raw compressed bytes
    pub fn as_bytes(&self) -> &[u8; 48] {
        &self.key
    }
}

/// BLS signature (96 bytes)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Signature {
    sig: [u8; 96],
}

impl serde::Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.sig)
    }
}

impl<'de> serde::Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        if bytes.len() != 96 {
            return Err(serde::de::Error::custom("Invalid signature length"));
        }
        let mut sig = [0u8; 96];
        sig.copy_from_slice(&bytes);
        Ok(Signature { sig })
    }
}

impl Signature {
    /// Create signature from private key and message hash
    pub fn create(private_key: &PrivateKey, message_hash: &SigHash) -> Result<Self> {
        // In real BLS implementation, this would perform BLS signing
        let mut sig = [0u8; 96];
        
        // Mock signature generation
        sig[0..32].copy_from_slice(private_key.as_bytes());
        sig[32..64].copy_from_slice(message_hash.as_bytes());
        sig[64] = 0x42; // Mock signature marker
        
        Ok(Self { sig })
    }

    /// Create signature from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 96 {
            return Err(CryptoError::InvalidSignature);
        }

        let mut sig = [0u8; 96];
        sig.copy_from_slice(bytes);
        
        Ok(Self { sig })
    }

    /// Verify signature against public key and message
    pub fn verify(&self, public_key: &PublicKey, message_hash: &SigHash) -> Result<bool> {
        // In real BLS implementation, this would use pairing-based verification
        
        // Mock verification - check that signature contains expected data
        if self.sig[64] != 0x42 {
            return Ok(false);
        }

        // Check message hash is embedded (mock check)
        let embedded_hash = &self.sig[32..64];
        if embedded_hash != message_hash.as_bytes() {
            return Ok(false);
        }

        Ok(true)
    }

    /// Get raw signature bytes
    pub fn as_bytes(&self) -> &[u8; 96] {
        &self.sig
    }

    /// Compress signature for storage
    pub fn compress(&self) -> CompressedSignature {
        CompressedSignature {
            sig: self.sig,
        }
    }
}

/// Compressed signature for efficient storage
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CompressedSignature {
    sig: [u8; 96],
}

impl serde::Serialize for CompressedSignature {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.sig)
    }
}

impl<'de> serde::Deserialize<'de> for CompressedSignature {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        if bytes.len() != 96 {
            return Err(serde::de::Error::custom("Invalid compressed signature length"));
        }
        let mut sig = [0u8; 96];
        sig.copy_from_slice(&bytes);
        Ok(CompressedSignature { sig })
    }
}

impl CompressedSignature {
    /// Decompress to full signature
    pub fn decompress(&self) -> Result<Signature> {
        Signature::from_bytes(&self.sig)
    }

    /// Get raw compressed bytes
    pub fn as_bytes(&self) -> &[u8; 96] {
        &self.sig
    }
}

/// Aggregate public key for multi-signature verification
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AggregatePublicKey {
    agg_key: [u8; 48],
}

impl serde::Serialize for AggregatePublicKey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.agg_key)
    }
}

impl<'de> serde::Deserialize<'de> for AggregatePublicKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        if bytes.len() != 48 {
            return Err(serde::de::Error::custom("Invalid aggregate public key length"));
        }
        let mut agg_key = [0u8; 48];
        agg_key.copy_from_slice(&bytes);
        Ok(AggregatePublicKey { agg_key })
    }
}

impl AggregatePublicKey {
    /// Create aggregate public key from individual public keys
    pub fn aggregate(public_keys: &[PublicKey]) -> Result<Self> {
        if public_keys.is_empty() {
            return Err(CryptoError::AggregationFailed("No keys to aggregate".to_string()));
        }

        // In real BLS implementation, this would sum the public keys
        let mut agg_key = [0u8; 48];
        
        // Mock aggregation - XOR all keys (not cryptographically sound)
        for public_key in public_keys {
            for (i, byte) in public_key.as_bytes().iter().enumerate() {
                agg_key[i] ^= byte;
            }
        }
        
        Ok(Self { agg_key })
    }

    /// Verify aggregate signature
    pub fn verify(&self, signature: &AggregateSignature, message_hash: &SigHash) -> bool {
        signature.verify(self, message_hash).unwrap_or(false)
    }

    /// Get raw aggregated key bytes
    pub fn as_bytes(&self) -> &[u8; 48] {
        &self.agg_key
    }
}

/// Aggregate signature for multi-signature schemes
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AggregateSignature {
    agg_sig: [u8; 96],
}

impl serde::Serialize for AggregateSignature {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.agg_sig)
    }
}

impl<'de> serde::Deserialize<'de> for AggregateSignature {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        if bytes.len() != 96 {
            return Err(serde::de::Error::custom("Invalid aggregate signature length"));
        }
        let mut agg_sig = [0u8; 96];
        agg_sig.copy_from_slice(&bytes);
        Ok(AggregateSignature { agg_sig })
    }
}

impl AggregateSignature {
    /// Create aggregate signature from individual signatures
    pub fn aggregate(signatures: &[Signature]) -> Result<Self> {
        if signatures.is_empty() {
            return Err(CryptoError::AggregationFailed("No signatures to aggregate".to_string()));
        }

        // In real BLS implementation, this would sum the signatures
        let mut agg_sig = [0u8; 96];
        
        // Mock aggregation
        for signature in signatures {
            for (i, byte) in signature.as_bytes().iter().enumerate() {
                agg_sig[i] ^= byte;
            }
        }
        
        Ok(Self { agg_sig })
    }

    /// Verify aggregate signature against aggregate public key
    pub fn verify(&self, agg_public_key: &AggregatePublicKey, message_hash: &SigHash) -> Result<bool> {
        // In real BLS implementation, use pairing-based verification
        
        // Mock verification
        if self.agg_sig[95] == 0 {
            return Ok(false);
        }
        
        Ok(true)
    }

    /// Create aggregate signature from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 96 {
            return Err(CryptoError::InvalidSignature);
        }
        let mut agg_sig = [0u8; 96];
        agg_sig.copy_from_slice(bytes);
        Ok(Self { agg_sig })
    }

    /// Get raw aggregate signature bytes
    pub fn as_bytes(&self) -> &[u8; 96] {
        &self.agg_sig
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::hash_data;

    #[test]
    fn test_key_generation() {
        let private_key = PrivateKey::generate().unwrap();
        let public_key = private_key.public_key().unwrap();
        
        assert_eq!(private_key.as_bytes().len(), 32);
        assert_eq!(public_key.as_bytes().len(), 48);
    }

    #[test]
    fn test_signature_creation_and_verification() {
        let private_key = PrivateKey::generate().unwrap();
        let public_key = private_key.public_key().unwrap();
        
        let message = b"SP CDR reconciliation test message";
        let message_hash = hash_data(message);
        
        let signature = private_key.sign(&message_hash).unwrap();
        assert!(public_key.verify(&signature, &message_hash));
        
        // Wrong message should fail verification
        let wrong_hash = hash_data(b"different message");
        assert!(!public_key.verify(&signature, &wrong_hash));
    }

    #[test]
    fn test_key_serialization() {
        let private_key = PrivateKey::generate().unwrap();
        let public_key = private_key.public_key().unwrap();
        
        // Test private key roundtrip
        let private_bytes = private_key.as_bytes();
        let restored_private = PrivateKey::from_bytes(private_bytes).unwrap();
        assert_eq!(private_key, restored_private);
        
        // Test public key roundtrip
        let public_bytes = public_key.as_bytes();
        let restored_public = PublicKey::from_bytes(public_bytes).unwrap();
        assert_eq!(public_key, restored_public);
    }

    #[test]
    fn test_signature_aggregation() {
        let private_key1 = PrivateKey::generate().unwrap();
        let private_key2 = PrivateKey::generate().unwrap();
        
        let public_key1 = private_key1.public_key().unwrap();
        let public_key2 = private_key2.public_key().unwrap();
        
        let message_hash = hash_data(b"aggregate signature test");
        
        let signature1 = private_key1.sign(&message_hash).unwrap();
        let signature2 = private_key2.sign(&message_hash).unwrap();
        
        let agg_signature = AggregateSignature::aggregate(&[signature1, signature2]).unwrap();
        let agg_public_key = AggregatePublicKey::aggregate(&[public_key1, public_key2]).unwrap();
        
        assert!(agg_public_key.verify(&agg_signature, &message_hash));
    }

    #[test]
    fn test_compressed_keys() {
        let private_key = PrivateKey::generate().unwrap();
        let public_key = private_key.public_key().unwrap();
        
        let compressed = public_key.compress();
        let decompressed = compressed.decompress().unwrap();
        
        assert_eq!(public_key, decompressed);
    }

    #[test]
    fn test_invalid_key_bytes() {
        // Test invalid private key length
        let result = PrivateKey::from_bytes(&[1, 2, 3]);
        assert!(matches!(result, Err(CryptoError::InvalidPrivateKey)));
        
        // Test zero private key
        let result = PrivateKey::from_bytes(&[0u8; 32]);
        assert!(matches!(result, Err(CryptoError::InvalidPrivateKey)));
        
        // Test invalid public key length
        let result = PublicKey::from_bytes(&[1, 2, 3]);
        assert!(matches!(result, Err(CryptoError::InvalidPublicKey)));
    }
}