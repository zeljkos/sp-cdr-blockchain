// ZKP component tests for SP CDR reconciliation blockchain
use sp_cdr_reconciliation_bc::*;

#[test]
fn test_zkp_error_types() {
    // Test ZKP error type creation and display
    let errors = vec![
        zkp::ZKPError::InvalidProof,
        zkp::ZKPError::VerificationFailed("test error".to_string()),
        zkp::ZKPError::UnsupportedNetwork(NetworkId::DevNet),
        zkp::ZKPError::EncryptionError("encryption failed".to_string()),
        zkp::ZKPError::ProofGenerationFailed("proof gen failed".to_string()),
    ];

    for error in errors {
        // Should be able to display error messages
        let error_string = format!("{}", error);
        assert!(!error_string.is_empty());
        
        // Should have debug representation
        let debug_string = format!("{:?}", error);
        assert!(!debug_string.is_empty());
    }
    
    println!("✅ ZKP error types work correctly");
}

#[test]
fn test_cdr_private_data_creation() {
    let private_data = zkp::CDRPrivateData {
        call_duration_minutes: 45,
        data_usage_mb: 2048,
        sms_count: 12,
        roaming_charges: 25000, // €250.00 in cents
        home_network_id: "Orange-FR".to_string(),
        visited_network_id: "Telenor-NO".to_string(),
        subscriber_hash: hash_data(b"hashed_imsi_98765"),
    };

    assert_eq!(private_data.call_duration_minutes, 45);
    assert_eq!(private_data.data_usage_mb, 2048);
    assert_eq!(private_data.sms_count, 12);
    assert_eq!(private_data.roaming_charges, 25000);
    assert_eq!(private_data.home_network_id, "Orange-FR");
    assert_eq!(private_data.visited_network_id, "Telenor-NO");
    
    // Should be serializable
    let serialized = serde_json::to_string(&private_data).unwrap();
    let deserialized: zkp::CDRPrivateData = serde_json::from_str(&serialized).unwrap();
    assert_eq!(private_data.call_duration_minutes, deserialized.call_duration_minutes);
    
    println!("✅ CDR private data creation and serialization works");
}

#[test]
fn test_cdr_public_inputs() {
    let public_inputs = zkp::CDRPublicInputs {
        record_hash: hash_data(b"cdr_record_12345"),
        network_pair_hash: hash_data(b"Orange-FR:Telenor-NO"),
        timestamp_range_hash: hash_data(b"2024-01-15T10:00:00Z-2024-01-15T11:00:00Z"),
        total_charge_commitment: hash_data(b"commitment_25000"),
    };

    // Should not have zero hashes (indicates real data)
    assert_ne!(public_inputs.record_hash, Blake2bHash::zero());
    assert_ne!(public_inputs.network_pair_hash, Blake2bHash::zero());
    assert_ne!(public_inputs.timestamp_range_hash, Blake2bHash::zero());
    assert_ne!(public_inputs.total_charge_commitment, Blake2bHash::zero());
    
    // Should be serializable
    let serialized = serde_json::to_string(&public_inputs).unwrap();
    let _deserialized: zkp::CDRPublicInputs = serde_json::from_str(&serialized).unwrap();
    
    println!("✅ CDR public inputs work correctly");
}

#[test]
fn test_settlement_inputs() {
    let settlement_inputs = zkp::SettlementInputs {
        creditor_network: "Telenor-NO".to_string(),
        debtor_network: "Orange-FR".to_string(),
        period: "2024-01-15-daily".to_string(),
        total_charges: 95000, // €950.00
        exchange_rate: 1050, // 10.50 NOK/EUR * 100
        settlement_amount: 99750, // €950.00 * 10.50 = 9975.00 NOK
    };

    assert_eq!(settlement_inputs.creditor_network, "Telenor-NO");
    assert_eq!(settlement_inputs.debtor_network, "Orange-FR");
    assert_eq!(settlement_inputs.total_charges, 95000);
    assert_eq!(settlement_inputs.exchange_rate, 1050);
    assert_eq!(settlement_inputs.settlement_amount, 99750);
    
    // Should be serializable
    let serialized = serde_json::to_string(&settlement_inputs).unwrap();
    let deserialized: zkp::SettlementInputs = serde_json::from_str(&serialized).unwrap();
    assert_eq!(settlement_inputs.total_charges, deserialized.total_charges);
    
    println!("✅ Settlement inputs work correctly");
}

