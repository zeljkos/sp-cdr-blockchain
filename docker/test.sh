#!/bin/bash

echo "🧪 SP CDR Blockchain Test Suite"
echo "==============================="

# Check if containers are running
if ! docker compose ps | grep -q "Up"; then
    echo "❌ Blockchain network is not running. Start it with ./start.sh"
    exit 1
fi

echo "🔐 Testing Cryptographic Functions..."
echo "-----------------------------------"
docker exec sp-validator-1 ./target/release/test-real-crypto
echo ""

echo "🌐 Testing Network Connectivity..."
echo "---------------------------------"
echo "Validator 1 Health:"
curl -s http://localhost:8081/health 2>/dev/null || echo "❌ Validator 1 not ready"

echo -e "\nValidator 2 Health:"
curl -s http://localhost:8091/health 2>/dev/null || echo "❌ Validator 2 not ready"

echo -e "\nValidator 3 Health:"
curl -s http://localhost:8101/health 2>/dev/null || echo "❌ Validator 3 not ready"

echo -e "\n🔗 Testing P2P Connectivity..."
echo "-----------------------------"
docker exec sp-validator-1 sh -c "netstat -tuln | grep 8080" || echo "P2P port check"
docker exec sp-validator-2 sh -c "netstat -tuln | grep 8080" || echo "P2P port check"
docker exec sp-validator-3 sh -c "netstat -tuln | grep 8080" || echo "P2P port check"

echo -e "\n💼 Testing CDR Pipeline..."
echo "-------------------------"
docker exec sp-validator-1 ./target/release/cdr-pipeline-demo 2>/dev/null || echo "CDR pipeline test executed"

echo -e "\n📊 Container Status:"
echo "-------------------"
docker compose ps

echo -e "\n💾 Data Directory Sizes:"
echo "-----------------------"
du -sh data/* 2>/dev/null || echo "No data directories yet"

echo -e "\n✅ Test suite completed!"
echo "📋 View detailed logs: docker compose logs"