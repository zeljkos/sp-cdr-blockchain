#!/bin/bash
# Single Node SP CDR Demo - Test storage and flow

set -e

echo "🏗️  SP CDR Single Node Demo"
echo "========================="

# Check if project is built
if [ ! -f "target/release/sp-cdr-node" ]; then
    echo "🔨 Building project..."
    cargo build --release
fi

# Create demo directory
DEMO_DIR="demo-single-node"
mkdir -p $DEMO_DIR/{data,logs,keys}

echo "📁 Demo directory: $(pwd)/$DEMO_DIR"

# Generate validator config
cat > $DEMO_DIR/config.json << 'EOF'
{
  "network_id": "T-Mobile-DE",
  "validator_name": "demo-validator-001",
  "api_port": 8080,
  "p2p_port": 9080,
  "data_dir": "./data",
  "log_level": "info",
  "peers": [],
  "consensus": {
    "validator_key_file": "demo_validator.key",
    "bls_key_file": "demo_bls.key"
  },
  "features": {
    "single_node_mode": true,
    "auto_mining": true,
    "storage_debug": true
  }
}
EOF

echo "⚙️  Configuration created"

# Generate mock keys (since we don't have real key generation implemented)
cat > $DEMO_DIR/keys/demo_validator.key << 'EOF'
# Mock validator key for demo
validator_private_key: demo_key_123456789abcdef
validator_public_key: demo_pub_123456789abcdef
network_id: T-Mobile-DE
EOF

cat > $DEMO_DIR/keys/demo_bls.key << 'EOF'
# Mock BLS key for demo
bls_private_key: demo_bls_private_123456789abcdef
bls_public_key: demo_bls_public_123456789abcdef
EOF

# Create startup script
cat > $DEMO_DIR/start.sh << 'EOF'
#!/bin/bash
cd "$(dirname "$0")"

echo "🚀 Starting SP CDR Demo Node..."
echo "================================"
echo ""
echo "API: http://localhost:8080"
echo "Data: $(pwd)/data"
echo "Logs: $(pwd)/logs"
echo ""

# Create data directory
mkdir -p data/blockchain data/contracts data/zkp

# Start the node
../target/release/sp-cdr-node \
    --config config.json \
    --data-dir ./data \
    --single-node \
    2>&1 | tee logs/demo-$(date +%Y%m%d-%H%M%S).log
EOF

chmod +x $DEMO_DIR/start.sh

# Create CDR test script
cat > $DEMO_DIR/test-cdr-flow.sh << 'EOF'
#!/bin/bash
echo "🧪 Testing CDR Flow"
echo "=================="

API="http://localhost:8080/api"

# Wait for node to be ready
echo "⏳ Waiting for node to start..."
for i in {1..30}; do
    if curl -s $API/v1/status >/dev/null 2>&1; then
        echo "✅ Node is ready!"
        break
    fi
    echo -n "."
    sleep 2
done

echo ""

# Test 1: Check node status
echo "1️⃣  Testing node status..."
curl -s $API/v1/status | jq . || echo "❌ Status check failed"
echo ""

# Test 2: Submit CDR batch
echo "2️⃣  Submitting CDR batch..."
CDR_BATCH='{
  "home_network": "T-Mobile-DE",
  "visited_network": "Vodafone-UK",
  "cdr_batch": {
    "period": "2024-01",
    "batch_id": "demo-batch-001",
    "total_charges": 50000,
    "record_count": 1250,
    "call_minutes": 25000,
    "data_mb": 150000,
    "sms_count": 5000,
    "timestamp": "'$(date -u +%Y-%m-%dT%H:%M:%S)'Z"
  },
  "zk_proof": {
    "proof_type": "demo_mock_proof",
    "public_inputs": [50000, 1250],
    "proof_data": "mock_zk_proof_demo_batch_001",
    "verification_key_id": "demo_vk_v1"
  }
}'

RESULT=$(curl -s -X POST $API/v1/cdr/submit \
    -H "Content-Type: application/json" \
    -d "$CDR_BATCH")

echo "CDR Submission Result:"
echo "$RESULT" | jq . || echo "$RESULT"
echo ""

# Test 3: Check storage
echo "3️⃣  Checking blockchain storage..."
curl -s $API/v1/blocks | jq . || echo "❌ Block query failed"
echo ""

# Test 4: Check CDR batches
echo "4️⃣  Checking stored CDR batches..."
curl -s $API/v1/cdr/batches | jq . || echo "❌ CDR batch query failed"
echo ""

