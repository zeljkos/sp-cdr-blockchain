#!/bin/bash
# Mac M4 Pro Setup Script for SP CDR PoC

set -e

echo "🍎 SP CDR Blockchain - Mac M4 Pro Setup"
echo "======================================"

# Check if running on Mac
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "❌ This script is for macOS only"
    exit 1
fi

# Install Homebrew if not present
if ! command -v brew &> /dev/null; then
    echo "🍺 Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
fi

# Install dependencies
echo "📦 Installing dependencies..."
brew install \
    rust \
    git \
    htop \
    jq \
    curl \
    nginx

# Build the project
echo "🏗️  Building SP CDR Blockchain..."
cargo build --release

# Create deployment directories
echo "📁 Creating deployment directories..."
mkdir -p ~/sp-blockchain/{tmobile,orange,dev-tools}

# Setup T-Mobile DE Validator
echo "🇩🇪 Setting up T-Mobile Germany Validator..."
cp target/release/sp-cdr-node ~/sp-blockchain/tmobile/
mkdir -p ~/sp-blockchain/tmobile/{data,logs,keys}

# Generate T-Mobile validator keys
cat > ~/sp-blockchain/tmobile/config.json << 'EOF'
{
  "network_id": "T-Mobile-DE",
  "validator_address": "tmobile-validator-001",
  "api_port": 8080,
  "p2p_port": 9080,
  "data_dir": "./data",
  "peers": [
    "vodafone-uk@PHYSICAL_IP:9081",
    "orange-fr@127.0.0.1:9082"
  ],
  "consensus": {
    "validator_key": "tmobile_validator.key",
    "bls_key": "tmobile_bls.key"
  }
}
EOF

# Setup Orange FR Validator
echo "🇫🇷 Setting up Orange France Validator..."
cp target/release/sp-cdr-node ~/sp-blockchain/orange/
mkdir -p ~/sp-blockchain/orange/{data,logs,keys}

cat > ~/sp-blockchain/orange/config.json << 'EOF'
{
  "network_id": "Orange-FR",
  "validator_address": "orange-validator-001",
  "api_port": 8082,
  "p2p_port": 9082,
  "data_dir": "./data",
  "peers": [
    "tmobile-de@127.0.0.1:9080",
    "vodafone-uk@PHYSICAL_IP:9081"
  ],
  "consensus": {
    "validator_key": "orange_validator.key",
    "bls_key": "orange_bls.key"
  }
}
EOF

# Create launch scripts
echo "🚀 Creating launch scripts..."

# T-Mobile launcher
cat > ~/sp-blockchain/tmobile/start.sh << 'EOF'
#!/bin/bash
cd "$(dirname "$0")"
echo "🇩🇪 Starting T-Mobile Germany Validator..."
./sp-cdr-node \
    --config config.json \
    --network-id "T-Mobile-DE" \
    --data-dir ./data \
    --api-port 8080 \
    --p2p-port 9080 \
    --log-level info \
    2>&1 | tee logs/tmobile-$(date +%Y%m%d-%H%M%S).log
EOF
chmod +x ~/sp-blockchain/tmobile/start.sh

# Orange launcher
cat > ~/sp-blockchain/orange/start.sh << 'EOF'
#!/bin/bash
cd "$(dirname "$0")"
echo "🇫🇷 Starting Orange France Validator..."
./sp-cdr-node \
    --config config.json \
    --network-id "Orange-FR" \
    --data-dir ./data \
    --api-port 8082 \
    --p2p-port 9082 \
    --log-level info \
    2>&1 | tee logs/orange-$(date +%Y%m%d-%H%M%S).log
EOF
chmod +x ~/sp-blockchain/orange/start.sh

echo "✅ Mac setup complete!"
echo ""
echo "📋 Next steps:"
echo "1. Update PHYSICAL_IP in config files with your physical machine IP"
echo "2. Copy project to physical machine: scp -r . user@PHYSICAL_IP:~/sp_cdr_reconciliation_bc"
echo "3. Start T-Mobile: ~/sp-blockchain/tmobile/start.sh"
echo "4. Start Orange: ~/sp-blockchain/orange/start.sh"