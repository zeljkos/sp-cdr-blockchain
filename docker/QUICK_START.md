# SP CDR Blockchain - Quick Start Guide

## 🚀 Starting from Scratch

### Prerequisites
- Docker & Docker Compose installed
- Rust toolchain (for building binaries)
- Minimum 4GB RAM, 10GB disk space

### Step 1: Build Release Binaries
```bash
# From project root
cargo build --release
```

### Step 2: Start the Blockchain Network
**Note**: ZK trusted setup is now automated! The start script will automatically generate keys if they don't exist.
```bash
cd docker
./start-persistent.sh
```

## 📊 Verify System Status

### Check Containers
```bash
docker compose -f docker-compose.persistent.yml ps
```

### View Logs
```bash
# All validators
docker compose -f docker-compose.persistent.yml logs -f

# Specific validator
docker compose -f docker-compose.persistent.yml logs -f validator-1
```

### Health Check
Look for these success indicators:
- ✅ `✅ ZK system initialized with real keys`
- ✅ `🌐 Network manager initialized`
- ✅ `💾 Storage initialized`
- ✅ `📋 Added sample CDR batch`
- ✅ `discovered peer on address`

## 🛑 Stop System
```bash
docker compose -f docker-compose.persistent.yml down
```

## 🔧 Troubleshooting

### If ZK Keys Are Missing:
The start script now handles this automatically! But if you need to manually regenerate:
```bash
# Check if keys exist
ls -la ../persistent_data/shared_zkp_keys/

# Manual regeneration (if needed):
cd ..
./target/release/trusted-setup-demo
cp -r test_ceremony_keys/* persistent_data/shared_zkp_keys/
cd docker
```

### If Containers Fail to Start:
```bash
# Clean rebuild
docker compose -f docker-compose.persistent.yml down
docker compose -f docker-compose.persistent.yml build --no-cache
docker compose -f docker-compose.persistent.yml up -d
```

### If Network Formation Fails:
- Wait 30-60 seconds for peer discovery
- Check logs for `discovered peer` messages
- Verify all 3 containers are running

## 📁 Data Persistence

All blockchain data is stored in:
```
../persistent_data/
├── validator-1/blockchain/    # Node 1 MDBX database
├── validator-2/blockchain/    # Node 2 MDBX database
├── validator-3/blockchain/    # Node 3 MDBX database
└── shared_zkp_keys/          # ZK ceremony keys
```

## 🎯 What You Get

- **3-Node Blockchain**: Full consensus network
- **Real ZK Proofs**: Groth16 with BN254 curve
- **MDBX Storage**: 2TB persistent databases
- **Settlement Processing**: €100M+ capacity
- **Network Discovery**: Automatic peer formation

---

**⚠️ IMPORTANT**: The ZK keys generated are for **DEVELOPMENT ONLY**. Production requires multi-party trusted setup ceremony with all network operators.