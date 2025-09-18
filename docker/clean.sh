#!/bin/bash

# SP CDR Blockchain - Complete System Cleanup Script
# This script removes ALL blockchain data, Docker containers, images, and keys
# WARNING: This is irreversible! Use with caution.

set -e

echo "🧹 SP CDR Blockchain - Complete System Cleanup"
echo "=============================================="
echo ""
echo "⚠️  WARNING: This will delete ALL blockchain data!"
echo "   • Docker containers and images"
echo "   • All validator data directories"
echo "   • ZK trusted setup keys"
echo "   • Blockchain state and transactions"
echo "   • Network configurations"
echo ""

# Ask for confirmation
read -p "Are you sure you want to continue? (yes/no): " confirm
if [[ $confirm != "yes" ]]; then
    echo "❌ Cleanup cancelled."
    exit 0
fi

echo ""
echo "🛑 Stopping all containers..."
docker-compose down --timeout 30 2>/dev/null || true

echo "🗑️  Removing containers..."
docker container rm -f sp-validator-1 sp-validator-2 sp-validator-3 2>/dev/null || true

echo "🗑️  Removing Docker images..."
docker image rm -f sp_cdr_reconciliation_bc-validator-1 2>/dev/null || true
docker image rm -f sp_cdr_reconciliation_bc-validator-2 2>/dev/null || true
docker image rm -f sp_cdr_reconciliation_bc-validator-3 2>/dev/null || true
docker image rm -f sp-cdr-reconciliation-bc-validator-1 2>/dev/null || true
docker image rm -f sp-cdr-reconciliation-bc-validator-2 2>/dev/null || true
docker image rm -f sp-cdr-reconciliation-bc-validator-3 2>/dev/null || true

echo "🗑️  Removing Docker networks..."
docker network rm sp_blockchain_net 2>/dev/null || true
docker network rm sp_cdr_reconciliation_bc_sp_blockchain_net 2>/dev/null || true

echo "🗑️  Removing Docker volumes..."
docker volume rm sp_cdr_reconciliation_bc_validator-1-data 2>/dev/null || true
docker volume rm sp_cdr_reconciliation_bc_validator-2-data 2>/dev/null || true
docker volume rm sp_cdr_reconciliation_bc_validator-3-data 2>/dev/null || true

echo "🗑️  Removing data directories..."
rm -rf ./data/validator-1
rm -rf ./data/validator-2
rm -rf ./data/validator-3
rm -rf ./data

echo "🗑️  Removing persistent data and ZK keys..."
rm -rf ../persistent_data/validator-1
rm -rf ../persistent_data/validator-2
rm -rf ../persistent_data/validator-3
rm -rf ../persistent_data/shared_zkp_keys
rm -rf ../persistent_data
rm -rf ../test_ceremony_keys

echo "🗑️  Removing build cache..."
docker builder prune -f 2>/dev/null || true

echo "🗑️  Removing dangling images..."
docker image prune -f 2>/dev/null || true

echo "🗑️  Removing unused networks..."
docker network prune -f 2>/dev/null || true

echo "🗑️  Removing unused volumes..."
docker volume prune -f 2>/dev/null || true

# Optional: Clean Docker system (commented out for safety)
# echo "🗑️  Deep cleaning Docker system..."
# docker system prune -a -f --volumes

echo ""
echo "✅ Complete cleanup finished!"
echo ""
echo "📊 Cleanup Summary:"
echo "   🗑️  All SP CDR containers removed"
echo "   🗑️  All blockchain data deleted"
echo "   🗑️  All ZK keys removed"
echo "   🗑️  Docker images and networks cleaned"
echo ""
echo "🚀 System is completely clean. To start from scratch:"
echo "   1. Run: ./target/release/trusted-setup-demo"
echo "   2. Run: cp -r test_ceremony_keys/* persistent_data/shared_zkp_keys/"
echo "   3. Run: ./start-persistent.sh"
echo ""
echo "💡 Or use the quick start guide: QUICK_START.md"