use std::collections::HashMap;
use std::net::SocketAddr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use ferris_swarm_core::{NodeCapabilities, NodeRegistration, ClientRegistration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: Uuid,
    pub address: SocketAddr,
    pub status: NodeStatus,
    pub last_heartbeat: DateTime<Utc>,
    pub capabilities: NodeCapabilities,
    pub current_chunks: Vec<ChunkAssignment>,
    pub total_processed: u64,
    pub total_failed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Online,
    Busy,
    Offline,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub id: Uuid,
    pub address: SocketAddr,
    pub status: ClientStatus,
    pub last_heartbeat: DateTime<Utc>,
    pub active_jobs: Vec<JobInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientStatus {
    Connected,
    Processing,
    Disconnected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub id: Uuid,
    pub client_id: Uuid,
    pub video_file: String,
    pub total_chunks: u32,
    pub completed_chunks: u32,
    pub failed_chunks: u32,
    pub status: JobStatus,
    pub started_at: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub encoder_parameters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Queued,
    InProgress,
    Completed,
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkAssignment {
    pub chunk_id: Uuid,
    pub job_id: Uuid,
    pub chunk_index: u32,
    pub node_id: Uuid,
    pub status: ChunkStatus,
    pub assigned_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub progress_percent: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkStatus {
    Assigned,
    InProgress,
    Completed,
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub total_nodes: u32,
    pub active_nodes: u32,
    pub total_clients: u32,
    pub active_jobs: u32,
    pub total_chunks_processed: u64,
    pub average_chunk_time: f64,
    pub system_load: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub id: Uuid,
    pub status: String,
    pub current_load: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkUpdate {
    pub chunk_id: Uuid,
    pub status: ChunkStatus,
    pub progress_percent: u8,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobUpdate {
    pub job_id: Uuid,
    pub status: JobStatus,
    pub total_chunks: Option<u32>,
    pub completed_chunks: Option<u32>,
    pub failed_chunks: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardData {
    pub nodes: HashMap<Uuid, NodeInfo>,
    pub clients: HashMap<Uuid, ClientInfo>,
    pub jobs: HashMap<Uuid, JobInfo>,
    pub chunks: HashMap<Uuid, ChunkAssignment>,
    pub stats: SystemStats,
    pub last_updated: DateTime<Utc>,
}