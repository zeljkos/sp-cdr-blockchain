#!/bin/bash
# Debian VM Setup Script for SP CDR PoC
# Run this on both Debian VMs

set -e

# Get VM identity from command line argument
VM_ROLE=${1:-"unknown"}

if [[ "$VM_ROLE" != "tmobile" && "$VM_ROLE" != "orange" ]]; then
    echo "❌ Usage: $0 [tmobile|orange]"
    echo "   Example: $0 tmobile"
    exit 1
fi

echo "🐧 SP CDR Blockchain - Debian VM Setup ($VM_ROLE)"
echo "================================================"

# System info
echo "📊 System Information:"
echo "CPU Cores: $(nproc)"
echo "Memory: $(free -h | grep '^Mem:' | awk '{print $2}')"
echo "Disk Space: $(df -h / | tail -1 | awk '{print $4}')"
echo "OS: $(lsb_release -d | cut -f2- 2>/dev/null || echo 'Debian')"
echo "IP Address: $(hostname -I | awk '{print $1}')"

# Update system
echo "🔄 Updating system packages..."
sudo apt update && sudo apt upgrade -y

# Install dependencies
echo "📦 Installing dependencies..."
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    git \
    curl \
    htop \
    net-tools \
    jq \
    nginx \
    screen \
    rsync \
    tcpdump

# Install Rust
echo "🦀 Installing Rust toolchain..."
if ! command -v cargo &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    echo 'source ~/.cargo/env' >> ~/.bashrc
else
    echo "✅ Rust already installed"
fi

# Setup project directory
echo "📁 Setting up project directory..."
mkdir -p ~/sp-blockchain
cd ~/sp-blockchain

# Copy project (user will need to scp from Mac host)
if [ ! -d "sp_cdr_reconciliation_bc" ]; then
    echo "📥 Ready for project transfer from Mac host"
    echo "   Run on Mac: scp -r sp_cdr_reconciliation_bc user@$(hostname -I | awk '{print $1}'):~/sp-blockchain/"
else
    echo "✅ Project already exists"
    cd sp_cdr_reconciliation_bc
    cargo build --release
fi

# Configure based on VM role
if [[ "$VM_ROLE" == "tmobile" ]]; then
    echo "🇩🇪 Configuring T-Mobile Germany Validator..."

    mkdir -p ~/sp-blockchain/{tmobile-data,logs}

    # T-Mobile config
    cat > ~/sp-blockchain/tmobile-config.json << 'EOF'
{
  "network_id": "T-Mobile-DE",
  "validator_name": "tmobile-validator-001",
  "api_port": 8080,
  "p2p_port": 9080,
  "data_dir": "/home/$USER/sp-blockchain/tmobile-data",
  "log_level": "info",
  "peers": [
    "Orange-FR@192.168.1.20:9080",
    "Vodafone-UK@192.168.1.100:9080"
  ],
  "consensus": {
    "validator_key_file": "tmobile_validator.key",
    "bls_key_file": "tmobile_bls.key",
    "network_key_file": "tmobile_network.key"
  },
  "features": {
    "web_ui": true,
    "api_explorer": true,
    "metrics": true
  }
}
EOF

    # T-Mobile startup script
    cat > ~/sp-blockchain/start-tmobile.sh << 'EOF'
#!/bin/bash
cd ~/sp-blockchain/sp_cdr_reconciliation_bc

echo "🇩🇪 Starting T-Mobile Germany Validator..."
echo "==========================================="
echo "API: http://$(hostname -I | awk '{print $1}'):8080"
echo "P2P: $(hostname -I | awk '{print $1}'):9080"
echo ""

# Generate keys if they don't exist
if [ ! -f ~/sp-blockchain/tmobile-data/tmobile_validator.key ]; then
    echo "🔑 Generating validator keys..."
    ./target/release/sp-cdr-node generate-keys \
        --output-dir ~/sp-blockchain/tmobile-data \
        --network-id T-Mobile-DE
fi

# Start the node
./target/release/sp-cdr-node \
    --config ~/sp-blockchain/tmobile-config.json \
    2>&1 | tee ~/sp-blockchain/logs/tmobile-$(date +%Y%m%d-%H%M%S).log
