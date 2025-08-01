# mDNS Implementation for Ferris Swarm

## Overview

The Ferris Swarm project now includes real mDNS (Multicast DNS) service discovery functionality, replacing the previous placeholder implementation. This allows constellation servers to automatically advertise their services on the local network, and nodes/clients to discover them without manual configuration.

## Features Implemented

### 1. **mDNS Service Advertisement** (Constellation)
- Constellation servers automatically advertise themselves on the local network
- Service type: `_ferris-swarm._tcp.local`
- Includes TXT records with version and service information
- Enabled by default, can be disabled with `--no-mdns` flag

### 2. **mDNS Service Discovery** (Nodes & Clients)
- Nodes automatically discover constellation services during auto-registration
- Fallback to manual configuration if mDNS discovery fails
- Timeout-based discovery with configurable duration
- Health check verification of discovered services

### 3. **Network Resilience**
- Graceful fallback to common local addresses (localhost, etc.)
- Local network scanning for constellation services
- HTTP health check verification before using discovered services
- Timeout handling for network operations

## Usage

### Constellation Server
```bash
# Start with mDNS advertisement (default)
cargo run --bin ferris_swarm_constellation start --bind 0.0.0.0:3030

# Disable mDNS advertisement
cargo run --bin ferris_swarm_constellation start --bind 0.0.0.0:3030 --no-mdns
```

### Node Auto-Registration
```bash
# Auto-register with mDNS discovery (default)
cargo run --bin node -- --address 0.0.0.0:8080

# Manual constellation URL (bypasses mDNS)
cargo run --bin node -- --address 0.0.0.0:8080 --constellation-url http://192.168.1.100:3030
```

### Testing mDNS Functionality
```bash
# Run the mDNS test script
./test_mdns.sh
```

## Architecture

### Service Discovery Flow
1. **Advertisement**: Constellation starts and broadcasts mDNS service announcement
2. **Discovery**: Nodes query for `_ferris-swarm._tcp.local` services
3. **Resolution**: Parse mDNS responses to extract IP address and port
4. **Verification**: HTTP health check to confirm constellation availability
5. **Registration**: Node registers with discovered constellation

### Network Protocol
- **Service Type**: `_ferris-swarm._tcp.local`
- **Port**: Configurable (default: 3030 for constellation, 8080+ for nodes)
- **Records**:
  - A records: IP address mapping
  - SRV records: Service port and target
  - TXT records: Service metadata (version, type)

## Configuration

### mDNS Settings
- **Discovery Timeout**: 5 seconds (main discovery)
- **Query Timeout**: 3 seconds (mDNS query)
- **Health Check Timeout**: 500ms per service
- **Advertisement Interval**: 30 seconds

### Fallback Discovery
If mDNS fails, the system tries:
1. `localhost:3030`
2. `constellation:3030` (Docker/hostname resolution)
3. Local network scan: `192.168.x.{1,10,100,101,200}:3030`

## Benefits

1. **Zero Configuration**: Nodes automatically find constellation services
2. **Network Flexibility**: Works across different network topologies
3. **Development Friendly**: No manual IP configuration needed
4. **Production Ready**: Graceful fallback when mDNS is unavailable
5. **Docker Compatible**: Works in containerized environments

## Implementation Details

### Dependencies
- `mdns = "3.0.0"`: Core mDNS functionality
- `futures`: Stream processing for mDNS responses
- `if-addrs`: Network interface discovery
- `reqwest`: HTTP health checks

### Key Components
- `DiscoveryService`: Main service discovery interface
- `ConstellationInfo`: Service information structure
- `parse_mdns_response()`: mDNS response parser
- `advertise_constellation()`: Service advertisement
- `discover_constellation()`: Service discovery

## Troubleshooting

### Common Issues
1. **mDNS Not Working**: Check firewall settings for multicast traffic
2. **Network Isolation**: Use manual constellation URL in isolated networks
3. **Multiple Constellations**: First discovered service is used
4. **Docker Networks**: Ensure proper network configuration for multicast

### Debug Logging
Enable verbose logging to debug mDNS issues:
```bash
RUST_LOG=debug cargo run --bin node
```

### Testing Local Network
```bash
# Test mDNS discovery without starting full services
cargo test --package ferris-swarm-discovery test_get_local_ip
```

## Future Enhancements
- Service priority and weight support
- Multiple constellation load balancing
- IPv6 support
- Service health monitoring
- Custom service metadata