#!/bin/bash

# Simple deployment script for Ferris Swarm nodes
# Usage: ./deploy_nodes.sh <constellation_host> [node_count]

set -e

CONSTELLATION_HOST="${1:-localhost}"
NODE_COUNT="${2:-3}"
BASE_PORT=8080

echo "🚀 Deploying $NODE_COUNT Ferris Swarm nodes"
echo "   Constellation: $CONSTELLATION_HOST:3030"
echo "   Starting from port: $BASE_PORT"
echo ""

# Create nodes directory
mkdir -p nodes
cd nodes

# Generate different node configurations
for i in $(seq 1 $NODE_COUNT); do
    PORT=$((BASE_PORT + i))
    NODE_NAME="node-$i"
    
    echo "📦 Setting up $NODE_NAME on port $PORT"
    
    # Create node-specific directory
    mkdir -p "$NODE_NAME"
    
    # Create startup script for this node
    cat > "$NODE_NAME/start.sh" << EOF
#!/bin/bash

# Node $i startup script
export CONSTELLATION_URL="http://$CONSTELLATION_HOST:3030"
export NODE_NAME="$NODE_NAME"
export NODE_CPU_CORES=$((4 * i))
export NODE_MEMORY_GB=$((8 * i))
export NODE_MAX_CHUNKS=$((2 * i))

case $i in
    1)
        export NODE_ENCODERS="h264"
        ;;
    2)
        export NODE_ENCODERS="h264,hevc"
        ;;
    *)
        export NODE_ENCODERS="av1,h264,hevc"
        ;;
esac

echo "🟢 Starting $NODE_NAME with capabilities:"
echo "   CPU Cores: \$NODE_CPU_CORES"
echo "   Memory: \$NODE_MEMORY_GB GB"
echo "   Max Chunks: \$NODE_MAX_CHUNKS"
echo "   Encoders: \$NODE_ENCODERS"
echo ""

# Navigate to project root (assuming we're in nodes/node-X/)
cd ../../

# Start the node with auto-registration
cargo run --bin ferris_swarm_node -- \\
    --auto-register \\
    --heartbeat \\
    --heartbeat-interval 30 \\
    --address "0.0.0.0:$PORT"
EOF

    chmod +x "$NODE_NAME/start.sh"
    
    # Create stop script
    cat > "$NODE_NAME/stop.sh" << EOF
#!/bin/bash
echo "🛑 Stopping $NODE_NAME..."
pkill -f "0.0.0.0:$PORT" || echo "Node was not running"
EOF

    chmod +x "$NODE_NAME/stop.sh"
done

# Create master control scripts
cat > start_all.sh << 'EOF'
#!/bin/bash

echo "🚀 Starting all Ferris Swarm nodes..."

for node_dir in node-*; do
    if [ -d "$node_dir" ]; then
        echo "Starting $node_dir..."
        cd "$node_dir"
        ./start.sh &
        cd ..
        sleep 2
    fi
done

echo "✅ All nodes started!"
echo "🌐 Check dashboard: http://localhost:3030"
EOF

chmod +x start_all.sh

cat > stop_all.sh << 'EOF'
#!/bin/bash

echo "🛑 Stopping all Ferris Swarm nodes..."

for node_dir in node-*; do
    if [ -d "$node_dir" ]; then
        echo "Stopping $node_dir..."
        cd "$node_dir"
        ./stop.sh
        cd ..
    fi
done

# Cleanup any remaining processes
pkill -f "ferris_swarm_node" 2>/dev/null || true

echo "✅ All nodes stopped!"
EOF

chmod +x stop_all.sh

cat > status.sh << 'EOF'
#!/bin/bash

echo "📊 Ferris Swarm Status"
echo "====================="

# Check constellation
if curl -s http://localhost:3030/api/status > /dev/null 2>&1; then
    echo "🟢 Constellation: Running"
    curl -s http://localhost:3030/api/status | jq .
else
    echo "🔴 Constellation: Not running"
fi

echo ""
echo "🖥️  Node Processes:"
ps aux | grep "ferris_swarm_node" | grep -v grep || echo "No node processes found"
EOF

chmod +x status.sh

# Go back to project root
cd ..

echo ""
echo "✅ Deployment setup completed!"
echo ""
echo "📁 Directory structure created:"
echo "   nodes/"
echo "   ├── node-1/ (start.sh, stop.sh)"
echo "   ├── node-2/ (start.sh, stop.sh)"
echo "   ├── node-3/ (start.sh, stop.sh)"
echo "   ├── start_all.sh"
echo "   ├── stop_all.sh"
echo "   └── status.sh"
echo ""
echo "🚀 To deploy:"
echo "   1. Start constellation: cargo run --bin ferris_swarm_constellation start"
echo "   2. Start all nodes: cd nodes && ./start_all.sh"
echo "   3. Check status: cd nodes && ./status.sh"
echo "   4. Stop all nodes: cd nodes && ./stop_all.sh"
echo ""
echo "🌐 Dashboard will be available at: http://$CONSTELLATION_HOST:3030"