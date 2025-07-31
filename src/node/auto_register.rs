use std::net::SocketAddr;
use std::time::Duration;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::time::{interval, sleep};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    constellation::models::{NodeCapabilities, NodeRegistration},
    discovery::DiscoveryService,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRegisterConfig {
    pub constellation_url: String,
    pub node_name: String,
    pub capabilities: NodeCapabilities,
    pub heartbeat_interval: Duration,
}

pub struct NodeAutoRegister {
    config: AutoRegisterConfig,
    client: reqwest::Client,
    node_id: Option<Uuid>,
    node_address: SocketAddr,
}

impl NodeAutoRegister {
    pub fn new(config: AutoRegisterConfig, node_address: SocketAddr) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
            node_id: None,
            node_address,
        }
    }

    pub async fn register(&mut self) -> Result<Uuid> {
        let registration = NodeRegistration {
            address: self.node_address,
            capabilities: self.config.capabilities.clone(),
        };

        info!(
            "Registering node '{}' with constellation at {}",
            self.config.node_name, self.config.constellation_url
        );

        let response = self
            .client
            .post(&format!("{}/api/nodes", self.config.constellation_url))
            .json(&registration)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send registration request: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Registration failed with status: {}",
                response.status()
            ));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse registration response: {}", e))?;

        let node_id = result["node_id"]
            .as_str()
            .ok_or_else(|| anyhow!("No node_id in registration response"))?;

        let node_id = Uuid::parse_str(node_id)
            .map_err(|e| anyhow!("Invalid node_id format: {}", e))?;

        self.node_id = Some(node_id);
        info!("Successfully registered as node: {}", node_id);

        Ok(node_id)
    }

    pub async fn start_heartbeat_service(self) -> Result<()> {
        let node_id = self.node_id.ok_or_else(|| anyhow!("Node not registered yet"))?;
        
        info!(
            "Starting heartbeat service for node {} (interval: {}s)",
            node_id,
            self.config.heartbeat_interval.as_secs()
        );

        let mut interval = interval(self.config.heartbeat_interval);

        loop {
            interval.tick().await;

            if let Err(e) = self.send_heartbeat(node_id).await {
                warn!("Heartbeat failed: {}", e);
                
                // Try to re-register if heartbeat fails multiple times
                if let Err(re_register_err) = self.try_reregister().await {
                    error!("Re-registration failed: {}", re_register_err);
                    // Wait longer before next attempt
                    sleep(Duration::from_secs(60)).await;
                }
            } else {
                debug!("Heartbeat sent successfully for node {}", node_id);
            }
        }
    }

    async fn send_heartbeat(&self, node_id: Uuid) -> Result<()> {
        let heartbeat_data = serde_json::json!({
            "id": node_id,
            "status": "online",
            "current_load": self.get_current_load().await
        });

        let response = self
            .client
            .put(&format!(
                "{}/api/nodes/{}/heartbeat",
                self.config.constellation_url, node_id
            ))
            .json(&heartbeat_data)
            .timeout(Duration::from_secs(5))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Heartbeat failed with status: {}", response.status()));
        }

        Ok(())
    }

    async fn try_reregister(&self) -> Result<()> {
        info!("Attempting to re-register with constellation...");
        
        // Create a new instance for re-registration
        let mut new_register = Self::new(self.config.clone(), self.node_address);
        new_register.register().await?;
        
        Ok(())
    }

    async fn get_current_load(&self) -> f64 {
        // Simple load calculation - could be enhanced with actual system metrics
        // For now, return a placeholder
        0.1
    }

    /// Create an auto-register config using mDNS discovery
    pub async fn create_with_discovery(
        node_name: Option<String>,
        capabilities: NodeCapabilities,
        heartbeat_interval: Duration,
    ) -> Result<AutoRegisterConfig> {
        let discovery_service = DiscoveryService::new();
        
        info!("Attempting to discover constellation service via mDNS...");
        
        match discovery_service.discover_constellation().await {
            Ok(constellation_info) => {
                info!("Discovered constellation at: {}", constellation_info.url);
                
                let node_name = node_name.unwrap_or_else(|| {
                    hostname::get()
                        .map(|h| h.to_string_lossy().to_string())
                        .unwrap_or_else(|_| format!("node-{}", uuid::Uuid::new_v4().simple()))
                });

                Ok(AutoRegisterConfig {
                    constellation_url: constellation_info.url,
                    node_name,
                    capabilities,
                    heartbeat_interval,
                })
            }
            Err(e) => {
                error!("Failed to discover constellation via mDNS: {}", e);
                Err(anyhow!("Constellation discovery failed: {}", e))
            }
        }
    }

    /// Try to create config with discovery, fallback to manual URL
    pub async fn create_with_discovery_fallback(
        constellation_url: Option<String>,
        node_name: Option<String>,
        capabilities: NodeCapabilities,
        heartbeat_interval: Duration,
    ) -> Result<AutoRegisterConfig> {
        // If URL is provided, use it directly
        if let Some(url) = constellation_url {
            let node_name = node_name.unwrap_or_else(|| {
                hostname::get()
                    .map(|h| h.to_string_lossy().to_string())
                    .unwrap_or_else(|_| format!("node-{}", uuid::Uuid::new_v4().simple()))
            });

            return Ok(AutoRegisterConfig {
                constellation_url: url,
                node_name,
                capabilities,
                heartbeat_interval,
            });
        }

        // Try mDNS discovery first
        match Self::create_with_discovery(node_name.clone(), capabilities.clone(), heartbeat_interval).await {
            Ok(config) => {
                info!("Using constellation discovered via mDNS");
                Ok(config)
            }
            Err(discovery_err) => {
                warn!("mDNS discovery failed: {}", discovery_err);
                
                // Fallback to default local constellation
                let fallback_url = "http://localhost:3030".to_string();
                warn!("Falling back to default constellation URL: {}", fallback_url);
                
                let node_name = node_name.unwrap_or_else(|| {
                    hostname::get()
                        .map(|h| h.to_string_lossy().to_string())
                        .unwrap_or_else(|_| format!("node-{}", uuid::Uuid::new_v4().simple()))
                });

                Ok(AutoRegisterConfig {
                    constellation_url: fallback_url,
                    node_name,
                    capabilities,
                    heartbeat_interval,
                })
            }
        }
    }
}

