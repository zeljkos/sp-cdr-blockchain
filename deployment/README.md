# SP CDR Blockchain Deployment Guide

This guide covers building and deploying the SP CDR reconciliation blockchain across 3 virtual machines on your M4 Pro MacBook.

## Prerequisites

- **macOS with M4 Pro processor**
- **3 Virtual machines** (recommend Ubuntu 22.04 LTS ARM64)
- **Rust toolchain** installed on each VM
- **Network connectivity** between VMs
- **8GB RAM minimum** per VM for optimal performance

## Build Instructions

### 1. Clone and Build on Each VM

```bash
# Clone the repository
git clone <your-repo-url> sp_cdr_reconciliation_bc
cd sp_cdr_reconciliation_bc

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Build the project
cargo build --release
```

### 2. Verify Build Success

```bash
# Test cryptographic functionality
cargo run --bin test-real-crypto

# Expected output:
# 🔐 Testing Real Cryptographic Key Generation
# ✅ All keys are unique - cryptographically secure!
# ✅ Complete cryptographic pipeline is working!
# 🎯 Conclusion: SP CDR Blockchain cryptography is production-ready!
```

## Network Configuration

### VM Network Setup

Configure each VM with static IP addresses:

- **VM1 (Validator)**: `192.168.1.10`
- **VM2 (Validator)**: `192.168.1.11`
- **VM3 (Validator)**: `192.168.1.12`

### Firewall Configuration

Open required ports on each VM:

```bash
# Open blockchain ports
sudo ufw allow 8080  # P2P networking
sudo ufw allow 8081  # RPC API
sudo ufw allow 8082  # Consensus
```

## Deployment Steps

### 1. Generate Validator Keys (on each VM)

```bash
# Generate unique validator keys for each node
cargo run --bin sp-cdr-node -- generate-keys --output validator_keys.json

# Each VM should have unique keys
```

### 2. Create Network Configuration

Create `network_config.toml` on each VM:

```toml
[network]
listen_address = "/ip4/0.0.0.0/tcp/8080"
external_address = "/ip4/YOUR_VM_IP/tcp/8080"
bootstrap_peers = [
    "/ip4/192.168.1.10/tcp/8080",
    "/ip4/192.168.1.11/tcp/8080",
    "/ip4/192.168.1.12/tcp/8080"
]

[consensus]
validator_id = "YOUR_VALIDATOR_ID"
min_validators = 2
timeout_seconds = 10

[storage]
data_dir = "./blockchain_data"
```

### 3. Start Validator Nodes

**VM1 (Primary Bootstrap):**
```bash
cargo run --bin sp-cdr-node -- \
    --config network_config.toml \
    --validator-keys validator_keys.json \
    --listen 192.168.1.10:8080 \
    --bootstrap
```

**VM2 (Secondary Validator):**
```bash
cargo run --bin sp-cdr-node -- \
    --config network_config.toml \
    --validator-keys validator_keys.json \
    --listen 192.168.1.11:8080 \
    --connect 192.168.1.10:8080
```

**VM3 (Tertiary Validator):**
```bash
cargo run --bin sp-cdr-node -- \
    --config network_config.toml \
    --validator-keys validator_keys.json \
    --listen 192.168.1.12:8080 \
    --connect 192.168.1.10:8080
```

## Testing and Verification

### 1. Check Network Connectivity

```bash
# Test P2P connectivity between nodes
cargo run --example network_demo
```

### 2. Verify Consensus

Monitor consensus logs on each VM:
```bash
# Look for consensus establishment messages
tail -f blockchain_data/logs/consensus.log

# Expected: "Consensus established with 3 validators"
```

### 3. Test Smart Contract Deployment

```bash
# Deploy a test settlement contract
cargo run --bin cdr-pipeline-demo

# This will:
# 1. Deploy CDR settlement smart contract
# 2. Submit test CDR transactions
# 3. Execute settlement calculations
# 4. Verify ZK proofs
```

