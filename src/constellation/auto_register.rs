use std::collections::HashMap;
use std::path::PathBuf;
use std::net::SocketAddr;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::time::{interval, Duration};
use tokio::fs;
use tracing::{debug, info, warn};

use crate::constellation::{
    models::{NodeCapabilities, NodeRegistration},
    state::ConstellationState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodesConfig {
    pub constellation: ConstellationSettings,
    pub nodes: Vec<NodeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstellationSettings {
    pub url: String,
    pub auto_register: bool,
    pub heartbeat_interval: u64,
    pub registration_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub name: String,
    pub address: String,
    pub enabled: bool,
    pub capabilities: NodeCapabilities,
    #[serde(default)]
    pub tags: HashMap<String, String>,
}

impl Default for ConstellationSettings {
    fn default() -> Self {
        Self {
            url: "http://localhost:3030".to_string(),
            auto_register: true,
            heartbeat_interval: 30,
            registration_interval: 60,
        }
    }
}

pub struct AutoRegister {
    config_path: PathBuf,
    state: ConstellationState,
    client: reqwest::Client,
    registered_nodes: HashMap<String, uuid::Uuid>,
}

impl AutoRegister {
    pub fn new(config_path: PathBuf, state: ConstellationState) -> Self {
        Self {
            config_path,
            state,
            client: reqwest::Client::new(),
            registered_nodes: HashMap::new(),
        }
    }

    pub async fn start_auto_registration(&mut self) -> Result<()> {
        info!("Starting auto-registration service from config: {:?}", self.config_path);
        
        // Initial registration
        if let Err(e) = self.process_config().await {
            warn!("Initial auto-registration failed: {}", e);
        }

        // Periodic registration check
        let mut interval = interval(Duration::from_secs(60));
        
        loop {
            interval.tick().await;
            if let Err(e) = self.process_config().await {
                warn!("Auto-registration failed: {}", e);
            }
        }
    }

    async fn process_config(&mut self) -> Result<()> {
        if !self.config_path.exists() {
            debug!("Configuration file does not exist: {:?}", self.config_path);
            return Ok(());
        }

        let config_content = fs::read_to_string(&self.config_path).await?;
        let config: NodesConfig = toml::from_str(&config_content)?;

        if !config.constellation.auto_register {
            debug!("Auto-registration disabled in configuration");
            return Ok(());
        }

        for node_config in config.nodes.iter().filter(|n| n.enabled) {
            if !self.is_node_registered(&node_config.address).await {
                match self.register_node_from_config(node_config).await {
                    Ok(node_id) => {
                        info!("Auto-registered node '{}': {}", node_config.name, node_id);
                        self.registered_nodes.insert(node_config.address.clone(), node_id);
                    },
                    Err(e) => {
                        warn!("Failed to register node '{}': {}", node_config.name, e);
                    }
                }
            } else {
                debug!("Node '{}' already registered", node_config.name);
            }
        }

        Ok(())
    }

    async fn is_node_registered(&self, address: &str) -> bool {
        // Check if already in our local registry
        if self.registered_nodes.contains_key(address) {
            return true;
        }

        // Check constellation state
        let nodes = self.state.nodes.read().await;
        nodes.values().any(|node| node.address.to_string() == address)
    }

    async fn register_node_from_config(&self, node_config: &NodeConfig) -> Result<uuid::Uuid> {
        let address: SocketAddr = node_config.address.parse()?;
        
        let registration = NodeRegistration {
            address,
            capabilities: node_config.capabilities.clone(),
        };

        let node_id = self.state.register_node(registration).await;
        
        info!(
            "Registered node '{}' from config with ID: {} ({})",
            node_config.name, node_id, node_config.address
        );

        Ok(node_id)
    }

    pub async fn generate_sample_config(path: &PathBuf) -> Result<()> {
        let sample_config = NodesConfig {
            constellation: ConstellationSettings::default(),
            nodes: vec![
                NodeConfig {
                    name: "workstation-01".to_string(),
                    address: "192.168.1.101:8080".to_string(),
                    enabled: true,
                    capabilities: NodeCapabilities {
                        max_concurrent_chunks: 8,
                        supported_encoders: vec!["av1".to_string(), "h264".to_string(), "hevc".to_string()],
                        cpu_cores: 16,
                        memory_gb: 32,
                    },
                    tags: {
                        let mut tags = HashMap::new();
                        tags.insert("environment".to_string(), "production".to_string());
                        tags.insert("location".to_string(), "datacenter-1".to_string());
                        tags
                    },
                },
                NodeConfig {
                    name: "workstation-02".to_string(),
                    address: "192.168.1.102:8080".to_string(),
                    enabled: true,
                    capabilities: NodeCapabilities {
                        max_concurrent_chunks: 4,
                        supported_encoders: vec!["h264".to_string(), "hevc".to_string()],
                        cpu_cores: 8,
                        memory_gb: 16,
                    },
                    tags: {
                        let mut tags = HashMap::new();
                        tags.insert("environment".to_string(), "development".to_string());
                        tags.insert("location".to_string(), "office".to_string());
                        tags
                    },
                },
                NodeConfig {
                    name: "gpu-server-01".to_string(),
                    address: "192.168.1.103:8080".to_string(),
                    enabled: false, // Disabled by default
                    capabilities: NodeCapabilities {
                        max_concurrent_chunks: 16,
                        supported_encoders: vec!["av1".to_string(), "h264".to_string(), "hevc".to_string()],
                        cpu_cores: 32,
                        memory_gb: 64,
                    },
                    tags: {
                        let mut tags = HashMap::new();
                        tags.insert("environment".to_string(), "production".to_string());
                        tags.insert("gpu".to_string(), "nvidia-rtx-4090".to_string());
                        tags.insert("priority".to_string(), "high".to_string());
                        tags
                    },
                },
            ],
        };

        let toml_content = toml::to_string_pretty(&sample_config)?;
        fs::write(path, toml_content).await?;
        
        info!("Generated sample nodes configuration at: {:?}", path);
        Ok(())
    }
}