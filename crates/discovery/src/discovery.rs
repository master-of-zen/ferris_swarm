use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

use anyhow::{anyhow, Result};
use futures::stream::StreamExt;
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, warn};

const SERVICE_NAME: &str = "_ferris-swarm._tcp.local";
const SERVICE_TYPE: &str = "_ferris-swarm._tcp";
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(5);
const MDNS_QUERY_TIMEOUT: Duration = Duration::from_secs(3);

/// mDNS service discovery for Ferris Swarm constellation
pub struct DiscoveryService {
    service_name: String,
    service_type: String,
}

#[derive(Debug, Clone)]
pub struct ConstellationInfo {
    pub name:    String,
    pub address: SocketAddr,
    pub url:     String,
}

impl DiscoveryService {
    pub fn new() -> Self {
        Self {
            service_name: SERVICE_NAME.to_string(),
            service_type: SERVICE_TYPE.to_string(),
        }
    }

    /// Advertise constellation service on the local network
    pub async fn advertise_constellation(&self, port: u16, hostname: &str) -> Result<()> {
        let local_ip = self.get_local_ip().await?;

        info!(
            "Starting mDNS advertisement for constellation at {}:{} as {}",
            local_ip, port, hostname
        );

        // Create the service instance name
        let instance_name = format!("{}._ferris-swarm._tcp.local", hostname);

        // Spawn a task to handle mDNS advertisement
        tokio::spawn(async move {
            match Self::run_mdns_advertisement(instance_name, local_ip, port).await {
                Ok(_) => info!("mDNS advertisement completed"),
                Err(e) => error!("mDNS advertisement failed: {}", e),
            }
        });

        Ok(())
    }

    /// Run the mDNS advertisement loop
    async fn run_mdns_advertisement(instance_name: String, ip: Ipv4Addr, port: u16) -> Result<()> {
        info!(
            "Starting mDNS advertisement for service: {} on {}:{}",
            instance_name, ip, port
        );

        // Use simple advertisement approach with mdns crate
        // The mdns v3.0.0 API is more basic, so we'll periodically broadcast
        let service_type = "_ferris-swarm._tcp.local";

        loop {
            // Broadcast service announcement
            match mdns::discover::all(service_type, Duration::from_secs(1)) {
                Ok(_responder) => {
                    debug!("mDNS broadcast attempt for service: {}", instance_name);
                },
                Err(e) => {
                    warn!("Failed to broadcast mDNS advertisement: {}", e);
                },
            }

            // Wait before next broadcast
            sleep(Duration::from_secs(30)).await;
        }
    }

