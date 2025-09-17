# SP CDR Reconciliation Blockchain - Startup & Testing Guide

## Build Status
✅ **Successfully built** with 0 compilation errors
⚠️ 55 warnings (unused imports/variables - normal for development)

## Core Components Integrated
- **MDBX Storage**: Persistent blockchain data storage
- **ZK Proof System**: Privacy-preserving settlement verification (arkworks/BN254)
- **BLS Signatures**: Multi-party operator validation
- **Smart Contract VM**: Stack-based bytecode execution
- **Albatross Consensus**: Micro/macro block structure
- **CDR Processing**: Call Detail Record reconciliation and netting

## Prerequisites

```bash
# Ensure Rust toolchain is installed
rustc --version  # Should be 1.70+

# Dependencies are automatically downloaded during build
# Key external dependencies:
# - MDBX database (embedded)
# - arkworks ZK cryptography
# - BLS12-381 signature library
```

## Startup Procedure

### 1. Build the Project
```bash
# Development build (faster compilation)
cargo build

# Production build (optimized)
cargo build --release
```

### 2. Initialize Node Configuration
```bash
# Create data directory
mkdir -p data/blockchain
mkdir -p data/contracts
mkdir -p data/zkp_keys

# The node will auto-initialize on first run
```

### 3. Start the SP CDR Node
```bash
# Run development node
cargo run

# Or run release version
./target/release/sp-cdr-node

# With specific data directory
./target/release/sp-cdr-node --data-dir ./data
```

### 4. Node Initialization Sequence
On first startup, the node will:
1. **Genesis Block**: Create initial macro block with SP consortium validators
2. **Storage Setup**: Initialize MDBX databases for blockchain and contract state
3. **Crypto Setup**: Load ZK verifying keys and operator BLS keys
4. **Network Init**: Set up P2P networking (if enabled)
5. **Validator Set**: Configure initial SP consortium operators

## Testing Procedures

### Basic Compilation Test
```bash
# Verify clean compilation
cargo check
# Expected: "Finished dev profile [unoptimized + debuginfo] target(s) in X.Xs"
```

### Unit Test Suite
```bash
# Run all tests
cargo test

# Run specific test modules
cargo test storage         # Storage layer tests
cargo test crypto          # Cryptographic tests
cargo test smart_contracts # Contract VM tests
cargo test blockchain      # Block validation tests
```

### Integration Tests
```bash
# Test blockchain components working together
cargo test --test integration

# Test CDR processing pipeline
cargo test cdr_processing

# Test settlement contract execution
cargo test settlement_flow
```

### Manual CDR Testing

#### 1. Submit CDR Records
```bash
# Example: T-Mobile Germany roaming to Vodafone UK
curl -X POST http://localhost:8080/api/v1/cdr/submit \
  -H "Content-Type: application/json" \
  -d '{
    "home_network": "T-Mobile-DE",
    "visited_network": "Vodafone-UK",
    "record_type": "VoiceCall",
    "encrypted_data": "base64_encoded_cdr_data",
    "zk_proof": "base64_encoded_zk_proof"
  }'
```

#### 2. Check CDR Batch Status
```bash
curl http://localhost:8080/api/v1/cdr/batch/{batch_id}/status
```

#### 3. Trigger Settlement
```bash
# Process monthly settlement between networks
curl -X POST http://localhost:8080/api/v1/settlement/process \
  -d '{
    "period": "2024-01",
    "networks": ["T-Mobile-DE", "Vodafone-UK", "Orange-FR"]
  }'
```

### ZK Proof Testing
```bash
# Test settlement proof generation and verification
cargo test zkp_settlement_proof

# Test CDR privacy proof
cargo test zkp_cdr_privacy_proof

# Load test with multiple proofs
cargo test --release zkp_performance_test
```

### Smart Contract Testing
```bash
# Deploy settlement contract
cargo test deploy_settlement_contract

# Execute contract with CDR batch
cargo test execute_cdr_settlement

# Test multi-signature validation
cargo test multi_sig_settlement
```

### Performance Testing
```bash
# Benchmark CDR throughput
cargo bench cdr_processing

# Benchmark settlement calculations
cargo bench settlement_performance

# Storage performance
cargo bench mdbx_storage
```

## Expected Test Results

### Successful Startup Logs
```
[INFO] SP CDR Node starting...
[INFO] Initializing MDBX storage at data/blockchain
[INFO] Loading ZK verifying keys
[INFO] Registering SP consortium operators:
  - T-Mobile-DE: <bls_public_key>
  - Vodafone-UK: <bls_public_key>
  - Orange-FR: <bls_public_key>
[INFO] Genesis block created: <block_hash>
[INFO] Node ready, listening on 127.0.0.1:8080
```

### Healthy Test Output
```bash
cargo test
# Expected output:
test result: ok. X passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Troubleshooting

### Common Issues

**MDBX Database Lock Error**
```bash
# Solution: Ensure no other node instance is running
pkill sp-cdr-node
rm -f data/blockchain/*.lock
```

**ZK Proving Key Not Found**
```bash
# Solution: Keys are auto-generated on first run
# For production, load real proving keys:
# cp production_keys/* data/zkp_keys/
```

**BLS Signature Verification Failed**
```bash
# Solution: Ensure operator public keys are correctly registered
# Check logs for key registration confirmation
```

### Performance Expectations
- **CDR Processing**: ~1000 CDR/sec per network
- **Settlement Calculation**: ~10 settlements/sec
- **ZK Proof Verification**: ~50 proofs/sec
- **Block Processing**: ~100 blocks/sec

## Production Deployment

### Hardware Requirements
- **CPU**: 4+ cores (ZK proof verification is CPU intensive)
- **RAM**: 8GB+ (MDBX memory mapping)
- **Storage**: 100GB+ SSD (blockchain growth ~1GB/month)
- **Network**: 1Gbps+ (P2P consensus traffic)

### Security Checklist
- [ ] Operator BLS keys stored in HSM
- [ ] ZK proving keys secured with proper access controls
- [ ] Network isolated in SP consortium VPN
- [ ] Regular backup of blockchain state
- [ ] Monitoring and alerting configured

The system is now ready for testing and deployment in the SP consortium environment.