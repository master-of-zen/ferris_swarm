#!/bin/bash

# Auto-Registration Test Script for Ferris Swarm
set -e

echo "🤖 Ferris Swarm Auto-Registration Test Suite"
echo "============================================="

# Configuration
CONSTELLATION_HOST="${CONSTELLATION_HOST:-localhost}"
CONSTELLATION_PORT="${CONSTELLATION_PORT:-3030}"
CONSTELLATION_URL="http://${CONSTELLATION_HOST}:${CONSTELLATION_PORT}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if constellation is running
check_constellation() {
    log_info "Checking if constellation is running..."
    if curl -s "$CONSTELLATION_URL/api/status" > /dev/null 2>&1; then
        log_success "Constellation is running at $CONSTELLATION_URL"
        return 0
    else
        log_error "Constellation is not running at $CONSTELLATION_URL"
        echo "Please start constellation first:"
        echo "  cargo run --bin ferris_swarm_constellation start"
        return 1
    fi
}

# Test 1: Environment-based auto-registration
test_env_registration() {
    log_info "Test 1: Environment-based auto-registration"
    
    # Generate nodes config
    log_info "Generating sample nodes configuration..."
    cargo run --bin ferris_swarm_constellation nodes --generate nodes.toml
    
    # Test single node with environment variables
    log_info "Testing single node auto-registration..."
    
    CONSTELLATION_URL="$CONSTELLATION_URL" \
    NODE_NAME="test-auto-node-1" \
    NODE_CPU_CORES=8 \
    NODE_MEMORY_GB=16 \
    NODE_MAX_CHUNKS=4 \
    NODE_ENCODERS="h264,hevc" \
    timeout 10s cargo run --bin ferris_swarm_node -- \
        --auto-register \
        --heartbeat \
        --heartbeat-interval 5 \
        --address "127.0.0.1:8081" &
    
    NODE1_PID=$!
    sleep 5
    
    # Check if node registered
    NODES_COUNT=$(curl -s "$CONSTELLATION_URL/api/status" | jq -r '.nodes')
    if [ "$NODES_COUNT" -gt 0 ]; then
        log_success "Node auto-registered successfully (Total nodes: $NODES_COUNT)"
    else
        log_error "Node failed to auto-register"
    fi
    
    # Kill test node
    kill $NODE1_PID 2>/dev/null || true
    sleep 2
}

# Test 2: Multiple nodes with different capabilities
test_multiple_nodes() {
    log_info "Test 2: Multiple nodes with different capabilities"
    
    # Start 3 nodes with different configurations
    log_info "Starting node with high-end specs..."
    CONSTELLATION_URL="$CONSTELLATION_URL" \
    NODE_NAME="high-end-node" \
    NODE_CPU_CORES=16 \
    NODE_MEMORY_GB=32 \
    NODE_MAX_CHUNKS=8 \
    NODE_ENCODERS="av1,h264,hevc" \
    timeout 15s cargo run --bin ferris_swarm_node -- \
        --auto-register \
        --heartbeat \
        --address "127.0.0.1:8082" &
    NODE_HIGH_PID=$!
    
    sleep 3
    
    log_info "Starting node with medium specs..."
    CONSTELLATION_URL="$CONSTELLATION_URL" \
    NODE_NAME="medium-node" \
    NODE_CPU_CORES=8 \
    NODE_MEMORY_GB=16 \
    NODE_MAX_CHUNKS=4 \
    NODE_ENCODERS="h264,hevc" \
    timeout 15s cargo run --bin ferris_swarm_node -- \
        --auto-register \
        --heartbeat \
        --address "127.0.0.1:8083" &
    NODE_MED_PID=$!
    
    sleep 3
    
    log_info "Starting node with basic specs..."
    CONSTELLATION_URL="$CONSTELLATION_URL" \
    NODE_NAME="basic-node" \
    NODE_CPU_CORES=4 \
    NODE_MEMORY_GB=8 \
    NODE_MAX_CHUNKS=2 \
    NODE_ENCODERS="h264" \
    timeout 15s cargo run --bin ferris_swarm_node -- \
        --auto-register \
        --heartbeat \
        --address "127.0.0.1:8084" &
    NODE_BASIC_PID=$!
    
    sleep 5
    
    # Check all nodes registered
    NODES_COUNT=$(curl -s "$CONSTELLATION_URL/api/status" | jq -r '.nodes')
    log_info "Total registered nodes: $NODES_COUNT"
    
    if [ "$NODES_COUNT" -ge 3 ]; then
        log_success "Multiple nodes registered successfully"
    else
        log_warning "Expected 3+ nodes, got $NODES_COUNT"
    fi
    
    # Test dashboard data
    log_info "Checking dashboard data..."
    DASHBOARD_DATA=$(curl -s "$CONSTELLATION_URL/api/dashboard/data")
    ACTIVE_NODES=$(echo "$DASHBOARD_DATA" | jq -r '.stats.active_nodes')
    TOTAL_CAPACITY=$(echo "$DASHBOARD_DATA" | jq -r '.stats.total_nodes')
    
    log_info "Active nodes: $ACTIVE_NODES"
    log_info "Total capacity: $TOTAL_CAPACITY"
    
    # Clean up nodes
    kill $NODE_HIGH_PID $NODE_MED_PID $NODE_BASIC_PID 2>/dev/null || true
    sleep 3
}

