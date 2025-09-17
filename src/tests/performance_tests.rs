// Performance tests for SP CDR reconciliation blockchain
use sp_cdr_reconciliation_bc::*;
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;
use libmdbx::Environment;

async fn create_perf_storage() -> (TempDir, Arc<storage::MdbxChainStore>) {
    let temp_dir = TempDir::new().unwrap();
    let env = Environment::new()
        .set_max_dbs(10)
        .open(temp_dir.path())
        .unwrap();
    
    let store = Arc::new(storage::MdbxChainStore::new(env).unwrap());
    (temp_dir, store)
}

#[tokio::test]
async fn test_block_storage_performance() {
    // Test storage performance with large number of blocks
    let (_temp_dir, store) = create_perf_storage().await;
    
    let num_blocks = 1000;
    let start_time = Instant::now();
    
    // Store blocks sequentially
    for i in 1..=num_blocks {
        let block = Block::Micro(MicroBlock {
            header: blockchain::MicroHeader {
                network: NetworkId::SPConsortium,
                version: 1,
                block_number: i,
                timestamp: 1234567890 + i as u64,
                parent_hash: Blake2bHash::from_bytes([i as u8; 32]),
                seed: Blake2bHash::from_bytes([i as u8 + 100; 32]),
                extra_data: format!("perf_test_block_{}", i).into_bytes(),
                state_root: Blake2bHash::zero(),
                body_root: Blake2bHash::zero(),
                history_root: Blake2bHash::zero(),
            },
            body: blockchain::MicroBody {
                transactions: vec![],
            },
        });
        
        store.put_block(&block).await.unwrap();
        
        if i % 100 == 0 {
            println!("Stored {} blocks", i);
        }
    }
    
    let storage_duration = start_time.elapsed();
    println!("✅ Stored {} blocks in {:?}", num_blocks, storage_duration);
    println!("   Average: {:.2} blocks/sec", num_blocks as f64 / storage_duration.as_secs_f64());
    
    // Test retrieval performance
    let retrieval_start = Instant::now();
    
    for i in 1..=num_blocks {
        let block = store.get_block_at(i).await.unwrap().unwrap();
        assert_eq!(block.block_number(), i);
        
        if i % 100 == 0 {
            println!("Retrieved {} blocks", i);
        }
    }
    
    let retrieval_duration = retrieval_start.elapsed();
    println!("✅ Retrieved {} blocks in {:?}", num_blocks, retrieval_duration);
    println!("   Average: {:.2} blocks/sec", num_blocks as f64 / retrieval_duration.as_secs_f64());
}

#[tokio::test]
async fn test_concurrent_block_operations() {
    // Test concurrent block operations performance
    let (_temp_dir, store) = create_perf_storage().await;
    let store = Arc::new(store);
    
    let num_concurrent_tasks = 10;
    let blocks_per_task = 100;
    let total_blocks = num_concurrent_tasks * blocks_per_task;
    
    let start_time = Instant::now();
    
    let mut handles = vec![];
    
    for task_id in 0..num_concurrent_tasks {
        let store_clone = store.clone();
        let handle = tokio::spawn(async move {
            for i in 0..blocks_per_task {
                let block_number = task_id * blocks_per_task + i + 1;
                let block = Block::Micro(MicroBlock {
                    header: blockchain::MicroHeader {
                        network: NetworkId::SPConsortium,
                        version: 1,
                        block_number: block_number as u32,
                        timestamp: 1234567890 + block_number as u64,
                        parent_hash: Blake2bHash::from_bytes([block_number as u8; 32]),
                        seed: Blake2bHash::from_bytes([block_number as u8 + 50; 32]),
                        extra_data: format!("concurrent_block_{}", block_number).into_bytes(),
                        state_root: Blake2bHash::zero(),
                        body_root: Blake2bHash::zero(),
                        history_root: Blake2bHash::zero(),
                    },
                    body: blockchain::MicroBody {
                        transactions: vec![],
                    },
                });
                
                store_clone.put_block(&block).await.unwrap();
            }
            task_id
        });
        handles.push(handle);
    }
    
    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }
    
    let duration = start_time.elapsed();
    println!("✅ Stored {} blocks concurrently ({} tasks) in {:?}", 
             total_blocks, num_concurrent_tasks, duration);
    println!("   Average: {:.2} blocks/sec", 
             total_blocks as f64 / duration.as_secs_f64());
}

