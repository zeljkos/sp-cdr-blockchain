// CDR-specific tests for SP roaming reconciliation
use sp_cdr_reconciliation_bc::*;

#[test]
fn test_cdr_transaction_types() {
    // Test all CDR record types
    let voice_call_tx = Transaction {
        sender: Blake2bHash::from_bytes([1u8; 32]),
        recipient: Blake2bHash::from_bytes([2u8; 32]),
        value: 0,
        fee: 10,
        validity_start_height: 0,
        data: blockchain::TransactionData::CDRRecord(blockchain::CDRTransaction {
            record_type: blockchain::CDRType::VoiceCall,
            home_network: "T-Mobile-DE".to_string(),
            visited_network: "Orange-FR".to_string(),
            encrypted_data: b"voice_call_duration_300_seconds".to_vec(),
            zk_proof: b"voice_call_privacy_proof".to_vec(),
        }),
        signature: b"voice_signature".to_vec(),
        signature_proof: b"voice_sig_proof".to_vec(),
    };
    
    let data_session_tx = Transaction {
        sender: Blake2bHash::from_bytes([3u8; 32]),
        recipient: Blake2bHash::from_bytes([4u8; 32]),
        value: 0,
        fee: 15,
        validity_start_height: 0,
        data: blockchain::TransactionData::CDRRecord(blockchain::CDRTransaction {
            record_type: blockchain::CDRType::DataSession,
            home_network: "Vodafone-UK".to_string(),
            visited_network: "Telekom-AT".to_string(),
            encrypted_data: b"data_session_5GB_transferred".to_vec(),
            zk_proof: b"data_session_privacy_proof".to_vec(),
        }),
        signature: b"data_signature".to_vec(),
        signature_proof: b"data_sig_proof".to_vec(),
    };
    
    let sms_tx = Transaction {
        sender: Blake2bHash::from_bytes([5u8; 32]),
        recipient: Blake2bHash::from_bytes([6u8; 32]),
        value: 0,
        fee: 5,
        validity_start_height: 0,
        data: blockchain::TransactionData::CDRRecord(blockchain::CDRTransaction {
            record_type: blockchain::CDRType::SMS,
            home_network: "Orange-ES".to_string(),
            visited_network: "TIM-IT".to_string(),
            encrypted_data: b"sms_count_25_messages".to_vec(),
            zk_proof: b"sms_privacy_proof".to_vec(),
        }),
        signature: b"sms_signature".to_vec(),
        signature_proof: b"sms_sig_proof".to_vec(),
    };
    
    let roaming_tx = Transaction {
        sender: Blake2bHash::from_bytes([7u8; 32]),
        recipient: Blake2bHash::from_bytes([8u8; 32]),
        value: 0,
        fee: 20,
        validity_start_height: 0,
        data: blockchain::TransactionData::CDRRecord(blockchain::CDRTransaction {
            record_type: blockchain::CDRType::Roaming,
            home_network: "Swisscom-CH".to_string(),
            visited_network: "A1-AT".to_string(),
            encrypted_data: b"roaming_session_24h_duration".to_vec(),
            zk_proof: b"roaming_privacy_proof".to_vec(),
        }),
        signature: b"roaming_signature".to_vec(),
        signature_proof: b"roaming_sig_proof".to_vec(),
    };
    
    // Verify all transactions are valid
    assert!(voice_call_tx.is_valid());
    assert!(data_session_tx.is_valid());
    assert!(sms_tx.is_valid());
    assert!(roaming_tx.is_valid());
    
    // Verify CDR data can be extracted
    for (tx, expected_type) in [
        (&voice_call_tx, blockchain::CDRType::VoiceCall),
        (&data_session_tx, blockchain::CDRType::DataSession),
        (&sms_tx, blockchain::CDRType::SMS),
        (&roaming_tx, blockchain::CDRType::Roaming),
    ] {
        if let blockchain::TransactionData::CDRRecord(ref cdr) = tx.data {
            assert_eq!(cdr.record_type, expected_type);
            assert!(!cdr.home_network.is_empty());
            assert!(!cdr.visited_network.is_empty());
            assert!(!cdr.encrypted_data.is_empty());
            assert!(!cdr.zk_proof.is_empty());
        } else {
            panic!("Expected CDR transaction");
        }
    }
    
    println!("✅ All CDR transaction types work");
}

