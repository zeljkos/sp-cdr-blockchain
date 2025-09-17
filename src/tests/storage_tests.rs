// Storage layer tests
use sp_cdr_reconciliation_bc::*;
use tempfile::TempDir;
use libmdbx::Environment;
use std::sync::Arc;

async fn create_test_storage() -> (TempDir, Arc<storage::MdbxChainStore>) {
    let temp_dir = TempDir::new().unwrap();
    let env = Environment::new()
        .set_max_dbs(10)
        .open(temp_dir.path())
        .unwrap();
    
    let store = Arc::new(storage::MdbxChainStore::new(env).unwrap());
    (temp_dir, store)
}

#[tokio::test]
async fn test_storage_creation() {
    // Test MDBX storage can be created
    let (_temp_dir, store) = create_test_storage().await;
    
    // Test initial state
    let head_hash = store.get_head_hash().await.unwrap();
    assert_eq!(head_hash, Blake2bHash::zero());
    
    let macro_head = store.get_macro_head_hash().await.unwrap();
    assert_eq!(macro_head, Blake2bHash::zero());
    
    let election_head = store.get_election_head_hash().await.unwrap();
    assert_eq!(election_head, Blake2bHash::zero());
    
    println!("✅ Storage creation works");
}

#[tokio::test]
async fn test_block_storage_and_retrieval() {
    // Test basic block storage operations
    let (_temp_dir, store) = create_test_storage().await;
    
    // Create test blocks
    let micro_block = Block::Micro(MicroBlock {
        header: blockchain::MicroHeader {
            network: NetworkId::SPConsortium,
            version: 1,
            block_number: 1,
            timestamp: 1234567890,
            parent_hash: Blake2bHash::zero(),
            seed: Blake2bHash::from_bytes([1u8; 32]),
            extra_data: b"test_block".to_vec(),
            state_root: Blake2bHash::zero(),
            body_root: Blake2bHash::zero(),
            history_root: Blake2bHash::zero(),
        },
        body: blockchain::MicroBody {
            transactions: vec![],
        },
    });
    
    let macro_block = Block::Macro(MacroBlock {
        header: blockchain::MacroHeader {
            network: NetworkId::SPConsortium,
            version: 1,
            block_number: 32,
            round: 0,
            timestamp: 1234567890,
            parent_hash: Blake2bHash::from_bytes([31u8; 32]),
            parent_election_hash: Blake2bHash::zero(),
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
    
    let micro_hash = micro_block.hash();
    let macro_hash = macro_block.hash();
    
    // Store blocks
    store.put_block(&micro_block).await.unwrap();
    store.put_block(&macro_block).await.unwrap();
    
    // Retrieve blocks by hash
    let retrieved_micro = store.get_block(&micro_hash).await.unwrap().unwrap();
    let retrieved_macro = store.get_block(&macro_hash).await.unwrap().unwrap();
    
    assert_eq!(micro_block.hash(), retrieved_micro.hash());
    assert_eq!(macro_block.hash(), retrieved_macro.hash());
    assert_eq!(micro_block.block_number(), retrieved_micro.block_number());
    assert_eq!(macro_block.block_number(), retrieved_macro.block_number());
    
    // Retrieve blocks by height
    let micro_by_height = store.get_block_at(1).await.unwrap().unwrap();
    let macro_by_height = store.get_block_at(32).await.unwrap().unwrap();
    
    assert_eq!(micro_block.hash(), micro_by_height.hash());
    assert_eq!(macro_block.hash(), macro_by_height.hash());
    
    println!("✅ Block storage and retrieval works");
}

#[tokio::test]
async fn test_head_pointer_management() {
    // Test head pointer updates
    let (_temp_dir, store) = create_test_storage().await;
    
    // Create chain of blocks
    let blocks: Vec<Block> = (1..=5).map(|i| {
        Block::Micro(MicroBlock {
            header: blockchain::MicroHeader {
                network: NetworkId::SPConsortium,
                version: 1,
                block_number: i,
                timestamp: 1234567890 + i as u64,
                parent_hash: if i == 1 { 
                    Blake2bHash::zero() 
                } else { 
                    Blake2bHash::from_bytes([i as u8 - 1; 32]) 
                },
                seed: Blake2bHash::from_bytes([i as u8; 32]),
                extra_data: vec![],
                state_root: Blake2bHash::zero(),
                body_root: Blake2bHash::zero(),
                history_root: Blake2bHash::zero(),
            },
            body: blockchain::MicroBody {
                transactions: vec![],
            },
        })
    }).collect();
    
    // Store blocks and update head
    for block in &blocks {
        store.put_block(block).await.unwrap();
        store.set_head(&block.hash()).await.unwrap();
    }
    
    // Verify head pointer
    let head_hash = store.get_head_hash().await.unwrap();
    assert_eq!(head_hash, blocks[4].hash()); // Block 5
    
    // Verify we can retrieve the head block
    let head_block = store.get_block(&head_hash).await.unwrap().unwrap();
    assert_eq!(head_block.block_number(), 5);
    
    println!("✅ Head pointer management works");
}

#[tokio::test]
async fn test_macro_and_election_heads() {
    // Test macro and election head management
    let (_temp_dir, store) = create_test_storage().await;
    
    // Create macro blocks
    let macro_block_32 = Block::Macro(MacroBlock {
        header: blockchain::MacroHeader {
            network: NetworkId::SPConsortium,
            version: 1,
            block_number: 32,
            round: 0,
            timestamp: 1234567890,
            parent_hash: Blake2bHash::from_bytes([31u8; 32]),
            parent_election_hash: Blake2bHash::zero(),
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
    
    // Election block (every 256 blocks)
    let election_block_256 = Block::Macro(MacroBlock {
        header: blockchain::MacroHeader {
            network: NetworkId::SPConsortium,
            version: 1,
            block_number: 256,
            round: 0,
            timestamp: 1234567890,
            parent_hash: Blake2bHash::from_bytes([255u8; 32]),
            parent_election_hash: Blake2bHash::zero(),
            seed: Blake2bHash::from_bytes([0u8; 32]),
            extra_data: vec![],
            state_root: Blake2bHash::zero(),
            body_root: Blake2bHash::zero(),
            history_root: Blake2bHash::zero(),
        },
        body: blockchain::MacroBody {
            validators: Some(vec![ValidatorInfo {
                address: Blake2bHash::from_bytes([1u8; 32]),
                signing_key: vec![1u8; 48],
                voting_key: vec![1u8; 32],
                reward_address: Blake2bHash::from_bytes([1u8; 32]),
                signal_data: None,
                inactive_from: None,
                jailed_from: None,
            }]),
            lost_reward_set: vec![],
            disabled_set: vec![],
            transactions: vec![],
        },
    });
    
    let macro_hash = macro_block_32.hash();
    let election_hash = election_block_256.hash();
    
    // Store blocks
    store.put_block(&macro_block_32).await.unwrap();
    store.put_block(&election_block_256).await.unwrap();
    
    // Set macro head
    store.set_macro_head(&macro_hash).await.unwrap();
    let retrieved_macro_head = store.get_macro_head_hash().await.unwrap();
    assert_eq!(retrieved_macro_head, macro_hash);
    
    // Set election head
    store.set_election_head(&election_hash).await.unwrap();
    let retrieved_election_head = store.get_election_head_hash().await.unwrap();
    assert_eq!(retrieved_election_head, election_hash);
    
    println!("✅ Macro and election head management works");
}

#[tokio::test]
async fn test_block_with_transactions_storage() {
    // Test storing blocks with various transaction types
    let (_temp_dir, store) = create_test_storage().await;
    
    // Create CDR transaction
    let cdr_tx = Transaction {
        sender: Blake2bHash::from_bytes([10u8; 32]),
        recipient: Blake2bHash::from_bytes([20u8; 32]),
        value: 0,
        fee: 10,
        validity_start_height: 0,
        data: blockchain::TransactionData::CDRRecord(blockchain::CDRTransaction {
            record_type: blockchain::CDRType::VoiceCall,
            home_network: "T-Mobile-DE".to_string(),
            visited_network: "Vodafone-UK".to_string(),
            encrypted_data: b"encrypted_call_data".to_vec(),
            zk_proof: b"privacy_proof".to_vec(),
        }),
        signature: b"cdr_signature".to_vec(),
        signature_proof: b"cdr_sig_proof".to_vec(),
    };
    
    // Create settlement transaction
    let settlement_tx = Transaction {
        sender: Blake2bHash::from_bytes([30u8; 32]),
        recipient: Blake2bHash::from_bytes([40u8; 32]),
        value: 0,
        fee: 5,
        validity_start_height: 0,
        data: blockchain::TransactionData::Settlement(blockchain::SettlementTransaction {
            creditor_network: "Vodafone-UK".to_string(),
            debtor_network: "T-Mobile-DE".to_string(),
            amount: 75000,
            currency: "EUR".to_string(),
            period: "2024-01-16-daily".to_string(),
        }),
        signature: b"settlement_signature".to_vec(),
        signature_proof: b"settlement_sig_proof".to_vec(),
    };
    
    // Create block with transactions
    let block_with_txs = Block::Micro(MicroBlock {
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
            transactions: vec![cdr_tx.clone(), settlement_tx.clone()],
        },
    });
    
    let block_hash = block_with_txs.hash();
    
    // Store block with transactions
    store.put_block(&block_with_txs).await.unwrap();
    
    // Retrieve and verify transactions are preserved
    let retrieved_block = store.get_block(&block_hash).await.unwrap().unwrap();
    
    if let Block::Micro(ref micro) = retrieved_block {
        assert_eq!(micro.body.transactions.len(), 2);
        
        // Verify CDR transaction
        if let blockchain::TransactionData::CDRRecord(ref cdr) = micro.body.transactions[0].data {
            assert_eq!(cdr.record_type, blockchain::CDRType::VoiceCall);
            assert_eq!(cdr.home_network, "T-Mobile-DE");
            assert_eq!(cdr.visited_network, "Vodafone-UK");
        } else {
            panic!("Expected CDR transaction");
        }
        
        // Verify settlement transaction
        if let blockchain::TransactionData::Settlement(ref settlement) = micro.body.transactions[1].data {
            assert_eq!(settlement.amount, 75000);
            assert_eq!(settlement.currency, "EUR");
            assert_eq!(settlement.period, "2024-01-16-daily");
        } else {
            panic!("Expected settlement transaction");
        }
    } else {
        panic!("Expected micro block");
    }
    
    println!("✅ Block with transactions storage works");
}

#[tokio::test]
async fn test_storage_error_handling() {
    // Test storage error scenarios
    let (_temp_dir, store) = create_test_storage().await;
    
    // Test retrieving non-existent block
    let non_existent_hash = Blake2bHash::from_bytes([99u8; 32]);
    let result = store.get_block(&non_existent_hash).await.unwrap();
    assert!(result.is_none());
    
    // Test retrieving block at non-existent height
    let result = store.get_block_at(1000).await.unwrap();
    assert!(result.is_none());
    
    println!("✅ Storage error handling works");
}

#[tokio::test]
async fn test_storage_persistence() {
    // Test that data persists across store instances
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();
    
    // Create first store instance
    {
        let env = Environment::new()
            .set_max_dbs(10)
            .open(&path)
            .unwrap();
        let store = storage::MdbxChainStore::new(env).unwrap();
        
        let test_block = Block::Micro(MicroBlock {
            header: blockchain::MicroHeader {
                network: NetworkId::SPConsortium,
                version: 1,
                block_number: 42,
                timestamp: 1234567890,
                parent_hash: Blake2bHash::zero(),
                seed: Blake2bHash::from_bytes([42u8; 32]),
                extra_data: b"persistent_test".to_vec(),
                state_root: Blake2bHash::zero(),
                body_root: Blake2bHash::zero(),
                history_root: Blake2bHash::zero(),
            },
            body: blockchain::MicroBody {
                transactions: vec![],
            },
        });
        
        let test_hash = test_block.hash();
        store.put_block(&test_block).await.unwrap();
        store.set_head(&test_hash).await.unwrap();
    }
    
    // Create second store instance with same path
    {
        let env = Environment::new()
            .set_max_dbs(10)
            .open(&path)
            .unwrap();
        let store = storage::MdbxChainStore::new(env).unwrap();
        
        // Verify data persisted
        let head_hash = store.get_head_hash().await.unwrap();
        assert_ne!(head_hash, Blake2bHash::zero());
        
        let persisted_block = store.get_block(&head_hash).await.unwrap().unwrap();
        assert_eq!(persisted_block.block_number(), 42);
        
        if let Block::Micro(ref micro) = persisted_block {
            assert_eq!(micro.header.extra_data, b"persistent_test");
        } else {
            panic!("Expected micro block");
        }
    }
    
    println!("✅ Storage persistence works");
}

#[tokio::test]
async fn test_concurrent_storage_operations() {
    // Test concurrent access to storage
    let (_temp_dir, store) = create_test_storage().await;
    let store = Arc::new(store);
    
    let mut handles = vec![];
    
    // Spawn multiple tasks that store blocks concurrently
    for i in 1..=10 {
        let store_clone = store.clone();
        let handle = tokio::spawn(async move {
            let block = Block::Micro(MicroBlock {
                header: blockchain::MicroHeader {
                    network: NetworkId::SPConsortium,
                    version: 1,
                    block_number: i,
                    timestamp: 1234567890 + i as u64,
                    parent_hash: Blake2bHash::from_bytes([i as u8; 32]),
                    seed: Blake2bHash::from_bytes([i as u8 + 100; 32]),
                    extra_data: format!("concurrent_block_{}", i).into_bytes(),
                    state_root: Blake2bHash::zero(),
                    body_root: Blake2bHash::zero(),
                    history_root: Blake2bHash::zero(),
                },
                body: blockchain::MicroBody {
                    transactions: vec![],
                },
            });
            
            store_clone.put_block(&block).await.unwrap();
            block.hash()
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    let mut hashes = vec![];
    for handle in handles {
        let hash = handle.await.unwrap();
        hashes.push(hash);
    }
    
    // Verify all blocks were stored
    for (i, hash) in hashes.iter().enumerate() {
        let block = store.get_block(hash).await.unwrap().unwrap();
        assert_eq!(block.block_number(), (i + 1) as u32);
    }
    
    println!("✅ Concurrent storage operations work");
}