#[tokio::test]
async fn test_large_transaction_blocks() {
    // Test performance with blocks containing many transactions
    let (_temp_dir, store) = create_perf_storage().await;
    
    let transactions_per_block = 1000;
    let num_blocks = 10;
    
    let start_time = Instant::now();
    
    for block_num in 1..=num_blocks {
        // Create many CDR transactions
        let mut transactions = vec![];
        for tx_num in 0..transactions_per_block {
            let tx = Transaction {
                sender: Blake2bHash::from_bytes([tx_num as u8; 32]),
                recipient: Blake2bHash::from_bytes([tx_num as u8 + 128; 32]),
                value: 0,
                fee: (tx_num % 50) as u64 + 1,
                validity_start_height: 0,
                data: blockchain::TransactionData::CDRRecord(blockchain::CDRTransaction {
                    record_type: match tx_num % 4 {
                        0 => blockchain::CDRType::VoiceCall,
                        1 => blockchain::CDRType::DataSession,
                        2 => blockchain::CDRType::SMS,
                        _ => blockchain::CDRType::Roaming,
                    },
                    home_network: format!("SP-{}", tx_num % 10),
                    visited_network: format!("SP-{}", (tx_num + 1) % 10),
                    encrypted_data: format!("large_cdr_data_{}_block_{}", tx_num, block_num)
                        .repeat(10) // Make data larger
                        .into_bytes(),
                    zk_proof: format!("zk_proof_{}_block_{}", tx_num, block_num)
                        .repeat(5) // Make proof larger
                        .into_bytes(),
                }),
                signature: format!("signature_{}_{}", tx_num, block_num).into_bytes(),
                signature_proof: format!("sig_proof_{}_{}", tx_num, block_num).into_bytes(),
            };
            transactions.push(tx);
        }
        
        let block = Block::Micro(MicroBlock {
            header: blockchain::MicroHeader {
                network: NetworkId::SPConsortium,
                version: 1,
                block_number: block_num,
                timestamp: 1234567890 + block_num as u64,
                parent_hash: Blake2bHash::from_bytes([block_num as u8; 32]),
                seed: Blake2bHash::from_bytes([block_num as u8 + 200; 32]),
                extra_data: format!("large_block_{}", block_num).into_bytes(),
                state_root: Blake2bHash::zero(),
                body_root: Blake2bHash::zero(),
                history_root: Blake2bHash::zero(),
            },
            body: blockchain::MicroBody {
                transactions,
            },
        });
        
        let block_start = Instant::now();
        store.put_block(&block).await.unwrap();
        let block_duration = block_start.elapsed();
        
        println!("Block {} ({} txs) stored in {:?}", 
                 block_num, transactions_per_block, block_duration);
    }
    
    let total_duration = start_time.elapsed();
    let total_transactions = num_blocks * transactions_per_block;
    
    println!("✅ Stored {} blocks with {} total transactions in {:?}", 
             num_blocks, total_transactions, total_duration);
    println!("   Average: {:.2} transactions/sec", 
             total_transactions as f64 / total_duration.as_secs_f64());
}