EOF

    chmod +x ~/sp-blockchain/start-tmobile.sh

    # Web UI nginx config
    sudo tee /etc/nginx/sites-available/tmobile-ui << 'EOF'
server {
    listen 80;
    server_name _;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    location /api/ {
        proxy_pass http://127.0.0.1:8080/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
EOF
    sudo ln -sf /etc/nginx/sites-available/tmobile-ui /etc/nginx/sites-enabled/
    sudo systemctl reload nginx

elif [[ "$VM_ROLE" == "orange" ]]; then
    echo "🇫🇷 Configuring Orange France Validator + Load Testing..."

    mkdir -p ~/sp-blockchain/{orange-data,logs,test-data}

    # Orange config
    cat > ~/sp-blockchain/orange-config.json << 'EOF'
{
  "network_id": "Orange-FR",
  "validator_name": "orange-validator-001",
  "api_port": 8080,
  "p2p_port": 9080,
  "data_dir": "/home/$USER/sp-blockchain/orange-data",
  "log_level": "info",
  "peers": [
    "T-Mobile-DE@192.168.1.10:9080",
    "Vodafone-UK@192.168.1.100:9080"
  ],
  "consensus": {
    "validator_key_file": "orange_validator.key",
    "bls_key_file": "orange_bls.key",
    "network_key_file": "orange_network.key"
  },
  "features": {
    "load_testing": true,
    "cdr_generator": true,
    "metrics": true
  }
}
EOF

    # Orange startup script
    cat > ~/sp-blockchain/start-orange.sh << 'EOF'
#!/bin/bash
cd ~/sp-blockchain/sp_cdr_reconciliation_bc

echo "🇫🇷 Starting Orange France Validator + Load Testing..."
echo "===================================================="
echo "API: http://$(hostname -I | awk '{print $1}'):8080"
echo "P2P: $(hostname -I | awk '{print $1}'):9080"
echo ""

# Generate keys if they don't exist
if [ ! -f ~/sp-blockchain/orange-data/orange_validator.key ]; then
    echo "🔑 Generating validator keys..."
    ./target/release/sp-cdr-node generate-keys \
        --output-dir ~/sp-blockchain/orange-data \
        --network-id Orange-FR
fi

# Start the node
./target/release/sp-cdr-node \
    --config ~/sp-blockchain/orange-config.json \
    2>&1 | tee ~/sp-blockchain/logs/orange-$(date +%Y%m%d-%H%M%S).log
EOF

    chmod +x ~/sp-blockchain/start-orange.sh

    # Load testing script
    cat > ~/sp-blockchain/load-test.sh << 'EOF'
#!/bin/bash
echo "🧪 SP CDR Load Testing Tool"
echo "=========================="

# Test data generation
generate_cdr_data() {
    local home_network=$1
    local visited_network=$2
    local month=$3

    # Generate realistic CDR data
    python3 << PYTHON
import json
import random
import hashlib
from datetime import datetime, timedelta

# Generate sample CDR batch
cdr_batch = {
    "period": "$month",
    "home_network": "$home_network",
    "visited_network": "$visited_network",
    "total_charges": random.randint(10000, 100000),  # cents
    "record_count": random.randint(500, 2000),
    "call_minutes": random.randint(50000, 200000),
    "data_mb": random.randint(100000, 500000),
    "sms_count": random.randint(5000, 25000),
    "timestamp": datetime.now().isoformat(),
    "batch_id": hashlib.sha256(f"$home_network-$visited_network-$month".encode()).hexdigest()[:16]
}

# Mock ZK proof (in real system, this would be generated by ZK prover)
zk_proof = {
    "proof_type": "groth16_bn254",
    "public_inputs": [cdr_batch["total_charges"], cdr_batch["record_count"]],
    "proof_data": "mock_zk_proof_" + cdr_batch["batch_id"],
    "verification_key_id": "cdr_privacy_vk_v1"
}

cdr_submission = {
    "cdr_batch": cdr_batch,
    "zk_proof": zk_proof
}

print(json.dumps(cdr_submission, indent=2))
PYTHON
}

# Load test scenarios
echo "📊 Running CDR load tests..."

# Test 1: T-Mobile -> Orange roaming
echo "Test 1: T-Mobile to Orange roaming CDRs"
TMO_TO_ORG=$(generate_cdr_data "T-Mobile-DE" "Orange-FR" "2024-01")
curl -s -X POST http://192.168.1.10:8080/api/v1/cdr/submit \
    -H "Content-Type: application/json" \
    -d "$TMO_TO_ORG" | jq .

# Test 2: Orange -> Vodafone roaming
echo "Test 2: Orange to Vodafone roaming CDRs"
ORG_TO_VOD=$(generate_cdr_data "Orange-FR" "Vodafone-UK" "2024-01")
curl -s -X POST http://192.168.1.100:8080/api/v1/cdr/submit \
    -H "Content-Type: application/json" \
    -d "$ORG_TO_VOD" | jq .

# Test 3: Vodafone -> T-Mobile roaming
echo "Test 3: Vodafone to T-Mobile roaming CDRs"
VOD_TO_TMO=$(generate_cdr_data "Vodafone-UK" "T-Mobile-DE" "2024-01")
curl -s -X POST http://192.168.1.20:8080/api/v1/cdr/submit \
    -H "Content-Type: application/json" \
    -d "$VOD_TO_TMO" | jq .

echo ""
echo "✅ Load test complete. Check validator logs for processing results."
EOF

    chmod +x ~/sp-blockchain/load-test.sh

    # Install Python for load testing
    sudo apt install -y python3 python3-pip
    pip3 install requests
fi

# Create monitoring script
cat > ~/sp-blockchain/monitor.sh << 'EOF'
#!/bin/bash
echo "📊 SP CDR Blockchain Monitor"
echo "============================"

while true; do
    clear
    echo "🕒 $(date)"
    echo ""

    # Local node status
    echo "🏠 Local Node Status:"
    curl -s http://localhost:8080/api/v1/status | jq . 2>/dev/null || echo "❌ Local node not responding"
    echo ""

    # Peer status
    echo "🤝 Peer Status:"
    if [[ "$(hostname -I | awk '{print $1}')" == "192.168.1.10" ]]; then
        # T-Mobile VM checking others
        echo "Orange (192.168.1.20):"
        curl -s --connect-timeout 3 http://192.168.1.20:8080/api/v1/status | jq .network_id 2>/dev/null || echo "  ❌ Offline"
        echo "Vodafone (192.168.1.100):"
        curl -s --connect-timeout 3 http://192.168.1.100:8080/api/v1/status | jq .network_id 2>/dev/null || echo "  ❌ Offline"
    elif [[ "$(hostname -I | awk '{print $1}')" == "192.168.1.20" ]]; then
        # Orange VM checking others
        echo "T-Mobile (192.168.1.10):"
        curl -s --connect-timeout 3 http://192.168.1.10:8080/api/v1/status | jq .network_id 2>/dev/null || echo "  ❌ Offline"
        echo "Vodafone (192.168.1.100):"
        curl -s --connect-timeout 3 http://192.168.1.100:8080/api/v1/status | jq .network_id 2>/dev/null || echo "  ❌ Offline"
    fi

    echo ""
    echo "📈 Blockchain Stats:"
    curl -s http://localhost:8080/api/v1/stats | jq . 2>/dev/null || echo "❌ Stats not available"

    sleep 10
done
EOF
chmod +x ~/sp-blockchain/monitor.sh

echo ""
echo "✅ Debian VM ($VM_ROLE) setup complete!"
echo ""
echo "📋 Next steps:"
echo "1. Copy project from Mac: scp -r sp_cdr_reconciliation_bc user@$(hostname -I | awk '{print $1}'):~/sp-blockchain/"
echo "2. Build project: cd ~/sp-blockchain/sp_cdr_reconciliation_bc && cargo build --release"
if [[ "$VM_ROLE" == "tmobile" ]]; then
    echo "3. Start T-Mobile validator: ~/sp-blockchain/start-tmobile.sh"
    echo "4. Access Web UI: http://$(hostname -I | awk '{print $1}')"
elif [[ "$VM_ROLE" == "orange" ]]; then
    echo "3. Start Orange validator: ~/sp-blockchain/start-orange.sh"
    echo "4. Run load tests: ~/sp-blockchain/load-test.sh"
fi
echo "5. Monitor network: ~/sp-blockchain/monitor.sh"