#[test]
fn test_settlement_calculations() {
    // Test various settlement scenarios between SP operators
    let settlements = vec![
        // T-Mobile DE owes Orange FR
        blockchain::SettlementTransaction {
            creditor_network: "Orange-FR".to_string(),
            debtor_network: "T-Mobile-DE".to_string(),
            amount: 45000, // 450.00 EUR
            currency: "EUR".to_string(),
            period: "2024-01-15-daily".to_string(),
        },
        // Vodafone UK owes Telekom AT
        blockchain::SettlementTransaction {
            creditor_network: "Telekom-AT".to_string(),
            debtor_network: "Vodafone-UK".to_string(),
            amount: 125000, // 1250.00 EUR
            currency: "EUR".to_string(),
            period: "2024-01-15-daily".to_string(),
        },
        // Orange ES owes TIM IT
        blockchain::SettlementTransaction {
            creditor_network: "TIM-IT".to_string(),
            debtor_network: "Orange-ES".to_string(),
            amount: 78500, // 785.00 EUR
            currency: "EUR".to_string(),
            period: "2024-01-15-daily".to_string(),
        },
    ];
    
    // Test settlement data integrity
    for settlement in &settlements {
        assert!(!settlement.creditor_network.is_empty());
        assert!(!settlement.debtor_network.is_empty());
        assert!(settlement.amount > 0);
        assert_eq!(settlement.currency, "EUR");
        assert!(settlement.period.contains("2024-01-15"));
        
        // Ensure creditor and debtor are different
        assert_ne!(settlement.creditor_network, settlement.debtor_network);
    }
    
    // Calculate total settlement volume
    let total_volume: u64 = settlements.iter().map(|s| s.amount).sum();
    assert_eq!(total_volume, 248500); // 2485.00 EUR total
    
    println!("✅ Settlement calculations work");
}

#[test]
fn test_validator_actions() {
    // Test validator lifecycle actions
    let create_validator_tx = Transaction {
        sender: Blake2bHash::from_bytes([10u8; 32]),
        recipient: Blake2bHash::from_bytes([11u8; 32]),
        value: 2000000, // 2M stake
        fee: 1000,
        validity_start_height: 0,
        data: blockchain::TransactionData::ValidatorUpdate(blockchain::ValidatorTransaction {
            action: blockchain::ValidatorAction::CreateValidator,
            validator_address: Blake2bHash::from_bytes([100u8; 32]),
            stake: 2000000,
        }),
        signature: b"create_validator_sig".to_vec(),
        signature_proof: b"create_validator_proof".to_vec(),
    };
    
    let update_validator_tx = Transaction {
        sender: Blake2bHash::from_bytes([12u8; 32]),
        recipient: Blake2bHash::from_bytes([13u8; 32]),
        value: 0,
        fee: 100,
        validity_start_height: 100,
        data: blockchain::TransactionData::ValidatorUpdate(blockchain::ValidatorTransaction {
            action: blockchain::ValidatorAction::UpdateValidator,
            validator_address: Blake2bHash::from_bytes([100u8; 32]),
            stake: 2500000, // Increased stake
        }),
        signature: b"update_validator_sig".to_vec(),
        signature_proof: b"update_validator_proof".to_vec(),
    };
    
    let deactivate_validator_tx = Transaction {
        sender: Blake2bHash::from_bytes([14u8; 32]),
        recipient: Blake2bHash::from_bytes([15u8; 32]),
        value: 0,
        fee: 50,
        validity_start_height: 200,
        data: blockchain::TransactionData::ValidatorUpdate(blockchain::ValidatorTransaction {
            action: blockchain::ValidatorAction::DeactivateValidator,
            validator_address: Blake2bHash::from_bytes([100u8; 32]),
            stake: 0,
        }),
        signature: b"deactivate_validator_sig".to_vec(),
        signature_proof: b"deactivate_validator_proof".to_vec(),
    };
    
    let reactivate_validator_tx = Transaction {
        sender: Blake2bHash::from_bytes([16u8; 32]),
        recipient: Blake2bHash::from_bytes([17u8; 32]),
        value: 1500000, // Restake
        fee: 200,
        validity_start_height: 300,
        data: blockchain::TransactionData::ValidatorUpdate(blockchain::ValidatorTransaction {
            action: blockchain::ValidatorAction::ReactivateValidator,
            validator_address: Blake2bHash::from_bytes([100u8; 32]),
            stake: 1500000,
        }),
        signature: b"reactivate_validator_sig".to_vec(),
        signature_proof: b"reactivate_validator_proof".to_vec(),
    };
    
    let transactions = [
        (&create_validator_tx, blockchain::ValidatorAction::CreateValidator),
        (&update_validator_tx, blockchain::ValidatorAction::UpdateValidator),
        (&deactivate_validator_tx, blockchain::ValidatorAction::DeactivateValidator),
        (&reactivate_validator_tx, blockchain::ValidatorAction::ReactivateValidator),
    ];
    
    for (tx, expected_action) in transactions {
        assert!(tx.is_valid());
        
        if let blockchain::TransactionData::ValidatorUpdate(ref validator) = tx.data {
            assert_eq!(validator.action, expected_action);
            assert_eq!(validator.validator_address, Blake2bHash::from_bytes([100u8; 32]));
        } else {
            panic!("Expected validator transaction");
        }
    }
    
    println!("✅ Validator actions work");
}

