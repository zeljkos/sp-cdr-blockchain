#!/bin/bash

echo "🔐 SP CDR Blockchain - 3 Validator Demo"
echo "======================================"

# Create data directories
echo "📁 Creating data directories..."
mkdir -p data/{validator-1,validator-2,validator-3}

# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

# Build and start the blockchain network
echo "🚀 Building and starting SP CDR blockchain network..."
docker compose up --build

echo "✅ SP CDR Blockchain network started!"
echo ""
echo "🌐 Access Points:"
echo "  Validator 1: http://localhost:8081"
echo "  Validator 2: http://localhost:8091"
echo "  Validator 3: http://localhost:8101"
echo ""
echo "📊 Monitor with: docker compose logs -f"
echo "🛑 Stop with: docker compose down"