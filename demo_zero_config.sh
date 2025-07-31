#!/bin/bash

# Ferris Swarm Zero-Configuration Demo
# This script demonstrates the new default auto-registration and heartbeat behavior

echo "=== Ferris Swarm Zero-Configuration Demo ==="
echo

echo "🎯 NEW DEFAULT BEHAVIOR:"
echo "✓ Auto-registration: ENABLED by default (use --no-auto-register to disable)"
echo "✓ Heartbeat service: ENABLED by default (use --no-heartbeat to disable)"
echo "✓ mDNS discovery: Automatic constellation discovery on local network"
echo "✓ Fallback: Falls back to localhost:3030 if no constellation found"
echo

# Build first
echo "📦 Building Ferris Swarm..."
cargo build --release --quiet || {
    echo "❌ Build failed"
    exit 1
}
echo "✅ Build completed"
echo

echo "🔍 Help output shows new default behavior:"
echo "----------------------------------------"
./target/release/ferris_swarm_node --help | head -4
echo

echo "🚀 NEW USAGE EXAMPLES:"
echo

echo "1️⃣  ZERO-CONFIG STARTUP (NEW DEFAULT):"
echo "   ./ferris_swarm_node"
echo "   → Automatically discovers constellation"
echo "   → Registers with auto-detected capabilities"
echo "   → Starts heartbeat service"
echo

echo "2️⃣  MINIMAL EXPLICIT CONFIG:"
echo "   ./ferris_swarm_node --constellation-url http://10.0.1.100:3030"
echo "   → Uses specific constellation URL"
echo "   → Still auto-registers and sends heartbeats"
echo

echo "3️⃣  STANDALONE MODE (DISABLE AUTO-FEATURES):"
echo "   ./ferris_swarm_node --no-auto-register --no-heartbeat"
echo "   → Pure encoding node without constellation integration"
echo

echo "4️⃣  CUSTOM CONFIGURATION:"
echo "   ./ferris_swarm_node --node-name my-render-farm-01 --cpu-cores 32 --memory-gb 64"
echo "   → Custom node identity with auto-registration"
echo

echo "📊 COMPARISON - BEFORE vs AFTER:"
echo
echo "BEFORE (manual setup required):"
echo "  ./ferris_swarm_node --auto-register --heartbeat --constellation-url http://constellation:3030"
echo
echo "AFTER (zero-config):"
echo "  ./ferris_swarm_node"
echo

echo "🌟 BENEFITS:"
echo "   • True plug-and-play operation"
echo "   • No configuration files required"
echo "   • Automatic service discovery"
echo "   • Enterprise-ready defaults"
echo "   • Backward compatible"
echo

echo "🧪 Testing zero-config startup..."
echo "Starting node for 5 seconds to demonstrate default behavior..."
echo

timeout 5s ./target/release/ferris_swarm_node --address 127.0.0.1:50099 2>&1 | \
    grep -E "(INFO|WARN|ERROR)" | \
    head -8 || echo

echo
echo "✅ Demo complete!"
echo
echo "💡 TIP: Start a constellation service first, then nodes will discover it automatically:"
echo "   Terminal 1: ./ferris_swarm_constellation start"
echo "   Terminal 2: ./ferris_swarm_node"
echo "   Terminal 3: ./ferris_swarm_node --address :50052"
echo "   Terminal 4: ./ferris_swarm_node --address :50053"
echo
echo "🎉 Welcome to zero-configuration distributed video encoding!"