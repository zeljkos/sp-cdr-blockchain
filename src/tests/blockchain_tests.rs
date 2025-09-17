// Blockchain core component tests
use sp_cdr_reconciliation_bc::*;

#[test]
fn test_block_creation_and_hashing() {
    // Test block creation following Albatross patterns
    let micro_block = Block::Micro(MicroBlock {
        header: blockchain::MicroHeader {
            network: NetworkId::SPConsortium,
            version: 1,
            block_number: 42,
            timestamp: 1234567890,
            parent_hash: Blake2bHash::zero(),
            seed: Blake2bHash::from_bytes([1u8; 32]),
            extra_data: b"test_data".to_vec(),
            state_root: Blake2bHash::zero(),
            body_root: Blake2bHash::zero(),
            history_root: Blake2bHash::zero(),
        },
        body: blockchain::MicroBody {
            transactions: vec![],
        },
    });
    
    // Test hash consistency
    let hash1 = micro_block.hash();
    let hash2 = micro_block.hash();
    assert_eq!(hash1, hash2);
    
    // Test block properties
    assert_eq!(micro_block.block_number(), 42);
    assert_eq!(micro_block.timestamp(), 1234567890);
    
    println!("✅ Block creation and hashing works");
}

#[test]
fn test_macro_block_validator_updates() {
    // Test macro block with validator set updates
    let validators = vec![
        ValidatorInfo {
            address: Blake2bHash::from_bytes([1u8; 32]),
            signing_key: vec![1u8; 48], // BLS key size
            voting_key: vec![1u8; 32],  // Ed25519 key size
            reward_address: Blake2bHash::from_bytes([2u8; 32]),
            signal_data: Some(b"validator_signal".to_vec()),
            inactive_from: None,
            jailed_from: None,
        }
    ];
    
    let macro_block = Block::Macro(MacroBlock {
        header: blockchain::MacroHeader {
            network: NetworkId::SPConsortium,
            version: 1,
            block_number: 256, // Election block number
            round: 0,
            timestamp: 1234567890,
            parent_hash: Blake2bHash::from_bytes([255u8; 32]),
            parent_election_hash: Blake2bHash::from_bytes([224u8; 32]), // Previous election
            seed: Blake2bHash::from_bytes([42u8; 32]),
            extra_data: vec![],
            state_root: Blake2bHash::zero(),
            body_root: Blake2bHash::zero(),
            history_root: Blake2bHash::zero(),
        },
        body: blockchain::MacroBody {
            validators: Some(validators.clone()),
            lost_reward_set: vec![],
            disabled_set: vec![],
            transactions: vec![],
        },
    });
    
    if let Block::Macro(ref block) = macro_block {
        assert!(block.body.validators.is_some());
        assert_eq!(block.body.validators.as_ref().unwrap().len(), 1);
        assert_eq!(block.header.block_number, 256);
    } else {
        panic!("Expected macro block");
    }
    
    println!("✅ Macro block validator updates work");
}

#[test]
fn test_transaction_types() {
    // Test different transaction types for SP CDR system
    
    // CDR Record transaction
    let cdr_tx = Transaction {
        sender: Blake2bHash::from_bytes([10u8; 32]),
        recipient: Blake2bHash::from_bytes([20u8; 32]),
        value: 0,
        fee: 10,
        validity_start_height: 100,
        data: blockchain::TransactionData::CDRRecord(blockchain::CDRTransaction {
            record_type: blockchain::CDRType::DataSession,
            home_network: "T-Mobile-DE".to_string(),
            visited_network: "Vodafone-UK".to_string(),
            encrypted_data: b"encrypted_session_data".to_vec(),
            zk_proof: b"zk_privacy_proof".to_vec(),
        }),
        signature: b"cdr_signature".to_vec(),
        signature_proof: b"cdr_proof".to_vec(),
    };
    
    // Settlement transaction
    let settlement_tx = Transaction {
        sender: Blake2bHash::from_bytes([30u8; 32]),
        recipient: Blake2bHash::from_bytes([40u8; 32]),
        value: 0,
        fee: 5,
        validity_start_height: 200,
        data: blockchain::TransactionData::Settlement(blockchain::SettlementTransaction {
            creditor_network: "Vodafone-UK".to_string(),
            debtor_network: "T-Mobile-DE".to_string(),
            amount: 125000, // 1250.00 EUR in cents
            currency: "EUR".to_string(),
            period: "2024-01-15-daily".to_string(),
        }),
        signature: b"settlement_signature".to_vec(),
        signature_proof: b"settlement_proof".to_vec(),
    };
    
    // Validator transaction
    let validator_tx = Transaction {
        sender: Blake2bHash::from_bytes([50u8; 32]),
        recipient: Blake2bHash::from_bytes([60u8; 32]),
        value: 1000000, // 1M stake
        fee: 100,
        validity_start_height: 300,
        data: blockchain::TransactionData::ValidatorUpdate(blockchain::ValidatorTransaction {
            action: blockchain::ValidatorAction::CreateValidator,
            validator_address: Blake2bHash::from_bytes([70u8; 32]),
            stake: 1000000,
        }),
        signature: b"validator_signature".to_vec(),
        signature_proof: b"validator_proof".to_vec(),
    };
    
    // Test transaction validation
    assert!(cdr_tx.is_valid());
    assert!(settlement_tx.is_valid());
    assert!(validator_tx.is_valid());
    
    // Test transaction hashes are unique
    let cdr_hash = cdr_tx.hash();
    let settlement_hash = settlement_tx.hash();
    let validator_hash = validator_tx.hash();
    
    assert_ne!(cdr_hash, settlement_hash);
    assert_ne!(settlement_hash, validator_hash);
    assert_ne!(cdr_hash, validator_hash);
    
    // Test transaction data extraction
    if let blockchain::TransactionData::CDRRecord(ref cdr) = cdr_tx.data {
        assert_eq!(cdr.record_type, blockchain::CDRType::DataSession);
        assert_eq!(cdr.home_network, "T-Mobile-DE");
        assert_eq!(cdr.visited_network, "Vodafone-UK");
    } else {
        panic!("Expected CDR transaction");
    }
    
    if let blockchain::TransactionData::Settlement(ref settlement) = settlement_tx.data {
        assert_eq!(settlement.amount, 125000);
        assert_eq!(settlement.currency, "EUR");
    } else {
        panic!("Expected settlement transaction");
    }
    
    if let blockchain::TransactionData::ValidatorUpdate(ref validator) = validator_tx.data {
        assert_eq!(validator.action, blockchain::ValidatorAction::CreateValidator);
        assert_eq!(validator.stake, 1000000);
    } else {
        panic!("Expected validator transaction");
    }
    
    println!("✅ All transaction types work correctly");
}