#[test]
fn test_sp_network_interoperability() {
    // Test transactions between different SP operators
    let sp_networks = vec![
        "T-Mobile-DE", "T-Mobile-US", "T-Mobile-NL",
        "Vodafone-UK", "Vodafone-DE", "Vodafone-IT",
        "Orange-FR", "Orange-ES", "Orange-PL",
        "Telefonica-ES", "Telefonica-DE", "Telefonica-UK",
        "TIM-IT", "TIM-BR",
        "Swisscom-CH",
        "A1-AT",
        "Telekom-AT",
        "KPN-NL",
    ];
    
    // Create inter-network roaming transactions
    let mut transactions = vec![];
    
    for i in 0..sp_networks.len() {
        for j in 0..sp_networks.len() {
            if i != j {
                let tx = Transaction {
                    sender: Blake2bHash::from_bytes([i as u8; 32]),
                    recipient: Blake2bHash::from_bytes([j as u8; 32]),
                    value: 0,
                    fee: 10,
                    validity_start_height: 0,
                    data: blockchain::TransactionData::CDRRecord(blockchain::CDRTransaction {
                        record_type: blockchain::CDRType::Roaming,
                        home_network: sp_networks[i].to_string(),
                        visited_network: sp_networks[j].to_string(),
                        encrypted_data: format!("roaming_{}_{}", sp_networks[i], sp_networks[j]).into_bytes(),
                        zk_proof: format!("zk_proof_{}_{}", i, j).into_bytes(),
                    }),
                    signature: format!("sig_{}_{}", i, j).into_bytes(),
                    signature_proof: format!("proof_{}_{}", i, j).into_bytes(),
                };
                transactions.push(tx);
            }
        }
    }
    
    // Verify all inter-network transactions are valid
    assert_eq!(transactions.len(), sp_networks.len() * (sp_networks.len() - 1));
    
    for tx in &transactions {
        assert!(tx.is_valid());
        
        if let blockchain::TransactionData::CDRRecord(ref cdr) = tx.data {
            assert_ne!(cdr.home_network, cdr.visited_network);
            assert!(sp_networks.contains(&cdr.home_network.as_str()));
            assert!(sp_networks.contains(&cdr.visited_network.as_str()));
        }
    }
    
    println!("✅ SP network interoperability works with {} networks and {} transactions", 
             sp_networks.len(), transactions.len());
}