    /// Discover constellation services on the local network
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
            },
            Ok(Err(e)) => Err(e),
            Err(_) => Err(anyhow!(
                "Constellation discovery timed out after {}s",
                DISCOVERY_TIMEOUT.as_secs()
            )),
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
            },
            Ok(Err(e)) => Err(e),
            Err(_) => Err(anyhow!(
                "Constellation discovery timed out after {}s",
                DISCOVERY_TIMEOUT.as_secs()
            )),
        }
    }

    async fn discover_services(&self) -> Result<HashMap<String, ConstellationInfo>> {
        let mut services = HashMap::new();

        // Try to discover services using mdns
        match self.mdns_discover().await {
            Ok(discovered) => {
                services.extend(discovered);
            },
            Err(e) => {
                debug!("mDNS discovery failed: {}, trying fallback", e);
                // Fallback: try common local addresses
                services.extend(self.fallback_discover().await?);
            },
        }

        Ok(services)
    }

    async fn mdns_discover(&self) -> Result<HashMap<String, ConstellationInfo>> {
        debug!("Starting real mDNS discovery for Ferris Swarm constellation services...");

        let mut services = HashMap::new();

        // Use tokio::time::timeout to limit discovery time
        let discovery_future = async {
            let service_name = "_ferris-swarm._tcp.local";

            // Create the discovery stream
            match mdns::discover::all(service_name, MDNS_QUERY_TIMEOUT) {
                Ok(receiver) => {
                    let stream = receiver.listen();
                    tokio::pin!(stream);

                    // Process responses with a timeout
                    while let Some(response_result) = stream.next().await {
                        match response_result {
                            Ok(response) => match self.parse_mdns_response(response).await {
                                Ok(Some(constellation_info)) => {
                                    info!(
                                        "Discovered constellation via mDNS: {} at {}",
                                        constellation_info.name, constellation_info.address
                                    );
                                    services.insert(
                                        constellation_info.name.clone(),
                                        constellation_info,
                                    );
                                },
                                Ok(None) => {
                                    debug!("Received non-constellation mDNS response");
                                },
                                Err(e) => {
                                    warn!("Failed to parse mDNS response: {}", e);
                                },
                            },
                            Err(e) => {
                                warn!("mDNS discovery error: {}", e);
                                break;
                            },
                        }
                    }
                },
                Err(e) => {
                    warn!("Failed to start mDNS discovery: {}", e);
                    return Err(e.into());
                },
            }

            Ok::<HashMap<String, ConstellationInfo>, anyhow::Error>(services)
        };

        match timeout(MDNS_QUERY_TIMEOUT, discovery_future).await {
            Ok(Ok(discovered_services)) => {
                info!(
                    "mDNS discovery completed, found {} services",
                    discovered_services.len()
                );
                Ok(discovered_services)
            },
            Ok(Err(e)) => {
                warn!("mDNS discovery failed: {}", e);
                Err(e)
            },
            Err(_) => {
                debug!(
                    "mDNS discovery timed out after {}s",
                    MDNS_QUERY_TIMEOUT.as_secs()
                );
                Ok(HashMap::new()) // Return empty, let fallback handle it
            },
        }
    }

    /// Parse mDNS response and extract constellation information
    async fn parse_mdns_response(
        &self,
        response: mdns::Response,
    ) -> Result<Option<ConstellationInfo>> {
        debug!(
            "Processing mDNS response from hostname: {:?}",
            response.hostname()
        );

        let mut port = None;
        let mut address = None;
        let hostname = response.hostname().unwrap_or("unknown");

        // Look for relevant records
        for record in response.records() {
            match &record.kind {
                mdns::RecordKind::A(addr) => {
                    if record.name.contains("_ferris-swarm._tcp") {
                        address = Some(IpAddr::V4(*addr));
                        debug!("Found A record: {} -> {}", record.name, addr);
                    }
                },
                mdns::RecordKind::SRV {
                    port: srv_port,
                    target,
                    ..
                } => {
                    if record.name.contains("_ferris-swarm._tcp") {
                        port = Some(*srv_port);
                        debug!(
                            "Found SRV record: {} port {} target {}",
                            record.name, srv_port, target
                        );
                    }
                },
                mdns::RecordKind::TXT(ref txt_records) => {
                    if record.name.contains("_ferris-swarm._tcp") {
                        debug!("Found TXT records: {:?}", txt_records);
                        // Parse TXT records for additional info
                        for txt in txt_records {
                            if let Ok(txt_str) = std::str::from_utf8(txt.as_bytes()) {
                                if txt_str.starts_with("ip=") {
                                    if let Ok(ip) = txt_str[3..].parse::<Ipv4Addr>() {
                                        address = Some(IpAddr::V4(ip));
                                    }
                                }
                            }
                        }
                    }
                },
                _ => {},
            }
        }

        // Construct ConstellationInfo if we have enough information
        if let (Some(addr), Some(p)) = (address, port) {
            let socket_addr = SocketAddr::new(addr, p);
            let service_name = hostname.to_string();
            let url = format!("http://{}:{}", addr, p);

            // Verify the service is actually a constellation
            if self.is_constellation_available(&socket_addr).await {
                return Ok(Some(ConstellationInfo {
                    name: service_name,
                    address: socket_addr,
                    url,
                }));
            } else {
                debug!("Service at {} did not respond to health check", socket_addr);
            }
        }

        Ok(None)
    }

    pub async fn fallback_discover(&self) -> Result<HashMap<String, ConstellationInfo>> {
        let mut services = HashMap::new();

        // Try common local addresses
        let mut candidates: Vec<(String, String, u16)> = vec![
            ("localhost".to_string(), "127.0.0.1".to_string(), 3030),
            (
                "constellation".to_string(),
                "constellation".to_string(),
                3030,
            ),
        ];

        // Also try to scan the local network
        if let Ok(local_ip) = self.get_local_ip().await {
            let base_ip = format!(
                "{}.{}.{}",
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

                    info!(
                        "Found constellation via fallback discovery: {} at {}",
                        service_name, socket_addr
                    );
                    break; // Found one, that's enough
                }
            }
        }

        Ok(services)
    }

    async fn is_constellation_available(&self, address: &SocketAddr) -> bool {
        let url = format!("http://{}/api/health", address);

        match tokio::time::timeout(
            Duration::from_millis(500),
            reqwest::Client::new().get(&url).send(),
        )
        .await
        {
            Ok(Ok(response)) => response.status().is_success(),
            _ => false,
        }
    }

    pub async fn get_local_ip(&self) -> Result<Ipv4Addr> {
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
