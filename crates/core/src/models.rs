use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

/// Node capabilities and specifications for registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapabilities {
    pub max_concurrent_chunks: u32,
    pub supported_encoders: Vec<String>,
    pub cpu_cores: u32,
    pub memory_gb: u32,
}

/// Node registration request
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeRegistration {
    pub address: SocketAddr,
    pub capabilities: NodeCapabilities,
}

/// Client registration request  
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientRegistration {
    pub address: SocketAddr,
}