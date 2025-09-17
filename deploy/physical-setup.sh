#!/bin/bash
# Physical Machine Setup Script for SP CDR PoC
# This machine runs Vodafone-UK validator + monitoring tools

set -e

echo "🖥️  SP CDR Blockchain - Physical Machine Setup (Vodafone-UK)"
echo "============================================================="

# System info
echo "📊 System Information:"
echo "CPU Cores: $(nproc)"
echo "Memory: $(free -h | grep '^Mem:' | awk '{print $2}')"
echo "Disk Space: $(df -h / | tail -1 | awk '{print $4}')"
echo "OS: $(lsb_release -d | cut -f2- 2>/dev/null || cat /etc/os-release | grep PRETTY_NAME | cut -d'"' -f2)"
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
    docker.io \
    docker-compose \
    grafana \
    prometheus \
    python3 \
    python3-pip

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
mkdir -p ~/sp-blockchain/{vodafone-data,logs,monitoring,backups}
cd ~/sp-blockchain

# Vodafone UK config
echo "🇬🇧 Configuring Vodafone UK Validator..."
cat > ~/sp-blockchain/vodafone-config.json << 'EOF'
{
  "network_id": "Vodafone-UK",
  "validator_name": "vodafone-validator-001",
  "api_port": 8080,
  "p2p_port": 9080,
  "data_dir": "/home/$USER/sp-blockchain/vodafone-data",
  "log_level": "info",
  "peers": [
    "T-Mobile-DE@192.168.1.10:9080",
    "Orange-FR@192.168.1.20:9080"
  ],
  "consensus": {
    "validator_key_file": "vodafone_validator.key",
    "bls_key_file": "vodafone_bls.key",
    "network_key_file": "vodafone_network.key"
  },
  "features": {
    "settlement_monitor": true,
    "blockchain_explorer": true,
    "metrics_exporter": true,
    "backup_service": true
  }
}
EOF

# Vodafone startup script
cat > ~/sp-blockchain/start-vodafone.sh << 'EOF'
#!/bin/bash
cd ~/sp-blockchain/sp_cdr_reconciliation_bc

echo "🇬🇧 Starting Vodafone UK Validator + Settlement Monitor..."
echo "========================================================"
echo "API: http://$(hostname -I | awk '{print $1}'):8080"
echo "P2P: $(hostname -I | awk '{print $1}'):9080"
echo "Explorer: http://$(hostname -I | awk '{print $1}'):8080/explorer"
echo "Metrics: http://$(hostname -I | awk '{print $1}'):9090"
echo ""

# Generate keys if they don't exist
if [ ! -f ~/sp-blockchain/vodafone-data/vodafone_validator.key ]; then
    echo "🔑 Generating validator keys..."
    ./target/release/sp-cdr-node generate-keys \
        --output-dir ~/sp-blockchain/vodafone-data \
        --network-id Vodafone-UK
fi

# Start the node
./target/release/sp-cdr-node \
    --config ~/sp-blockchain/vodafone-config.json \
    2>&1 | tee ~/sp-blockchain/logs/vodafone-$(date +%Y%m%d-%H%M%S).log
EOF
chmod +x ~/sp-blockchain/start-vodafone.sh

# Settlement monitoring script
cat > ~/sp-blockchain/settlement-monitor.sh << 'EOF'
#!/bin/bash
echo "💰 SP CDR Settlement Monitor"
echo "============================"

LOG_FILE=~/sp-blockchain/logs/settlements-$(date +%Y%m%d).log