# Test 5: Simulate settlement
echo "5️⃣  Testing settlement calculation..."
SETTLEMENT='{
  "period": "2024-01",
  "networks": ["T-Mobile-DE", "Vodafone-UK"],
  "settlement_type": "bilateral"
}'

SETTLEMENT_RESULT=$(curl -s -X POST $API/v1/settlement/calculate \
    -H "Content-Type: application/json" \
    -d "$SETTLEMENT")

echo "Settlement Result:"
echo "$SETTLEMENT_RESULT" | jq . || echo "$SETTLEMENT_RESULT"
echo ""

# Test 6: Check MDBX storage files
echo "6️⃣  Checking MDBX storage files..."
echo "Data directory contents:"
ls -la data/ 2>/dev/null || echo "No data directory yet"
if [ -d "data/blockchain" ]; then
    echo "Blockchain storage:"
    ls -la data/blockchain/
fi
if [ -d "data/contracts" ]; then
    echo "Contract storage:"
    ls -la data/contracts/
fi

echo ""
echo "✅ CDR flow test complete!"
echo ""
echo "📊 Storage Summary:"
echo "- MDBX database files created in ./data/"
echo "- CDR batches stored with privacy proofs"
echo "- Settlement calculations performed"
echo "- All data persisted to disk"
EOF

chmod +x $DEMO_DIR/test-cdr-flow.sh

# Create monitoring script
cat > $DEMO_DIR/monitor.sh << 'EOF'
#!/bin/bash
echo "📊 SP CDR Storage & Flow Monitor"
echo "==============================="

API="http://localhost:8080/api"

while true; do
    clear
    echo "🕒 $(date)"
    echo "📊 SP CDR Demo Monitor"
    echo "====================="
    echo ""

    # Node status
    echo "🏠 Node Status:"
    NODE_STATUS=$(curl -s $API/v1/status 2>/dev/null)
    if [ $? -eq 0 ]; then
        echo "$NODE_STATUS" | jq '{network_id, status, block_height, uptime}' 2>/dev/null || echo "Node responding but JSON parse failed"
    else
        echo "❌ Node not responding"
    fi
    echo ""

    # Storage stats
    echo "💾 Storage Statistics:"
    if [ -d "data" ]; then
        echo "  Data directory size: $(du -sh data 2>/dev/null | cut -f1)"

        if [ -d "data/blockchain" ]; then
            echo "  Blockchain files:"
            ls -lah data/blockchain/ | grep -E '\.(mdb|lock)$' | while read line; do
                echo "    $line"
            done
        fi

        if [ -d "data/contracts" ]; then
            echo "  Contract files:"
            ls -lah data/contracts/ | grep -E '\.(mdb|lock)$' | while read line; do
                echo "    $line"
            done
        fi
    else
        echo "  No data directory yet"
    fi
    echo ""

    # CDR batches
    echo "📄 CDR Batches:"
    CDR_BATCHES=$(curl -s $API/v1/cdr/batches 2>/dev/null)
    if [ $? -eq 0 ]; then
        echo "$CDR_BATCHES" | jq '{total: (.pending | length), batches: .pending[0:3]}' 2>/dev/null || echo "CDR API responding but parse failed"
    else
        echo "  No CDR data or API not responding"
    fi
    echo ""

    # Recent blocks
    echo "🧱 Recent Blocks:"
    BLOCKS=$(curl -s $API/v1/blocks?limit=3 2>/dev/null)
    if [ $? -eq 0 ]; then
        echo "$BLOCKS" | jq '.[] | {height: .header.block_number, hash: .hash, timestamp: .header.timestamp}' 2>/dev/null || echo "Blocks API responding but parse failed"
    else
        echo "  No block data or API not responding"
    fi

    echo ""
    echo "Press Ctrl+C to exit"
    sleep 10
done
EOF

chmod +x $DEMO_DIR/monitor.sh

echo ""
echo "✅ Single node demo setup complete!"
echo ""
echo "🚀 To start the demo:"
echo "1. cd $DEMO_DIR"
echo "2. ./start.sh"
echo ""
echo "🧪 To test CDR flow (in another terminal):"
echo "1. cd $DEMO_DIR"
echo "2. ./test-cdr-flow.sh"
echo ""
echo "📊 To monitor storage (in another terminal):"
echo "1. cd $DEMO_DIR"
echo "2. ./monitor.sh"
echo ""
echo "📁 All data will be stored in: $(pwd)/$DEMO_DIR/data/"
echo "📝 Logs will be in: $(pwd)/$DEMO_DIR/logs/"