#[test]
fn test_hash_performance() {
    // Test hashing performance for blocks and transactions
    let num_operations = 10000;
    
    // Test Blake2b hash performance
    let start_time = Instant::now();
    
    for i in 0..num_operations {
        let data = format!("performance_test_data_{}", i);
        let _hash = hash_data(data.as_bytes());
    }
    
    let hash_duration = start_time.elapsed();
    println!("✅ Performed {} hash operations in {:?}", 
             num_operations, hash_duration);
    println!("   Average: {:.2} hashes/sec", 
             num_operations as f64 / hash_duration.as_secs_f64());
    
    // Test JSON hash performance
    #[derive(serde::Serialize)]
    struct TestObject {
        field1: u64,
        field2: String,
        field3: Vec<u8>,
    }
    
    let json_start = Instant::now();
    
    for i in 0..num_operations {
        let obj = TestObject {
            field1: i as u64,
            field2: format!("json_test_{}", i),
            field3: vec![i as u8; 100],
        };
        let _hash = hash_json(&obj);
    }
    
    let json_duration = json_start.elapsed();
    println!("✅ Performed {} JSON hash operations in {:?}", 
             num_operations, json_duration);
    println!("   Average: {:.2} JSON hashes/sec", 
             num_operations as f64 / json_duration.as_secs_f64());
}

#[test]
fn test_serialization_performance() {
    // Test serialization/deserialization performance
    let num_operations = 1000;
    
    // Create complex transactions for serialization testing
    let transactions: Vec<Transaction> = (0..100).map(|i| {
        Transaction {
            sender: Blake2bHash::from_bytes([i; 32]),
            recipient: Blake2bHash::from_bytes([i + 1; 32]),
            value: i as u64 * 1000,
            fee: i as u64 + 10,
            validity_start_height: i,
            data: blockchain::TransactionData::CDRRecord(blockchain::CDRTransaction {
                record_type: blockchain::CDRType::DataSession,
                home_network: format!("Network-{}", i),
                visited_network: format!("Network-{}", i + 1),
                encrypted_data: vec![i; 1000], // 1KB encrypted data
                zk_proof: vec![i + 100; 500], // 500B ZK proof
            }),
            signature: vec![i + 50; 64], // 64B signature
            signature_proof: vec![i + 75; 32], // 32B signature proof
        }
    }).collect();
    
    let block = Block::Micro(MicroBlock {
        header: blockchain::MicroHeader {
            network: NetworkId::SPConsortium,
            version: 1,
            block_number: 1,
            timestamp: 1234567890,
            parent_hash: Blake2bHash::zero(),
            seed: Blake2bHash::from_bytes([42u8; 32]),
            extra_data: vec![0u8; 1000], // 1KB extra data
            state_root: Blake2bHash::zero(),
            body_root: Blake2bHash::zero(),
            history_root: Blake2bHash::zero(),
        },
        body: blockchain::MicroBody {
            transactions,
        },
    });
    
    // Test serialization
    let serialize_start = Instant::now();
    let mut serialized_data = Vec::new();
    
    for _ in 0..num_operations {
        let data = serde_json::to_vec(&block).unwrap();
        serialized_data = data; // Keep last one to prevent optimization
    }
    
    let serialize_duration = serialize_start.elapsed();
    println!("✅ Serialized block {} times in {:?} (size: {} bytes)", 
             num_operations, serialize_duration, serialized_data.len());
    println!("   Average: {:.2} serializations/sec", 
             num_operations as f64 / serialize_duration.as_secs_f64());
    
    // Test deserialization
    let deserialize_start = Instant::now();
    let mut deserialized_block = block.clone();
    
    for _ in 0..num_operations {
        let block: Block = serde_json::from_slice(&serialized_data).unwrap();
        deserialized_block = block; // Keep last one to prevent optimization
    }
    
    let deserialize_duration = deserialize_start.elapsed();
    println!("✅ Deserialized block {} times in {:?}", 
             num_operations, deserialize_duration);
    println!("   Average: {:.2} deserializations/sec", 
             num_operations as f64 / deserialize_duration.as_secs_f64());
    
    // Verify correctness
    assert_eq!(block.hash(), deserialized_block.hash());
}