pub fn detect_node_capabilities(
    cpu_cores_override: Option<u32>,
    memory_gb_override: Option<u32>,
    max_chunks_override: Option<u32>,
    encoders_override: Option<String>,
) -> Result<NodeCapabilities> {
    let cpu_cores = cpu_cores_override.unwrap_or_else(|| num_cpus::get() as u32);
    
    let memory_gb = memory_gb_override.unwrap_or_else(|| {
        detect_system_memory_gb().unwrap_or(8)
    });

    let supported_encoders = if let Some(encoders_str) = encoders_override {
        encoders_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    } else {
        detect_available_encoders()
    };

    let max_concurrent_chunks = max_chunks_override.unwrap_or_else(|| {
        (cpu_cores / 2).max(1)
    });

    Ok(NodeCapabilities {
        max_concurrent_chunks,
        supported_encoders,
        cpu_cores,
        memory_gb,
    })
}

fn detect_system_memory_gb() -> Option<u32> {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .find(|line| line.starts_with("MemTotal:"))
                    .and_then(|line| line.split_whitespace().nth(1))
                    .and_then(|kb| kb.parse::<u64>().ok())
                    .map(|kb| (kb / 1024 / 1024) as u32)
            })
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        // For non-Linux systems, could implement using sysinfo crate
        None
    }
}

fn detect_available_encoders() -> Vec<String> {
    let mut encoders = Vec::new();
    
    // Check for common encoders by trying to get their help
    let potential_encoders = [
        ("libx264", "h264"),
        ("libx265", "hevc"),
        ("libaom-av1", "av1"),
        ("libvpx-vp9", "vp9"),
    ];

    for (ffmpeg_name, encoder_name) in potential_encoders {
        if check_encoder_available(ffmpeg_name) {
            encoders.push(encoder_name.to_string());
        }
    }

    // If no encoders detected, provide defaults
    if encoders.is_empty() {
        encoders.push("h264".to_string());
    }

    encoders
}

fn check_encoder_available(encoder_name: &str) -> bool {
    std::process::Command::new("ffmpeg")
        .args(["-hide_banner", "-encoders"])
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout).contains(encoder_name)
        })
        .unwrap_or(false)
}

pub fn get_local_ip() -> Result<std::net::IpAddr> {
    // Try to get the local IP by connecting to a remote address
    let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    let local_addr = socket.local_addr()?;
    Ok(local_addr.ip())
}