#!/bin/bash

# Test script for mDNS discovery functionality
# This script demonstrates the automatic constellation discovery

echo "=== Ferris Swarm mDNS Discovery Test ==="
echo

# Function to check if a process is running
check_process() {
    pgrep -f "$1" > /dev/null
}

# Function to wait for process to be ready
wait_for_ready() {
    local url="$1"
    local timeout=10
    local count=0
    
    echo "Waiting for service at $url to be ready..."
    while [ $count -lt $timeout ]; do
        if curl -s "$url" > /dev/null 2>&1; then
            echo "✓ Service is ready"
            return 0
        fi
        sleep 1
        count=$((count + 1))
    done
    echo "✗ Service failed to start within $timeout seconds"
    return 1
}

echo "Step 1: Building Ferris Swarm..."
cargo build --release --quiet || {
    echo "✗ Build failed"
    exit 1
}
echo "✓ Build completed"
echo

echo "Step 2: Starting constellation with mDNS advertisement..."
./target/release/ferris_swarm_constellation start --bind 127.0.0.1:3030 &
CONSTELLATION_PID=$!

# Wait for constellation to start
if wait_for_ready "http://127.0.0.1:3030/api/health"; then
    echo "✓ Constellation started successfully"
else
    echo "✗ Constellation failed to start"
    kill $CONSTELLATION_PID 2>/dev/null
    exit 1
fi
echo

echo "Step 3: Testing node auto-discovery..."
echo "Starting node with auto-registration (should discover constellation automatically)..."

# Start node without specifying constellation URL - it should discover it
timeout 30 ./target/release/ferris_swarm_node \
    --auto-register \
    --heartbeat \
    --address 127.0.0.1:50051 &
NODE_PID=$!

# Give the node some time to register
sleep 5

echo

echo "Step 4: Checking registration status..."
# Check if node registered successfully
RESPONSE=$(curl -s http://127.0.0.1:3030/api/status)
NODE_COUNT=$(echo "$RESPONSE" | grep -o '"nodes":[0-9]*' | cut -d: -f2)

if [ "$NODE_COUNT" -gt 0 ]; then
    echo "✓ Node successfully auto-registered via discovery!"
    echo "✓ Found $NODE_COUNT node(s) in constellation"
else
    echo "✗ Node registration failed"
fi

echo
echo "Step 5: Testing dashboard accessibility..."
echo "Dashboard URL: http://127.0.0.1:3030"
echo "API Status URL: http://127.0.0.1:3030/api/status"
echo "Health Check URL: http://127.0.0.1:3030/api/health"

# Test API endpoints
if curl -s http://127.0.0.1:3030/api/health | grep -q "healthy"; then
    echo "✓ Health check endpoint working"
else
    echo "✗ Health check endpoint failed"
fi

if curl -s http://127.0.0.1:3030/api/status | grep -q "constellation"; then
    echo "✓ Status endpoint working"
else
    echo "✗ Status endpoint failed"
fi

echo
echo "Step 6: Discovery workflow summary..."
echo "1. Constellation advertises itself on the network"
echo "2. Node discovers constellation automatically (no manual URL needed)"
echo "3. Node registers and starts heartbeat service"
echo "4. Dashboard shows registered nodes"

echo
echo "=== Discovery Test Complete ==="
echo
echo "Cleanup: Stopping services..."

# Cleanup
kill $NODE_PID 2>/dev/null
kill $CONSTELLATION_PID 2>/dev/null

wait $NODE_PID 2>/dev/null
wait $CONSTELLATION_PID 2>/dev/null

echo "✓ Services stopped"
echo
echo "To manually test discovery:"
echo "1. Start constellation: ./target/release/ferris_swarm_constellation start"
echo "2. Start node: ./target/release/ferris_swarm_node --auto-register --heartbeat"
echo "3. Visit dashboard: http://localhost:3030"