#!/bin/bash

echo "📊 SP CDR Blockchain Monitor"
echo "============================"

while true; do
    clear
    echo "📊 SP CDR Blockchain Monitor - $(date)"
    echo "============================"
    echo ""

    echo "🐳 Container Status:"
    echo "-------------------"
    docker compose ps
    echo ""

    echo "💾 Resource Usage:"
    echo "-----------------"
    docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}"
    echo ""

    echo "🌐 Network Health:"
    echo "-----------------"
    echo -n "Validator 1: "
    curl -s -o /dev/null -w "%{http_code}" http://localhost:8081/health 2>/dev/null || echo "❌"

    echo -n "Validator 2: "
    curl -s -o /dev/null -w "%{http_code}" http://localhost:8091/health 2>/dev/null || echo "❌"

    echo -n "Validator 3: "
    curl -s -o /dev/null -w "%{http_code}" http://localhost:8101/health 2>/dev/null || echo "❌"
    echo ""

    echo "💾 Data Directory Sizes:"
    echo "-----------------------"
    du -sh data/* 2>/dev/null || echo "No data yet"
    echo ""

    echo "🔄 Refreshing in 10 seconds... (Ctrl+C to exit)"
    sleep 10
done