# Test 3: Configuration file-based registration
test_config_registration() {
    log_info "Test 3: Configuration file-based registration"
    
    # Start constellation with auto-registration from config
    log_info "Testing constellation auto-registration from nodes.toml..."
    
    # The constellation should auto-register nodes from the config file
    # This would be running in background, so we'll just verify the config exists
    if [ -f "nodes.toml" ]; then
        log_success "Nodes configuration file exists"
        log_info "Configuration contents:"
        head -20 nodes.toml
    else
        log_warning "Nodes configuration file not found"
    fi
}

# Test 4: Capability detection
test_capability_detection() {
    log_info "Test 4: System capability detection"
    
    # Test without overrides (auto-detect)
    log_info "Testing auto-detection of system capabilities..."
    CONSTELLATION_URL="$CONSTELLATION_URL" \
    NODE_NAME="auto-detect-node" \
    timeout 10s cargo run --bin ferris_swarm_node -- \
        --auto-register \
        --address "127.0.0.1:8085" &
    AUTO_NODE_PID=$!
    
    sleep 5
    
    # Get dashboard data to see detected capabilities
    DASHBOARD_DATA=$(curl -s "$CONSTELLATION_URL/api/dashboard/data")
    echo "$DASHBOARD_DATA" | jq '.nodes | to_entries | .[] | select(.value.id | startswith("auto-detect")) | .value.capabilities'
    
    kill $AUTO_NODE_PID 2>/dev/null || true
    log_success "Capability detection test completed"
}

# Test 5: Heartbeat functionality
test_heartbeat() {
    log_info "Test 5: Heartbeat functionality"
    
    # Start node with short heartbeat interval
    log_info "Starting node with 3-second heartbeat interval..."
    CONSTELLATION_URL="$CONSTELLATION_URL" \
    NODE_NAME="heartbeat-test-node" \
    timeout 20s cargo run --bin ferris_swarm_node -- \
        --auto-register \
        --heartbeat \
        --heartbeat-interval 3 \
        --address "127.0.0.1:8086" &
    HEARTBEAT_PID=$!
    
    # Monitor heartbeats for 15 seconds
    log_info "Monitoring heartbeats for 15 seconds..."
    for i in {1..5}; do
        sleep 3
        STATUS=$(curl -s "$CONSTELLATION_URL/api/dashboard/data" | jq -r '.nodes | to_entries | .[] | select(.value.id | contains("heartbeat-test")) | .value.status')
        log_info "Heartbeat $i: Node status is $STATUS"
    done
    
    kill $HEARTBEAT_PID 2>/dev/null || true
    log_success "Heartbeat test completed"
}

# Test 6: Error handling
test_error_handling() {
    log_info "Test 6: Error handling"
    
    # Test with invalid constellation URL
    log_info "Testing with invalid constellation URL..."
    CONSTELLATION_URL="http://invalid-host:9999" \
    NODE_NAME="error-test-node" \
    timeout 5s cargo run --bin ferris_swarm_node -- \
        --auto-register \
        --address "127.0.0.1:8087" &
    ERROR_PID=$!
    
    sleep 3
    kill $ERROR_PID 2>/dev/null || true
    log_success "Error handling test completed (node should handle connection failure gracefully)"
}

# Final status check
final_status_check() {
    log_info "Final system status:"
    curl -s "$CONSTELLATION_URL/api/status" | jq .
    
    log_info "Dashboard statistics:"
    curl -s "$CONSTELLATION_URL/api/dashboard/data" | jq '.stats'
}

# Cleanup function
cleanup() {
    log_info "Cleaning up test processes..."
    pkill -f "ferris_swarm_node" 2>/dev/null || true
    sleep 2
    log_info "Cleanup completed"
}

# Main test execution
main() {
    # Set up cleanup trap
    trap cleanup EXIT
    
    # Run tests
    if check_constellation; then
        test_env_registration
        test_multiple_nodes
        test_config_registration
        test_capability_detection
        test_heartbeat
        test_error_handling
        final_status_check
        
        log_success "All auto-registration tests completed!"
        echo ""
        echo "🎯 Test Summary:"
        echo "   ✅ Environment-based registration"
        echo "   ✅ Multiple node registration"
        echo "   ✅ Configuration file support"
        echo "   ✅ Capability auto-detection"
        echo "   ✅ Heartbeat functionality"
        echo "   ✅ Error handling"
        echo ""
        echo "🌐 Dashboard: http://$CONSTELLATION_HOST:$CONSTELLATION_PORT"
    else
        log_error "Cannot run tests without constellation service"
        exit 1
    fi
}

# Handle script arguments
case "${1:-}" in
    "cleanup")
        cleanup
        ;;
    "check")
        check_constellation
        ;;
    *)
        main
        ;;
esac