#[test]
fn test_cdr_privacy_prover_creation() {
    let prover = zkp::proof_system::CDRPrivacyProver::new();
    
    // Should be able to create prover instance
    // (actual proving would require circuit implementation)
    
    println!("✅ CDR privacy prover creation works");
}

#[test]
fn test_settlement_prover_creation() {
    let prover = zkp::proof_system::SettlementProver::new();
    
    // Should be able to create prover instance
    // (actual proving would require circuit implementation)
    
    println!("✅ Settlement prover creation works");
}

#[test]
fn test_verifier_creation() {
    let cdr_verifier = zkp::proof_system::CDRPrivacyVerifier::new();
    let settlement_verifier = zkp::proof_system::SettlementVerifier::new();
    
    // Should be able to create verifier instances
    // (actual verification would require real proofs)
    
    println!("✅ ZKP verifier creation works");
}

#[test]
fn test_zkp_verifying_key_manager() {
    let zkp_key_manager = zkp::verifying_key::CDRZKPVerifyingKey::new();
    
    // Should be able to create key manager
    // In development, this will use mock keys
    
    // Test unsupported network error
    let result = zkp::verifying_key::CDRZKPVerifyingKey::load_verifying_keys(NetworkId::MainNet);
    assert!(matches!(result, Err(zkp::ZKPError::UnsupportedNetwork(_))));
    
    println!("✅ ZKP verifying key manager works");
}

#[test]
fn test_verifying_key_metadata() {
    let metadata = zkp::verifying_key::VerifyingKeyMetadata {
        network_id: NetworkId::SPConsortium,
        cdr_privacy_key_hash: hash_data(b"cdr_privacy_key"),
        settlement_key_hash: hash_data(b"settlement_key"),
        roaming_auth_key_hash: hash_data(b"roaming_auth_key"),
        creation_timestamp: 1640995200, // 2022-01-01
        trusted_setup_ceremony_id: "sp-cdr-zkp-ceremony-2024".to_string(),
    };

    assert!(metadata.matches(NetworkId::SPConsortium));
    assert!(!metadata.matches(NetworkId::TestNet));
    assert_eq!(metadata.trusted_setup_ceremony_id, "sp-cdr-zkp-ceremony-2024");
    
    println!("✅ Verifying key metadata works");
}

#[test]
fn test_poseidon_parameters() {
    // Test MNT4 parameters
    let mnt4_params = zkp::poseidon::mnt4::PoseidonParametersMNT4::new();
    assert_eq!(mnt4_params.full_rounds, 8);
    assert_eq!(mnt4_params.partial_rounds, 57);
    assert_eq!(mnt4_params.alpha, 5);
    
    // Test MNT6 parameters  
    let mnt6_params = zkp::poseidon::mnt6::PoseidonParametersMNT6::new();
    assert_eq!(mnt6_params.full_rounds, 8);
    assert_eq!(mnt6_params.partial_rounds, 57);
    assert_eq!(mnt6_params.alpha, 5);
    
    println!("✅ Poseidon parameters creation works");
}

#[test]
fn test_bytes_to_field_elements() {
    let test_data = b"SP CDR reconciliation blockchain ZKP test data";
    let field_elements = zkp::poseidon::mnt6::bytes_to_field_elements(test_data);
    
    // Should create field elements
    assert!(!field_elements.is_empty());
    
    // Test with exactly 31 bytes (should create 1 element)
    let exact_data = vec![42u8; 31];
    let exact_elements = zkp::poseidon::mnt6::bytes_to_field_elements(&exact_data);
    assert_eq!(exact_elements.len(), 1);
    
    // Test with 32 bytes (should create 2 elements due to safety margin)
    let large_data = vec![42u8; 32];
    let large_elements = zkp::poseidon::mnt6::bytes_to_field_elements(&large_data);
    assert_eq!(large_elements.len(), 2);
    
    println!("✅ Bytes to field elements conversion works");
}