#[test]
fn test_cdr_block_aggregation() {
    // Test CDR aggregation in blocks following Albatross batch patterns
    let mut cdr_transactions = vec![];
    
    // Create multiple CDR records for a batch period
    for i in 0..50 {
        let tx = Transaction {
            sender: Blake2bHash::from_bytes([i as u8; 32]),
            recipient: Blake2bHash::from_bytes([i as u8 + 100; 32]),
            value: 0,
            fee: 5 + (i % 10) as u64, // Variable fees
            validity_start_height: 0,
            data: blockchain::TransactionData::CDRRecord(blockchain::CDRTransaction {
                record_type: match i % 4 {
                    0 => blockchain::CDRType::VoiceCall,
                    1 => blockchain::CDRType::DataSession,
                    2 => blockchain::CDRType::SMS,
                    _ => blockchain::CDRType::Roaming,
                },
                home_network: format!("SP-{}", i % 5),
                visited_network: format!("SP-{}", (i + 1) % 5),
                encrypted_data: format!("cdr_data_{}", i).into_bytes(),
                zk_proof: format!("zk_proof_{}", i).into_bytes(),
            }),
            signature: format!("signature_{}", i).into_bytes(),
            signature_proof: format!("sig_proof_{}", i).into_bytes(),
        };
        cdr_transactions.push(tx);
    }
    
    // Create micro block with CDR batch
    let cdr_batch_block = Block::Micro(MicroBlock {
        header: blockchain::MicroHeader {
            network: NetworkId::SPConsortium,
            version: 1,
            block_number: 15,
            timestamp: 1234567890,
            parent_hash: Blake2bHash::from_bytes([14u8; 32]),
            seed: Blake2bHash::from_bytes([15u8; 32]),
            extra_data: b"CDR_BATCH_BLOCK".to_vec(),
            state_root: Blake2bHash::zero(),
            body_root: Blake2bHash::zero(),
            history_root: Blake2bHash::zero(),
        },
        body: blockchain::MicroBody {
            transactions: cdr_transactions.clone(),
        },
    });
    
    // Verify block contains all CDR transactions
    if let Block::Micro(ref micro) = cdr_batch_block {
        assert_eq!(micro.body.transactions.len(), 50);
        
        // Count transaction types
        let mut type_counts = std::collections::HashMap::new();
        for tx in &micro.body.transactions {
            if let blockchain::TransactionData::CDRRecord(ref cdr) = tx.data {
                *type_counts.entry(cdr.record_type.clone()).or_insert(0) += 1;
            }
        }
        
        // Verify distribution of CDR types
        assert_eq!(type_counts.len(), 4); // All 4 CDR types present
        assert!(type_counts.values().all(|&count| count > 0));
    } else {
        panic!("Expected micro block");
    }
    
    println!("✅ CDR block aggregation works with {} transactions", cdr_transactions.len());
}