monitor_settlements() {
    while true; do
        TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')

        # Check for pending CDR batches across all validators
        echo "[$TIMESTAMP] Checking pending CDR batches..." | tee -a $LOG_FILE

        TMOBILE_BATCHES=$(curl -s http://192.168.1.10:8080/api/v1/cdr/batches | jq '.pending | length' 2>/dev/null || echo "0")
        ORANGE_BATCHES=$(curl -s http://192.168.1.20:8080/api/v1/cdr/batches | jq '.pending | length' 2>/dev/null || echo "0")
        VODAFONE_BATCHES=$(curl -s http://localhost:8080/api/v1/cdr/batches | jq '.pending | length' 2>/dev/null || echo "0")

        TOTAL_PENDING=$((TMOBILE_BATCHES + ORANGE_BATCHES + VODAFONE_BATCHES))

        echo "  T-Mobile: $TMOBILE_BATCHES, Orange: $ORANGE_BATCHES, Vodafone: $VODAFONE_BATCHES" | tee -a $LOG_FILE
        echo "  Total Pending: $TOTAL_PENDING" | tee -a $LOG_FILE

        # Trigger settlement if enough batches are pending
        if [ $TOTAL_PENDING -ge 3 ]; then
            echo "[$TIMESTAMP] 🚀 Triggering triangular settlement..." | tee -a $LOG_FILE

            SETTLEMENT_RESULT=$(curl -s -X POST http://localhost:8080/api/v1/settlement/process \
                -H "Content-Type: application/json" \
                -d '{
                    "period": "2024-01",
                    "networks": ["T-Mobile-DE", "Vodafone-UK", "Orange-FR"],
                    "settlement_type": "triangular_netting",
                    "auto_execute": true
                }')

            echo "  Settlement Result: $SETTLEMENT_RESULT" | tee -a $LOG_FILE

            # Calculate savings
            GROSS_AMOUNT=$(echo $SETTLEMENT_RESULT | jq '.before_netting.total_amount // 0')
            NET_AMOUNT=$(echo $SETTLEMENT_RESULT | jq '.after_netting.total_amount // 0')

            if [[ "$GROSS_AMOUNT" != "0" && "$NET_AMOUNT" != "0" ]]; then
                SAVINGS=$(echo "scale=1; (($GROSS_AMOUNT - $NET_AMOUNT) * 100) / $GROSS_AMOUNT" | bc 2>/dev/null || echo "N/A")
                echo "  💰 Settlement Savings: $SAVINGS% (€$GROSS_AMOUNT → €$NET_AMOUNT)" | tee -a $LOG_FILE
            fi

            # Notify all validators
            curl -s -X POST http://192.168.1.10:8080/api/v1/notify \
                -d '{"type": "settlement_complete", "settlement_id": "'$(echo $SETTLEMENT_RESULT | jq -r '.settlement_id')'"}' >/dev/null
            curl -s -X POST http://192.168.1.20:8080/api/v1/notify \
                -d '{"type": "settlement_complete", "settlement_id": "'$(echo $SETTLEMENT_RESULT | jq -r '.settlement_id')'"}' >/dev/null
        fi

        sleep 30
    done
}

# Start monitoring in background
monitor_settlements &
MONITOR_PID=$!
echo "Settlement monitor started (PID: $MONITOR_PID)"

# Display live dashboard
while true; do
    clear
    echo "💰 Live Settlement Dashboard - $(date)"
    echo "======================================"
    echo ""

    # Network status
    echo "🌐 Network Status:"
    curl -s --connect-timeout 2 http://192.168.1.10:8080/api/v1/status | jq -r '"  T-Mobile: " + .status + " (Block: " + (.block_number|tostring) + ")"' 2>/dev/null || echo "  T-Mobile: ❌ Offline"
    curl -s --connect-timeout 2 http://192.168.1.20:8080/api/v1/status | jq -r '"  Orange: " + .status + " (Block: " + (.block_number|tostring) + ")"' 2>/dev/null || echo "  Orange: ❌ Offline"
    curl -s --connect-timeout 2 http://localhost:8080/api/v1/status | jq -r '"  Vodafone: " + .status + " (Block: " + (.block_number|tostring) + ")"' 2>/dev/null || echo "  Vodafone: ❌ Offline"

    echo ""
    echo "📊 Settlement Statistics:"
    curl -s http://localhost:8080/api/v1/settlement/stats | jq . 2>/dev/null || echo "  No settlement data available"

    echo ""
    echo "📈 Recent Settlements (last 5):"
    tail -5 $LOG_FILE 2>/dev/null || echo "  No recent settlements"

    echo ""
    echo "Press Ctrl+C to stop monitoring"

    sleep 15
done
EOF
chmod +x ~/sp-blockchain/settlement-monitor.sh

# Blockchain explorer web interface
cat > ~/sp-blockchain/setup-explorer.sh << 'EOF'
#!/bin/bash
echo "🔍 Setting up Blockchain Explorer..."

# Create explorer directory
mkdir -p ~/sp-blockchain/explorer

# Simple web explorer
cat > ~/sp-blockchain/explorer/index.html << 'HTML'
<!DOCTYPE html>
<html>
<head>
    <title>SP CDR Blockchain Explorer</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .header { background: #2c3e50; color: white; padding: 20px; border-radius: 8px; }
        .stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin: 20px 0; }
        .stat-card { background: #f8f9fa; padding: 15px; border-radius: 8px; border-left: 4px solid #3498db; }
        .settlements { background: white; border: 1px solid #ddd; border-radius: 8px; margin: 20px 0; }
        .settlement { padding: 15px; border-bottom: 1px solid #eee; }
        .refresh { background: #3498db; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer; }
    </style>
</head>
<body>
    <div class="header">
        <h1>🔍 SP CDR Blockchain Explorer</h1>
        <p>Real-time monitoring of the SP consortium settlement network</p>
    </div>

    <button class="refresh" onclick="refreshData()">🔄 Refresh Data</button>

    <div class="stats" id="stats">
        <!-- Stats will be loaded here -->
    </div>

    <div class="settlements">
        <h2>Recent Settlements</h2>
        <div id="settlements">
            <!-- Settlements will be loaded here -->
        </div>
    </div>

    <script>
        async function refreshData() {
            try {
                // Load network stats
                const statsResponse = await fetch('/api/v1/stats');
                const stats = await statsResponse.json();

                document.getElementById('stats').innerHTML = `
                    <div class="stat-card">
                        <h3>Total Blocks</h3>
                        <h2>${stats.total_blocks || 0}</h2>
                    </div>
                    <div class="stat-card">
                        <h3>CDR Batches</h3>
                        <h2>${stats.cdr_batches || 0}</h2>
                    </div>
                    <div class="stat-card">
                        <h3>Settlements</h3>
                        <h2>${stats.settlements || 0}</h2>
                    </div>
                    <div class="stat-card">
                        <h3>Total Savings</h3>
                        <h2>€${stats.total_savings || 0}</h2>
                    </div>
                `;

                // Load recent settlements
                const settlementsResponse = await fetch('/api/v1/settlement/recent');
                const settlements = await settlementsResponse.json();

                const settlementsHtml = settlements.map(s => `
                    <div class="settlement">
                        <strong>${s.settlement_id}</strong> - ${s.period}
                        <br>Networks: ${s.networks.join(', ')}
                        <br>Savings: ${s.savings_percentage}% (€${s.gross_amount} → €${s.net_amount})
                        <br><small>${s.timestamp}</small>
                    </div>
                `).join('');

                document.getElementById('settlements').innerHTML = settlementsHtml || '<p>No settlements yet</p>';

            } catch (error) {
                console.error('Failed to refresh data:', error);
            }
        }

        // Auto-refresh every 30 seconds
        setInterval(refreshData, 30000);
        refreshData();
    </script>
</body>
</html>
HTML

# Setup nginx for explorer
sudo tee /etc/nginx/sites-available/sp-explorer << 'NGINX'
server {
    listen 80;
    server_name _;

    # Serve explorer UI
    location / {
        root /home/$USER/sp-blockchain/explorer;
        index index.html;
    }

    # Proxy API calls to blockchain node
    location /api/ {
        proxy_pass http://127.0.0.1:8080/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        add_header Access-Control-Allow-Origin *;
    }
}
NGINX

sudo ln -sf /etc/nginx/sites-available/sp-explorer /etc/nginx/sites-enabled/
sudo systemctl reload nginx

echo "✅ Blockchain Explorer setup complete!"
echo "Access at: http://$(hostname -I | awk '{print $1}')/"
EOF
chmod +x ~/sp-blockchain/setup-explorer.sh

# Network testing script
cat > ~/sp-blockchain/network-test.sh << 'EOF'
#!/bin/bash
echo "🌐 SP CDR Network Connectivity Test"
echo "==================================="

test_connection() {
    local name=$1
    local ip=$2
    local port=$3

    echo -n "Testing $name ($ip:$port)... "
    if curl -s --connect-timeout 5 http://$ip:$port/api/v1/status >/dev/null; then
        echo "✅ OK"
        return 0
    else
        echo "❌ Failed"
        return 1
    fi
}

# Test all validator connections
test_connection "T-Mobile DE" "192.168.1.10" "8080"
test_connection "Orange FR" "192.168.1.20" "8080"
test_connection "Vodafone UK (Local)" "127.0.0.1" "8080"

echo ""
echo "P2P Port Tests:"
test_connection "T-Mobile P2P" "192.168.1.10" "9080"
test_connection "Orange P2P" "192.168.1.20" "9080"

echo ""
echo "🔄 Running end-to-end CDR flow test..."

# Submit test CDR
TEST_CDR='{
    "home_network": "Vodafone-UK",
    "visited_network": "T-Mobile-DE",
    "cdr_batch": {
        "period": "2024-01-test",
        "total_charges": 12500,
        "record_count": 150,
        "batch_id": "test-batch-001"
    },
    "zk_proof": {
        "proof_type": "mock_test_proof",
        "verification_key_id": "test_vk"
    }
}'

echo "Submitting test CDR batch..."
RESULT=$(curl -s -X POST http://127.0.0.1:8080/api/v1/cdr/submit \
    -H "Content-Type: application/json" \
    -d "$TEST_CDR")

echo "Result: $RESULT"

echo ""
echo "✅ Network test complete!"
EOF
chmod +x ~/sp-blockchain/network-test.sh

echo ""
echo "✅ Physical machine (Vodafone-UK) setup complete!"
echo ""
echo "📋 Next steps:"
echo "1. Copy project: scp -r sp_cdr_reconciliation_bc user@$(hostname -I | awk '{print $1}'):~/sp-blockchain/"
echo "2. Build project: cd ~/sp-blockchain/sp_cdr_reconciliation_bc && cargo build --release"
echo "3. Setup explorer: ~/sp-blockchain/setup-explorer.sh"
echo "4. Start Vodafone validator: ~/sp-blockchain/start-vodafone.sh"
echo "5. Monitor settlements: ~/sp-blockchain/settlement-monitor.sh"
echo "6. Test network: ~/sp-blockchain/network-test.sh"
echo ""
echo "🌐 Services will be available at:"
echo "   - Blockchain API: http://$(hostname -I | awk '{print $1}'):8080"
echo "   - Web Explorer: http://$(hostname -I | awk '{print $1}')/"
echo "   - Settlement Monitor: Terminal-based dashboard"