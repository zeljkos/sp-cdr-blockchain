#!/bin/bash

echo "🔐 SP CDR Blockchain - Multi-Party Trusted Setup Ceremony"
echo "========================================================="
echo ""
echo "This simulates a real-world trusted setup ceremony between"
echo "telecom operators using separate Docker containers."
echo ""

# Create persistent data directories
echo "📁 Creating persistent data directories..."
mkdir -p ../persistent_data/{ceremony_coordinator,participant_tmobile,participant_vodafone,participant_orange,ceremony_verifier}
mkdir -p ../persistent_data/{shared_ceremony,shared_zkp_keys}
mkdir -p ../persistent_data/{validator-1,validator-2,validator-3}

# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

# Clean up any existing ceremony data
echo "🧹 Cleaning up any previous ceremony data..."
rm -rf ../persistent_data/shared_ceremony/*
rm -rf ../persistent_data/shared_zkp_keys/*
rm -f ../persistent_data/ceremony_coordinator/*
rm -f ../persistent_data/participant_*/*
rm -f ../persistent_data/ceremony_verifier/*

echo ""
echo "🚀 Starting Multi-Party Trusted Setup Ceremony..."
echo ""
echo "Ceremony Flow:"
echo "  1. 🎭 Coordinator initializes ceremony parameters"
echo "  2. 🇩🇪 T-Mobile Deutschland contributes randomness"
echo "  3. 🇬🇧 Vodafone UK contributes randomness"
echo "  4. 🇫🇷 Orange France contributes randomness"
echo "  5. 🔍 Independent verifier audits ceremony"
echo "  6. 🚀 Blockchain validators start using ceremony keys"
echo ""

# Start the ceremony
docker compose -f docker-compose.trusted-setup-persistent.yml up --build

echo ""
echo "📊 Ceremony Progress Monitoring:"
echo "  Coordinator:     http://localhost:9000"
echo "  T-Mobile:        http://localhost:9010"
echo "  Vodafone:        http://localhost:9020"
echo "  Orange:          http://localhost:9030"
echo "  Verifier:        http://localhost:9100"
echo ""
echo "🌐 Blockchain Access (after ceremony):"
echo "  Validator 1:     http://localhost:8081"
echo "  Validator 2:     http://localhost:8091"
echo "  Validator 3:     http://localhost:8101"
echo ""
echo "📊 Monitor with: docker compose -f docker-compose.trusted-setup-persistent.yml logs -f"
echo "🛑 Stop with: docker compose -f docker-compose.trusted-setup-persistent.yml down"