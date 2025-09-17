# SP CDR Reconciliation Blockchain - PoC Deployment Guide

## 🖥️ **Hardware Setup**

```
MacBook M4 Pro (Host)
├── Debian VM #1 (T-Mobile-DE)     IP: 192.168.1.10
├── Debian VM #2 (Orange-FR)       IP: 192.168.1.20
└── VMware Fusion Network

Physical Machine (Vodafone-UK)     IP: 192.168.1.100
├── 16GB RAM, 4 CPU cores
└── Settlement Monitor + Explorer
```

## 🚀 **Step-by-Step Deployment**

### **Phase 1: Setup Debian VMs (on Mac)**

```bash
# VM #1 - T-Mobile Germany
./deploy/vm-setup.sh tmobile

# VM #2 - Orange France
./deploy/vm-setup.sh orange
```

### **Phase 2: Setup Physical Machine**

```bash
# On physical machine
./deploy/physical-setup.sh
```

### **Phase 3: Deploy Project Code**

```bash
# From Mac host, copy to all machines
scp -r sp_cdr_reconciliation_bc user@192.168.1.10:~/sp-blockchain/
scp -r sp_cdr_reconciliation_bc user@192.168.1.20:~/sp-blockchain/
scp -r sp_cdr_reconciliation_bc user@192.168.1.100:~/sp-blockchain/
```

### **Phase 4: Build & Start Validators**

```bash
# On each machine, build the project
cd ~/sp-blockchain/sp_cdr_reconciliation_bc
cargo build --release

# T-Mobile VM (192.168.1.10)
~/sp-blockchain/start-tmobile.sh

# Orange VM (192.168.1.20)
~/sp-blockchain/start-orange.sh

# Physical Machine (192.168.1.100)
~/sp-blockchain/setup-explorer.sh
~/sp-blockchain/start-vodafone.sh
```

## 📊 **PoC Demo Workflow**

### **1. Network Status Check**

```bash
# Test connectivity between all nodes
~/sp-blockchain/network-test.sh
```

### **2. Load CDR Data**

```bash
# On Orange VM - run load tests
~/sp-blockchain/load-test.sh

# This will generate and submit:
# - T-Mobile → Orange roaming CDRs (€500)
# - Orange → Vodafone roaming CDRs (€750)
# - Vodafone → T-Mobile roaming CDRs (€250)
```

### **3. Monitor Settlement Process**

```bash
# On physical machine - watch settlements
~/sp-blockchain/settlement-monitor.sh

# Shows live dashboard:
# - Network status
# - Pending CDR batches
# - Auto-triggered settlements
# - Savings calculations
```

### **4. View Results**

Open blockchain explorer: `http://192.168.1.100/`

**Expected Settlement Results:**
```json
{
  "before_netting": {
    "bilateral_settlements": 6,
    "total_amount": "€1,500.00"
  },
  "after_netting": {
    "net_settlements": 2,
    "total_amount": "€500.00"
  },
  "savings": {
    "settlement_reduction": "66.7%",
    "amount_reduction": "66.7%",
    "fee_savings": "€200.00"
  }
}
```

## 🔍 **PoC Verification Points**

### **Privacy Preservation**
- ✅ CDR data stays encrypted on each validator
- ✅ Only settlement amounts are revealed
- ✅ ZK proofs verify correctness without data exposure

### **Triangular Netting**
- ✅ 6 bilateral settlements → 2 net settlements
- ✅ €1,500 gross → €500 net (66% reduction)
- ✅ No central clearing house needed

### **Consensus & Security**
- ✅ Byzantine fault tolerant (1/3 can fail)
- ✅ BLS signature aggregation
- ✅ Immutable settlement audit trail

### **Real-time Processing**
- ✅ CDR submission via REST API
- ✅ Automatic settlement triggering
- ✅ Live monitoring dashboard

## 🎯 **Demo Script**

```bash
# 1. Start all three validators in separate terminals

# Terminal 1 - T-Mobile VM
ssh user@192.168.1.10
~/sp-blockchain/start-tmobile.sh

# Terminal 2 - Orange VM
ssh user@192.168.1.20
~/sp-blockchain/start-orange.sh

# Terminal 3 - Physical Machine
ssh user@192.168.1.100
~/sp-blockchain/start-vodafone.sh

# 2. In Terminal 4 - Start settlement monitor
ssh user@192.168.1.100
~/sp-blockchain/settlement-monitor.sh

# 3. In Terminal 5 - Load test data
ssh user@192.168.1.20
~/sp-blockchain/load-test.sh

# 4. Watch the magic happen:
# - CDRs submitted with ZK proofs
# - Validators reach consensus
# - Settlement auto-triggered
# - Triangular netting applied
# - Savings calculated and displayed
```

## 📱 **Web Interfaces**

- **T-Mobile Dashboard**: `http://192.168.1.10/`
- **Orange Dashboard**: `http://192.168.1.20/`
- **Blockchain Explorer**: `http://192.168.1.100/`
- **Settlement Monitor**: Terminal-based (most detailed)

## 💡 **PoC Success Criteria**

1. **✅ Multi-node Operation**: 3 validators running on separate machines
2. **✅ P2P Consensus**: Validators communicate and reach consensus
3. **✅ CDR Privacy**: Data encrypted, only proofs shared
4. **✅ Triangular Netting**: Automatic settlement reduction
5. **✅ Real Cryptography**: BLS signatures + ZK proofs verified
6. **✅ Settlement Savings**: Demonstrable cost reduction
7. **✅ Monitoring**: Real-time visibility into the process

## 🔧 **Troubleshooting**

### **Network Issues**
```bash
# Check if validators can reach each other
ping 192.168.1.10  # from other machines
ping 192.168.1.20
ping 192.168.1.100

# Test API endpoints
curl http://192.168.1.10:8080/api/v1/status
```

### **Build Issues**
```bash
# Ensure Rust is installed
source ~/.cargo/env
rustc --version

# Clean rebuild
cargo clean && cargo build --release
```

### **Settlement Not Triggering**
```bash
# Check if enough CDR batches are pending
curl http://localhost:8080/api/v1/cdr/batches

# Manually trigger settlement
curl -X POST http://localhost:8080/api/v1/settlement/process \
  -H "Content-Type: application/json" \
  -d '{"period":"2024-01","networks":["T-Mobile-DE","Vodafone-UK","Orange-FR"]}'
```

This PoC demonstrates the complete SP CDR reconciliation workflow with **real privacy-preserving settlements** between telecom operators, solving the "zillions of CDR" problem through automated triangular netting!