### 4. Validate Blockchain State

Check that all VMs have synchronized blockchain state:

```bash
# Check latest block height on each VM
curl http://192.168.1.10:8081/api/v1/block/latest
curl http://192.168.1.11:8081/api/v1/block/latest
curl http://192.168.1.12:8081/api/v1/block/latest

# Block heights should match across all nodes
```

### 5. Test CDR Settlement Pipeline

```bash
# Run the complete CDR reconciliation test
cargo run --bin cdr-pipeline-demo

# Expected output:
# ✅ ZK proof generated and verified
# ✅ Settlement calculation complete
# ✅ Multi-operator reconciliation successful
# 💰 Settlement: Operator A owes Operator B: €1,234.56
```

## Production Monitoring

### Health Check Endpoints

Monitor node health via HTTP endpoints:

```bash
# Node status
curl http://VM_IP:8081/health

# Network peers
curl http://VM_IP:8081/peers

# Consensus status
curl http://VM_IP:8081/consensus

# Smart contract state
curl http://VM_IP:8081/contracts
```

### Key Metrics to Monitor

1. **Block Production Rate**: Should produce blocks every ~10 seconds
2. **Peer Connections**: Each node should maintain connections to other validators
3. **Gas Usage**: Monitor smart contract execution costs
4. **ZK Proof Performance**: Proof generation should complete within 5 seconds
5. **Settlement Accuracy**: Cross-reference settlement calculations

### Log Files

Important log locations:
- **Consensus**: `blockchain_data/logs/consensus.log`
- **Network**: `blockchain_data/logs/network.log`
- **Smart Contracts**: `blockchain_data/logs/contracts.log`
- **CDR Processing**: `blockchain_data/logs/cdr_pipeline.log`

## Troubleshooting

### Common Issues

**Consensus Not Establishing:**
```bash
# Check validator keys are unique
grep "validator_id" validator_keys.json

# Verify network connectivity
ping 192.168.1.10
telnet 192.168.1.10 8080
```

**Smart Contract Failures:**
```bash
# Check gas limits
curl http://VM_IP:8081/gas/estimate -d '{"contract":"settlement"}'

# Verify ZK setup
ls -la blockchain_data/zkp_params/
```

**High Memory Usage:**
```bash
# Monitor Rust process memory
ps aux | grep sp-cdr-node

# Consider increasing VM RAM to 16GB for heavy ZK workloads
```

## Performance Optimization

### Production Tuning

1. **Increase file descriptors** (each VM):
   ```bash
   echo "* soft nofile 65536" >> /etc/security/limits.conf
   echo "* hard nofile 65536" >> /etc/security/limits.conf
   ```

2. **Optimize network buffers**:
   ```bash
   echo 'net.core.rmem_max = 16777216' >> /etc/sysctl.conf
   echo 'net.core.wmem_max = 16777216' >> /etc/sysctl.conf
   sysctl -p
   ```

3. **Enable release optimizations**:
   ```bash
   export RUSTFLAGS="-C target-cpu=native"
   cargo build --release
   ```

## Security Considerations

- **Validator keys** are cryptographically secure (verified by test-real-crypto)
- **Network traffic** uses libp2p with Noise encryption
- **ZK proofs** ensure CDR privacy without revealing sensitive data
- **Smart contracts** have gas metering to prevent DoS attacks

## Success Criteria

Your 3-VM blockchain deployment is successful when:

1. ✅ All 3 validators are connected and participating in consensus
2. ✅ Smart contracts deploy and execute successfully
3. ✅ CDR settlement pipeline completes end-to-end
4. ✅ ZK proofs verify correctly maintaining privacy
5. ✅ Gas metering prevents resource exhaustion
6. ✅ Block production is stable (~10 second intervals)

This demonstrates a **production-ready consortium blockchain** suitable for real telecom CDR reconciliation between operators.