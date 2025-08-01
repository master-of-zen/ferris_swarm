#!/bin/bash

# Test mDNS Discovery for Ferris Swarm
echo "🦀 Testing Ferris Swarm mDNS Discovery"
echo "===================================="

# Function to cleanup background processes
cleanup() {
    echo "🧹 Cleaning up background processes..."
    if [ ! -z "$CONSTELLATION_PID" ]; then
        kill $CONSTELLATION_PID 2>/dev/null
    fi
    if [ ! -z "$NODE1_PID" ]; then
        kill $NODE1_PID 2>/dev/null
    fi
    if [ ! -z "$NODE2_PID" ]; then
        kill $NODE2_PID 2>/dev/null
    fi
    exit 0
}

# Set trap to cleanup on exit
trap cleanup EXIT INT TERM

echo "📦 Building project..."
cargo build --release || {
    echo "❌ Build failed"
    exit 1
}

echo "🌟 Starting constellation with mDNS advertisement..."
RUST_LOG=info cargo run --release --bin ferris_swarm_constellation start \
    --bind 0.0.0.0:3030 \
    --verbose &
CONSTELLATION_PID=$!

echo "⏳ Waiting for constellation to start..."
sleep 5

echo "🖥️  Starting node 1 with auto-registration and mDNS discovery..."
RUST_LOG=info cargo run --release --bin node -- \
    --address 0.0.0.0:8080 \
    --temp-dir ./test_node1_temp \
    --cpu-cores 4 \
    --memory-gb 8 \
    --max-chunks 2 &
NODE1_PID=$!

echo "🖥️  Starting node 2 with auto-registration and mDNS discovery..."
RUST_LOG=info cargo run --release --bin node -- \
    --address 0.0.0.0:8081 \
    --temp-dir ./test_node2_temp \
    --cpu-cores 2 \
    --memory-gb 4 \
    --max-chunks 1 &
NODE2_PID=$!

echo "⏳ Waiting for nodes to register..."
sleep 10

echo "📊 Checking constellation status..."
curl -s http://localhost:3030/api/status | jq . || echo "Constellation not responding"

echo "🔍 Checking registered nodes..."
curl -s http://localhost:3030/api/nodes | jq . || echo "No nodes API response"

echo "📈 Checking dashboard data..."
curl -s http://localhost:3030/api/dashboard/data | jq '.stats' || echo "No dashboard data"

echo ""
echo "✅ mDNS test completed!"
echo "🌐 Dashboard available at: http://localhost:3030"
echo "🔗 WebSocket endpoint: ws://localhost:3030/ws"
echo ""
echo "Press Ctrl+C to stop all services..."

# Wait for user interrupt
wait