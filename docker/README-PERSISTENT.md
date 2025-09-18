# SP CDR Blockchain - Persistent Storage Setup

## 🚀 Optimized Architecture

This setup addresses all the issues with the original Docker configuration:

### ✅ **Problems Solved**
1. **Persistent Storage**: Uses real MDBX database instead of in-memory storage
2. **Efficient Building**: Pre-builds binaries on host, copies to lightweight containers
3. **Space Optimization**: No build tools in containers, smaller images
4. **Lower Thresholds**: Settlement threshold reduced to €1 (from €100) for demo
5. **External Volumes**: Blockchain data persists on host filesystem

### 📊 **Key Improvements**

| Aspect | Before | After |
|--------|--------|--------|
| **Storage** | In-memory (lost on restart) | Persistent MDBX on host |
| **Build** | Inside container (slow, large) | Pre-built on host (fast, small) |
| **Settlement** | €100 threshold (too high) | €1 threshold (demo-friendly) |
| **Data Access** | Container-only | Direct host access |
| **Space** | ~2GB per container | ~200MB per container |

## 🔧 **Usage Instructions**

### **Quick Start**
```bash
cd docker
./start-persistent.sh
```

### **Manual Steps**
```bash
# 1. Build on host (faster)
cargo build --release

# 2. Start with persistent storage
cd docker
docker-compose -f docker-compose.persistent.yml up -d
```

### **Inspection Commands**
```bash
# View blockchain blocks (now persistent!)
docker exec sp-validator-1 ./target/release/sp-cdr-node inspect --target blocks

# View transactions
docker exec sp-validator-1 ./target/release/sp-cdr-node inspect --target transactions

# View CDR processing status
docker exec sp-validator-1 ./target/release/sp-cdr-node inspect --target cdrs

# View system statistics
docker exec sp-validator-1 ./target/release/sp-cdr-node inspect --target stats
```

### **Direct Data Access**
```bash
# Blockchain data is now accessible from host
ls -la ../persistent_data/

# MDBX databases
ls -la ../persistent_data/validator-1/blockchain/

# ZK keys
ls -la ../persistent_data/validator-1/zkp_keys/
```

## 📁 **Directory Structure**

```
sp_cdr_reconciliation_bc/
├── docker/
│   ├── start-persistent.sh          # Optimized startup
│   ├── docker-compose.persistent.yml # Persistent volumes config
│   ├── Dockerfile.prebuilt          # Lightweight container
│   └── clean.sh                     # Complete cleanup
├── persistent_data/                  # Created automatically
│   ├── validator-1/
│   │   ├── blockchain/              # MDBX database files
│   │   └── zkp_keys/               # ZK proving/verifying keys
│   ├── validator-2/
│   ├── validator-3/
│   └── shared_zkp_keys/            # Keys shared between validators
└── target/release/
    └── sp-cdr-node                 # Pre-built binary
```

## 🔍 **Monitoring & Debugging**

### **Container Status**
```bash
docker-compose -f docker-compose.persistent.yml ps
docker-compose -f docker-compose.persistent.yml logs -f validator-1
```

### **Blockchain Status**
```bash
# Check for persistent blocks
docker exec sp-validator-1 ./target/release/sp-cdr-node inspect --target blocks

# View settlement processing (now triggers at €1 instead of €100)
docker exec sp-validator-1 ./target/release/sp-cdr-node inspect --target settlements
```

### **Data Persistence Verification**
```bash
# Stop containers
docker-compose -f docker-compose.persistent.yml down

# Start again
docker-compose -f docker-compose.persistent.yml up -d

# Check if blockchain data persisted
docker exec sp-validator-1 ./target/release/sp-cdr-node inspect --target stats
```

## 🧹 **Cleanup Options**

### **Soft Cleanup** (keep persistent data)
```bash
docker-compose -f docker-compose.persistent.yml down
```

### **Complete Cleanup** (remove all data)
```bash
./clean.sh
```

## 💡 **Benefits of This Setup**

1. **🔄 Persistent Data**: Blockchain state survives container restarts
2. **⚡ Fast Builds**: No compilation inside containers
3. **💾 Space Efficient**: Smaller container images
4. **🔍 Direct Access**: Inspect MDBX files directly from host
5. **🎯 Demo Ready**: Lower thresholds trigger settlements quickly
6. **🏗️ Production Ready**: Real MDBX storage like Albatross

## 🚨 **Important Notes**

- **Host Dependencies**: Requires `cargo build --release` on host
- **MDBX Database**: Real persistent storage, not just files
- **Settlement Threshold**: Now €1 instead of €100 for demo
- **Data Location**: `../persistent_data/` on host filesystem
- **Container Restart**: Data persists across container lifecycle

This setup gives you a production-like blockchain with the efficiency of development-friendly tooling! 🎉