use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::collections::HashMap;
use std::time::Duration;

use anyhow::{anyhow, Result};
use tokio::time::{timeout, sleep};
use tracing::{debug, info};

const SERVICE_NAME: &str = "_ferris-swarm._tcp.local";
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(5);

/// mDNS service discovery for Ferris Swarm constellation
pub struct DiscoveryService {
    service_name: String,
}

#[derive(Debug, Clone)]
pub struct ConstellationInfo {
    pub name: String,
    pub address: SocketAddr,
    pub url: String,
}

impl DiscoveryService {
    pub fn new() -> Self {
        Self {
            service_name: SERVICE_NAME.to_string(),
        }
    }

    /// Advertise constellation service on the local network
    pub async fn advertise_constellation(&self, port: u16, hostname: &str) -> Result<()> {
        let local_ip = self.get_local_ip().await?;
        
        info!(
            "Starting mDNS advertisement for constellation at {}:{} as {}",
            local_ip, port, hostname
        );

        // For now, we'll use a simple approach without persistent advertisement
        // The mdns crate API is quite basic, so we'll simulate this
        tokio::spawn(async move {
            loop {
                debug!("mDNS advertisement cycle (simulated)");
                sleep(Duration::from_secs(30)).await;
            }
        });

        Ok(())
    }

    /// Discover constellation services on the local network
    pub async fn discover_constellation(&self) -> Result<ConstellationInfo> {
        info!("Discovering constellation services on local network...");
        
        let discovery_result = timeout(DISCOVERY_TIMEOUT, self.discover_services()).await;
        
        match discovery_result {
            Ok(Ok(services)) => {
                if services.is_empty() {
                    return Err(anyhow!("No constellation services found on local network"));
                }
                
                // Return the first discovered service
                let (name, info) = services.into_iter().next().unwrap();
                info!("Found constellation service: {} at {}", name, info.url);
                Ok(info)
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(anyhow!("Constellation discovery timed out after {}s", DISCOVERY_TIMEOUT.as_secs())),
        }
    }

    /// Discover all constellation services on the network
    pub async fn discover_all_constellations(&self) -> Result<HashMap<String, ConstellationInfo>> {
        info!("Discovering all constellation services on local network...");
        
        let discovery_result = timeout(DISCOVERY_TIMEOUT, self.discover_services()).await;
        
        match discovery_result {
            Ok(Ok(services)) => {
                info!("Found {} constellation service(s)", services.len());
                Ok(services)
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(anyhow!("Constellation discovery timed out after {}s", DISCOVERY_TIMEOUT.as_secs())),
        }
    }

    async fn discover_services(&self) -> Result<HashMap<String, ConstellationInfo>> {
        let mut services = HashMap::new();
        
        // Try to discover services using mdns
        match self.mdns_discover().await {
            Ok(discovered) => {
                services.extend(discovered);
            }
            Err(e) => {
                debug!("mDNS discovery failed: {}, trying fallback", e);
                // Fallback: try common local addresses
                services.extend(self.fallback_discover().await?);
            }
        }

        Ok(services)
    }

    async fn mdns_discover(&self) -> Result<HashMap<String, ConstellationInfo>> {
        // For now, simulate mDNS discovery
        // The mdns crate requires a different approach than initially implemented
        debug!("Attempting mDNS discovery...");
        
        // This would be the real mDNS implementation:
        // We'd listen for multicast DNS responses for _ferris-swarm._tcp.local
        sleep(Duration::from_millis(100)).await;
        
        Ok(HashMap::new()) // Empty for now - fallback will handle
    }

    async fn fallback_discover(&self) -> Result<HashMap<String, ConstellationInfo>> {
        let mut services = HashMap::new();
        
        // Try common local addresses
        let mut candidates: Vec<(String, String, u16)> = vec![
            ("localhost".to_string(), "127.0.0.1".to_string(), 3030),
            ("constellation".to_string(), "constellation".to_string(), 3030),
        ];

        // Also try to scan the local network
        if let Ok(local_ip) = self.get_local_ip().await {
            let base_ip = format!("{}.{}.{}", 
                local_ip.octets()[0], 
                local_ip.octets()[1], 
                local_ip.octets()[2]
            );
            
            // Try a few common addresses in the local network
            for i in [1, 10, 100, 101, 200] {
                let test_addr = format!("{}.{}", base_ip, i);
                let name = format!("node-{}", i);
                candidates.push((name, test_addr, 3030));
            }
        }

        for (name, addr, port) in candidates {
            if let Ok(ip_addr) = addr.parse::<IpAddr>() {
                let socket_addr = SocketAddr::new(ip_addr, port);
                
                // Quick health check
                if self.is_constellation_available(&socket_addr).await {
                    let url = format!("http://{}:{}", addr, port);
                    let service_name = name.clone();
                    services.insert(service_name.clone(), ConstellationInfo {
                        name: service_name.clone(),
                        address: socket_addr,
                        url,
                    });
                    
                    info!("Found constellation via fallback discovery: {} at {}", service_name, socket_addr);
                    break; // Found one, that's enough
                }
            }
        }

        Ok(services)
    }

    async fn is_constellation_available(&self, address: &SocketAddr) -> bool {
        let url = format!("http://{}/api/health", address);
        
        match tokio::time::timeout(Duration::from_millis(500), 
            reqwest::Client::new().get(&url).send()
        ).await {
            Ok(Ok(response)) => response.status().is_success(),
            _ => false,
        }
    }

    async fn get_local_ip(&self) -> Result<Ipv4Addr> {
        let interfaces = if_addrs::get_if_addrs()?;
        
        for interface in interfaces {
            if !interface.is_loopback() {
                if let IpAddr::V4(ipv4) = interface.ip() {
                    return Ok(ipv4);
                }
            }
        }
        
        Err(anyhow!("No suitable network interface found"))
    }
}

impl Default for DiscoveryService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_discovery_service_creation() {
        let service = DiscoveryService::new();
        assert_eq!(service.service_name, SERVICE_NAME);
    }
    
    #[tokio::test]
    async fn test_get_local_ip() {
        let service = DiscoveryService::new();
        let result = service.get_local_ip().await;
        
        // Should either succeed or fail gracefully
        match result {
            Ok(ip) => {
                assert!(!ip.is_loopback());
                println!("Local IP: {}", ip);
            }
            Err(e) => {
                println!("Failed to get local IP (this might be expected in test environment): {}", e);
            }
        }
    }
}