#!/bin/bash

echo "🛑 Stopping SP CDR Blockchain Demo"
echo "=================================="

# Stop all containers
docker compose down

echo "✅ All validators stopped."
echo "📊 View logs: docker compose logs"
echo "🔄 Restart: ./start.sh"