#[test]
fn test_block_validation_rules() {
    // Test block validation following Albatross rules
    let valid_micro = Block::Micro(MicroBlock {
        header: blockchain::MicroHeader {
            network: NetworkId::SPConsortium,
            version: 1,
            block_number: 10,
            timestamp: 1234567890,
            parent_hash: Blake2bHash::from_bytes([9u8; 32]),
            seed: Blake2bHash::from_bytes([10u8; 32]),
            extra_data: vec![],
            state_root: Blake2bHash::zero(),
            body_root: Blake2bHash::zero(),
            history_root: Blake2bHash::zero(),
        },
        body: blockchain::MicroBody {
            transactions: vec![],
        },
    });
    
    let valid_macro = Block::Macro(MacroBlock {
        header: blockchain::MacroHeader {
            network: NetworkId::SPConsortium,
            version: 1,
            block_number: 32, // Epoch boundary
            round: 0,
            timestamp: 1234567890,
            parent_hash: Blake2bHash::from_bytes([31u8; 32]),
            parent_election_hash: Blake2bHash::from_bytes([0u8; 32]),
            seed: Blake2bHash::from_bytes([32u8; 32]),
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
    
    // Basic validation checks
    assert_eq!(valid_micro.block_number(), 10);
    assert_eq!(valid_macro.block_number(), 32);
    
    // Test that block numbers are consistent with Albatross policy
    assert!(valid_macro.block_number() % lib::Policy::EPOCH_LENGTH == 0);
    
    println!("✅ Block validation rules work");
}

#[test]
fn test_network_id_consistency() {
    // Test that all blocks use correct network ID
    let micro_block = Block::Micro(MicroBlock {
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
    
    if let Block::Micro(ref block) = micro_block {
        assert_eq!(block.header.network, NetworkId::SPConsortium);
    }
    
    // Test network ID numeric value
    assert_eq!(NetworkId::SPConsortium as u8, 1);
    
    println!("✅ Network ID consistency works");
}

#[test]
fn test_policy_constants() {
    // Test that policy constants follow Albatross patterns
    assert_eq!(lib::Policy::EPOCH_LENGTH, 32);
    assert_eq!(lib::Policy::BATCH_LENGTH, 8);
    assert_eq!(lib::Policy::GENESIS_BLOCK_NUMBER, 0);
    assert_eq!(lib::Policy::BLOCK_TIME, 1000); // 1 second for SP reconciliation
    
    // Test epoch calculations
    let election_block_interval = lib::Policy::EPOCH_LENGTH * lib::Policy::BATCH_LENGTH;
    assert_eq!(election_block_interval, 256);
    
    println!("✅ Policy constants are correct");
}

#[test]
fn test_hash_functions() {
    // Test hash function consistency
    let data1 = b"test_data";
    let data2 = b"test_data";
    let data3 = b"different_data";
    
    let hash1 = hash_data(data1);
    let hash2 = hash_data(data2);
    let hash3 = hash_data(data3);
    
    assert_eq!(hash1, hash2); // Same data produces same hash
    assert_ne!(hash1, hash3); // Different data produces different hash
    
    // Test JSON hashing
    #[derive(serde::Serialize)]
    struct TestStruct {
        field1: u32,
        field2: String,
    }
    
    let obj1 = TestStruct { field1: 42, field2: "test".to_string() };
    let obj2 = TestStruct { field1: 42, field2: "test".to_string() };
    let obj3 = TestStruct { field1: 43, field2: "test".to_string() };
    
    let json_hash1 = hash_json(&obj1);
    let json_hash2 = hash_json(&obj2);
    let json_hash3 = hash_json(&obj3);
    
    assert_eq!(json_hash1, json_hash2);
    assert_ne!(json_hash1, json_hash3);
    
    println!("✅ Hash functions work correctly");
}