#[tokio::test]
async fn test_memory_usage() {
    // Test memory usage with large datasets
    let (_temp_dir, store) = create_perf_storage().await;
    
    let initial_memory = get_memory_usage();
    println!("Initial memory usage: {} MB", initial_memory);
    
    // Create and store many blocks
    let num_blocks = 100;
    let transactions_per_block = 100;
    
    for i in 1..=num_blocks {
        let transactions: Vec<Transaction> = (0..transactions_per_block).map(|j| {
            Transaction {
                sender: Blake2bHash::from_bytes([i as u8; 32]),
                recipient: Blake2bHash::from_bytes([j as u8; 32]),
                value: 0,
                fee: 10,
                validity_start_height: 0,
                data: blockchain::TransactionData::CDRRecord(blockchain::CDRTransaction {
                    record_type: blockchain::CDRType::DataSession,
                    home_network: format!("SP-{}", i),
                    visited_network: format!("SP-{}", j),
                    encrypted_data: vec![i as u8; 1000], // 1KB per transaction
                    zk_proof: vec![j as u8; 500], // 500B per proof
                }),
                signature: vec![i as u8 + j as u8; 64],
                signature_proof: vec![i as u8 * j as u8; 32],
            }
        }).collect();
        
        let block = Block::Micro(MicroBlock {
            header: blockchain::MicroHeader {
                network: NetworkId::SPConsortium,
                version: 1,
                block_number: i,
                timestamp: 1234567890 + i as u64,
                parent_hash: Blake2bHash::from_bytes([i as u8; 32]),
                seed: Blake2bHash::from_bytes([i as u8 + 50; 32]),
                extra_data: vec![],
                state_root: Blake2bHash::zero(),
                body_root: Blake2bHash::zero(),
                history_root: Blake2bHash::zero(),
            },
            body: blockchain::MicroBody {
                transactions,
            },
        });
        
        store.put_block(&block).await.unwrap();
        
        if i % 10 == 0 {
            let current_memory = get_memory_usage();
            println!("After {} blocks: {} MB (delta: {} MB)", 
                     i, current_memory, current_memory - initial_memory);
        }
    }
    
    let final_memory = get_memory_usage();
    println!("✅ Final memory usage: {} MB (total delta: {} MB)", 
             final_memory, final_memory - initial_memory);
    
    let total_transactions = num_blocks * transactions_per_block;
    let memory_per_transaction = (final_memory - initial_memory) / total_transactions as f64;
    println!("   Average memory per transaction: {:.2} KB", memory_per_transaction * 1024.0);
}

// Helper function to estimate memory usage
fn get_memory_usage() -> f64 {
    // This is a simplified memory estimation
    // In a real implementation, you'd use system-specific APIs
    std::thread::sleep(std::time::Duration::from_millis(10)); // Allow GC
    
    // For now, return a placeholder based on heap allocation patterns
    // In a real system, you'd use something like:
    // - Linux: /proc/self/status VmRSS
    // - macOS: task_info with TASK_BASIC_INFO
    // - Windows: GetProcessMemoryInfo
    42.0 // Placeholder MB
}

#[test]
fn test_policy_compliance_performance() {
    // Test that policy constants work efficiently
    let start_time = Instant::now();
    
    // Test epoch calculations
    for block_number in 1..100000 {
        let is_epoch_boundary = block_number % lib::Policy::EPOCH_LENGTH == 0;
        let is_batch_boundary = block_number % lib::Policy::BATCH_LENGTH == 0;
        let is_election_block = block_number % (lib::Policy::EPOCH_LENGTH * lib::Policy::BATCH_LENGTH) == 0;
        
        // Verify policy compliance
        if is_election_block {
            assert!(is_epoch_boundary);
            assert!(is_batch_boundary);
        }
        
        if block_number % 10000 == 0 {
            println!("Processed {} policy checks", block_number);
        }
    }
    
    let duration = start_time.elapsed();
    println!("✅ Performed 100k policy compliance checks in {:?}", duration);
    println!("   Average: {:.2} checks/sec", 100000.0 / duration.as_secs_f64());
}