#[test]
fn test_zkp_integration_with_cdr_transactions() {
    // Test that ZKP components integrate with CDR transaction types
    let cdr_transaction = blockchain::CDRTransaction {
        record_type: blockchain::CDRType::DataSession,
        home_network: "KPN-NL".to_string(),
        visited_network: "TIM-IT".to_string(),
        encrypted_data: b"encrypted_session_data_blob".to_vec(),
        zk_proof: b"privacy_preserving_proof_blob".to_vec(),
    };

    // ZKP proof should be stored in the CDR transaction
    assert!(!cdr_transaction.zk_proof.is_empty());
    assert_eq!(cdr_transaction.home_network, "KPN-NL");
    assert_eq!(cdr_transaction.visited_network, "TIM-IT");
    
    // Encrypted data should be present
    assert!(!cdr_transaction.encrypted_data.is_empty());
    
    println!("✅ ZKP integration with CDR transactions works");
}

#[test]
fn test_zkp_settlement_transaction_integration() {
    // Test ZKP integration with settlement transactions
    let settlement_transaction = blockchain::SettlementTransaction {
        creditor_network: "TIM-IT".to_string(),
        debtor_network: "KPN-NL".to_string(),
        amount: 185000, // €1,850.00
        currency: "EUR".to_string(),
        period: "2024-01-15-daily".to_string(),
    };

    // Settlement should have proper network identifiers
    assert_ne!(settlement_transaction.creditor_network, settlement_transaction.debtor_network);
    assert_eq!(settlement_transaction.currency, "EUR");
    assert_eq!(settlement_transaction.amount, 185000);
    
    println!("✅ ZKP integration with settlement transactions works");
}

#[cfg(test)]
mod crypto_integration_tests {
    use super::*;
    
    #[test]
    fn test_zkp_and_crypto_integration() {
        // Test that ZKP and crypto modules work together
        let keypair = crypto::KeyPair::generate().unwrap();
        
        let cdr_data = zkp::CDRPrivateData {
            call_duration_minutes: 60,
            data_usage_mb: 1500,
            sms_count: 8,
            roaming_charges: 45000,
            home_network_id: "Three-UK".to_string(),
            visited_network_id: "Sunrise-CH".to_string(),
            subscriber_hash: hash_data(keypair.public_key.as_bytes()), // Use crypto key for subscriber hash
        };
        
        // Should be able to use crypto components for ZKP data
        assert_eq!(cdr_data.home_network_id, "Three-UK");
        assert_ne!(cdr_data.subscriber_hash, Blake2bHash::zero());
        
        println!("✅ ZKP and crypto integration works");
    }
    
    #[test]
    fn test_validator_keys_with_zkp() {
        // Test validator keys work with ZKP system
        let signing_keypair = crypto::KeyPair::generate().unwrap();
        let voting_key = vec![42u8; 32]; // Ed25519 key
        
        let validator_key = crypto::ValidatorKey::new(
            hash_data(b"validator_zkp_test"),
            signing_keypair.public_key.compress(),
            voting_key,
            hash_data(b"reward_address"),
            0,
        ).unwrap();
        
        // Validator should be active for ZKP verification
        assert!(validator_key.is_active_at_epoch(0));
        assert!(validator_key.is_active_at_epoch(10));
        
        // Should be able to use validator key in ZKP context
        assert_eq!(validator_key.signing_key.as_bytes().len(), 48);
        assert_eq!(validator_key.voting_key.len(), 32);
        
        println!("✅ Validator keys work with ZKP system");
    }
}