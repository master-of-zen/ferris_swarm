#!/bin/bash

# Ferris Swarm Constellation Test Script
set -e

echo "🦀 Ferris Swarm Constellation Test Suite"
echo "========================================"

# Wait for constellation to start
echo "⏳ Waiting for constellation to start..."
sleep 3

echo "📊 Testing system status..."
curl -s http://localhost:3030/api/status | jq .

echo -e "\n🖥️  Registering Node 1..."
NODE1_RESP=$(curl -s -X POST http://localhost:3030/api/nodes \
  -H "Content-Type: application/json" \
  -d '{
    "address": "192.168.1.101:8080",
    "capabilities": {
      "max_concurrent_chunks": 8,
      "supported_encoders": ["av1", "h264"],
      "cpu_cores": 16,
      "memory_gb": 32
    }
  }')
echo $NODE1_RESP | jq .
NODE1_ID=$(echo $NODE1_RESP | jq -r '.node_id')

echo -e "\n🖥️  Registering Node 2..."
curl -s -X POST http://localhost:3030/api/nodes \
  -H "Content-Type: application/json" \
  -d '{
    "address": "192.168.1.102:8080",
    "capabilities": {
      "max_concurrent_chunks": 4,
      "supported_encoders": ["h264"],
      "cpu_cores": 8,
      "memory_gb": 16
    }
  }' | jq .

echo -e "\n👥 Registering Client..."
CLIENT_RESP=$(curl -s -X POST http://localhost:3030/api/clients \
  -H "Content-Type: application/json" \
  -d '{
    "address": "192.168.1.200:9090"
  }')
echo $CLIENT_RESP | jq .
CLIENT_ID=$(echo $CLIENT_RESP | jq -r '.client_id')

echo -e "\n💓 Sending Node Heartbeat..."
curl -s -X PUT http://localhost:3030/api/nodes/$NODE1_ID/heartbeat \
  -H "Content-Type: application/json" \
  -d '{
    "id": "'$NODE1_ID'",
    "status": "busy",
    "current_load": 0.8
  }' | jq .

echo -e "\n🎬 Creating Job..."
JOB_RESP=$(curl -s -X POST http://localhost:3030/api/jobs \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "'$CLIENT_ID'",
    "video_file": "test_video.mp4",
    "encoder_parameters": ["--crf", "23"]
  }')
echo $JOB_RESP | jq .

echo -e "\n📈 Final Status Check..."
curl -s http://localhost:3030/api/status | jq .

echo -e "\n📊 Dashboard Statistics..."
curl -s http://localhost:3030/api/dashboard/data | jq '.stats'

echo -e "\n✅ All tests completed successfully!"
echo "🌐 Dashboard: http://localhost:3030"
echo "🔌 WebSocket: ws://localhost:3030/ws"