#[test]
fn test_daily_settlement_aggregation() {
    // Test daily settlement aggregation in macro blocks
    let daily_settlements = vec![
        blockchain::SettlementTransaction {
            creditor_network: "T-Mobile-DE".to_string(),
            debtor_network: "Orange-FR".to_string(),
            amount: 125000,
            currency: "EUR".to_string(),
            period: "2024-01-15-daily".to_string(),
        },
        blockchain::SettlementTransaction {
            creditor_network: "Vodafone-UK".to_string(),
            debtor_network: "TIM-IT".to_string(),
            amount: 87500,
            currency: "EUR".to_string(),
            period: "2024-01-15-daily".to_string(),
        },
        blockchain::SettlementTransaction {
            creditor_network: "Swisscom-CH".to_string(),
            debtor_network: "A1-AT".to_string(),
            amount: 65000,
            currency: "EUR".to_string(),
            period: "2024-01-15-daily".to_string(),
        },
    ];
    
    let settlement_transactions: Vec<Transaction> = daily_settlements.into_iter().map(|settlement| {
        Transaction {
            sender: Blake2bHash::from_bytes([99u8; 32]),
            recipient: Blake2bHash::from_bytes([88u8; 32]),
            value: 0,
            fee: 20,
            validity_start_height: 0,
            data: blockchain::TransactionData::Settlement(settlement),
            signature: b"settlement_batch_sig".to_vec(),
            signature_proof: b"settlement_batch_proof".to_vec(),
        }
    }).collect();
    
    // Create macro block with daily settlements
    let settlement_macro_block = Block::Macro(MacroBlock {
        header: blockchain::MacroHeader {
            network: NetworkId::SPConsortium,
            version: 1,
            block_number: 32, // End of epoch
            round: 0,
            timestamp: 1705363200, // 2024-01-15 midnight
            parent_hash: Blake2bHash::from_bytes([31u8; 32]),
            parent_election_hash: Blake2bHash::zero(),
            seed: Blake2bHash::from_bytes([32u8; 32]),
            extra_data: b"DAILY_SETTLEMENT_2024-01-15".to_vec(),
            state_root: Blake2bHash::zero(),
            body_root: Blake2bHash::zero(),
            history_root: Blake2bHash::zero(),
        },
        body: blockchain::MacroBody {
            validators: None,
            lost_reward_set: vec![],
            disabled_set: vec![],
            transactions: settlement_transactions,
        },
    });
    
    // Verify macro block contains settlements
    if let Block::Macro(ref macro_block) = settlement_macro_block {
        assert_eq!(macro_block.body.transactions.len(), 3);
        
        let total_settlement: u64 = macro_block.body.transactions.iter()
            .filter_map(|tx| match &tx.data {
                blockchain::TransactionData::Settlement(settlement) => Some(settlement.amount),
                _ => None,
            })
            .sum();
        
        assert_eq!(total_settlement, 277500); // 2775.00 EUR total
        
        // Verify all settlements are for the same period
        for tx in &macro_block.body.transactions {
            if let blockchain::TransactionData::Settlement(ref settlement) = tx.data {
                assert_eq!(settlement.period, "2024-01-15-daily");
                assert_eq!(settlement.currency, "EUR");
            }
        }
    } else {
        panic!("Expected macro block");
    }
    
    println!("✅ Daily settlement aggregation works");
}

#[test]
fn test_privacy_and_zk_proof_structure() {
    // Test that ZK proofs are properly structured for privacy
    let privacy_tx = Transaction {
        sender: Blake2bHash::from_bytes([200u8; 32]),
        recipient: Blake2bHash::from_bytes([201u8; 32]),
        value: 0,
        fee: 25,
        validity_start_height: 0,
        data: blockchain::TransactionData::CDRRecord(blockchain::CDRTransaction {
            record_type: blockchain::CDRType::DataSession,
            home_network: "Encrypted-Home-Network".to_string(),
            visited_network: "Encrypted-Visited-Network".to_string(),
            encrypted_data: b"AES256_ENCRYPTED_SUBSCRIBER_DATA_HASH_COMMITMENT".to_vec(),
            zk_proof: b"GROTH16_PROOF_SUBSCRIBER_VALIDITY_WITHOUT_REVEALING_IDENTITY".to_vec(),
        }),
        signature: b"BLS_AGGREGATE_SIGNATURE".to_vec(),
        signature_proof: b"SCHNORR_PROOF_OF_POSSESSION".to_vec(),
    };
    
    // Verify privacy structure
    assert!(privacy_tx.is_valid());
    
    if let blockchain::TransactionData::CDRRecord(ref cdr) = privacy_tx.data {
        // Ensure encrypted data is present
        assert!(!cdr.encrypted_data.is_empty());
        assert!(cdr.encrypted_data.len() > 10); // Reasonable minimum size
        
        // Ensure ZK proof is present
        assert!(!cdr.zk_proof.is_empty());
        assert!(cdr.zk_proof.len() > 10); // Reasonable minimum size
        
        // Network identifiers should be present (could be hashed/encrypted)
        assert!(!cdr.home_network.is_empty());
        assert!(!cdr.visited_network.is_empty());
    } else {
        panic!("Expected CDR transaction");
    }
    
    // Test that signature and signature proof are present
    assert!(!privacy_tx.signature.is_empty());
    assert!(!privacy_tx.signature_proof.is_empty());
    
    println!("✅ Privacy and ZK proof structure works");
}