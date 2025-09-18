#!/bin/bash

# SP CDR Blockchain - Optimized Persistent Storage Startup
# Builds once on host, runs with persistent MDBX storage

set -e

echo "🚀 SP CDR Blockchain - Optimized Persistent Setup"
echo "================================================="
echo ""

# Check if we're in the docker directory
if [ ! -f "docker-compose.persistent.yml" ]; then
    echo "❌ Please run this script from the docker/ directory"
    echo "cd docker && ./start-persistent.sh"
    exit 1
fi

echo "🛠️  Step 1: Building on host (faster, less space)"
echo "------------------------------------------------"
cd ..

# Build release version on host
echo "⚡ Building optimized release binary on host..."
cargo build --release

# Verify binaries exist
if [ ! -f "target/release/sp-cdr-node" ]; then
    echo "❌ Build failed - sp-cdr-node binary not found"
    exit 1
fi

echo "✅ Host build complete!"
echo "   📦 Binary size: $(du -h target/release/sp-cdr-node | cut -f1)"

cd docker

echo ""
echo "💾 Step 2: Setting up persistent storage"
echo "----------------------------------------"

# Create persistent data directories on host
echo "📁 Creating persistent data directories..."
mkdir -p ../persistent_data/validator-1/{blockchain,zkp_keys}
mkdir -p ../persistent_data/validator-2/{blockchain,zkp_keys}
mkdir -p ../persistent_data/validator-3/{blockchain,zkp_keys}
mkdir -p ../persistent_data/shared_zkp_keys

echo "✅ Persistent directories created at:"
echo "   📁 $(pwd)/../persistent_data/"
echo "   📊 Total size: $(du -sh ../persistent_data/ 2>/dev/null | cut -f1 || echo '0B')"

echo ""
echo "🔐 Step 2.5: Checking ZK Trusted Setup Keys"
echo "---------------------------------------------"

# Check if ZK keys exist
ZK_KEYS_DIR="../persistent_data/shared_zkp_keys"
REQUIRED_FILES=("settlement_calculation.pk" "settlement_calculation.vk" "cdr_privacy.pk" "cdr_privacy.vk")

keys_exist=true
for file in "${REQUIRED_FILES[@]}"; do
    if [[ ! -f "$ZK_KEYS_DIR/$file" ]]; then
        keys_exist=false
        break
    fi
done

if [[ "$keys_exist" == true ]]; then
    echo "✅ ZK keys found - using existing trusted setup"
    echo "   📁 Keys location: $ZK_KEYS_DIR"
    echo "   🔑 Files found:"
    for file in "${REQUIRED_FILES[@]}"; do
        size=$(stat -c%s "$ZK_KEYS_DIR/$file" 2>/dev/null || echo "0")
        echo "      • $file ($size bytes)"
    done
else
    echo "⚠️  ZK keys missing - running trusted setup ceremony..."
    echo "   📍 This will generate production-grade cryptographic keys"
    echo "   ⏱️  This may take a few seconds..."

    # Change to project root to run trusted setup
    cd ..

    # Run trusted setup ceremony
    if [[ -f "./target/release/trusted-setup-demo" ]]; then
        echo "   🔐 Running trusted setup ceremony..."
        ./target/release/trusted-setup-demo

        # Copy generated keys to persistent storage
        if [[ -d "./test_ceremony_keys" ]]; then
            echo "   📋 Copying keys to persistent storage..."
            cp -r test_ceremony_keys/* persistent_data/shared_zkp_keys/
            echo "   ✅ ZK keys generated and installed successfully"

            # Verify keys were copied
            echo "   🔍 Verification:"
            for file in "${REQUIRED_FILES[@]}"; do
                if [[ -f "persistent_data/shared_zkp_keys/$file" ]]; then
                    size=$(stat -c%s "persistent_data/shared_zkp_keys/$file" 2>/dev/null || echo "0")
                    echo "      ✅ $file ($size bytes)"
                else
                    echo "      ❌ $file - MISSING!"
                fi
            done
        else
            echo "   ❌ ERROR: Trusted setup failed - no keys generated"
            echo "   💡 Try running manually: ./target/release/trusted-setup-demo"
            exit 1
        fi
    else
        echo "   ❌ ERROR: trusted-setup-demo binary not found"
        echo "   💡 Run: cargo build --release"
        exit 1
    fi

    # Return to docker directory
    cd docker
fi

echo ""
echo "🛑 Step 3: Stopping existing containers"
echo "---------------------------------------"
docker-compose -f docker-compose.persistent.yml down --timeout 30 2>/dev/null || true

echo ""
echo "🐳 Step 4: Building lightweight containers (using pre-built binaries)"
echo "---------------------------------------------------------------------"
docker-compose -f docker-compose.persistent.yml build --no-cache

echo ""
echo "🌐 Step 5: Starting persistent blockchain network"
echo "-------------------------------------------------"
docker-compose -f docker-compose.persistent.yml up -d

echo ""
echo "⏳ Waiting for network initialization..."
sleep 10

echo ""
echo "✅ SP CDR Blockchain Network Started with Persistent Storage!"
echo "============================================================"
echo ""
echo "🔍 Network Status:"
docker-compose -f docker-compose.persistent.yml ps

echo ""
echo "📊 Persistent Data Location:"
echo "   📁 Host directory: $(pwd)/../persistent_data/"
echo "   💾 MDBX databases: validator-*/blockchain/"
echo "   🔐 ZK keys: validator-*/zkp_keys/ + shared_zkp_keys/"
echo ""
echo "🔧 Useful Commands:"
echo "   📊 Monitor logs:    docker-compose -f docker-compose.persistent.yml logs -f"
echo "   🔍 Inspect blocks:  docker exec sp-validator-1 ./target/release/sp-cdr-node inspect --target blocks"
echo "   📈 View stats:      docker exec sp-validator-1 ./target/release/sp-cdr-node inspect --target stats"
echo "   🛑 Stop network:    docker-compose -f docker-compose.persistent.yml down"
echo "   🧹 Clean all:       ./clean.sh"
echo ""
echo "💡 Key Improvements:"
echo "   ✅ Persistent MDBX storage (survives container restarts)"
echo "   ✅ Pre-built binaries (faster, smaller images)"
echo "   ✅ Lower settlement thresholds (€1 vs €100)"
echo "   ✅ Host bind mounts (direct access to blockchain data)"
echo "   ✅ Shared ZK keys between containers"
echo ""
echo "🎉 Ready to process CDR settlements with persistent blockchain!"