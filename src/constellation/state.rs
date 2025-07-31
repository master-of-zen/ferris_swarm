use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::constellation::config::ConstellationConfig;
use crate::constellation::models::*;

#[derive(Debug, Clone)]
pub struct ConstellationState {
    pub nodes: Arc<RwLock<HashMap<Uuid, NodeInfo>>>,
    pub clients: Arc<RwLock<HashMap<Uuid, ClientInfo>>>,
    pub jobs: Arc<RwLock<HashMap<Uuid, JobInfo>>>,
    pub chunks: Arc<RwLock<HashMap<Uuid, ChunkAssignment>>>,
    pub config: Arc<ConstellationConfig>,
}

impl ConstellationState {
    pub fn new(config: ConstellationConfig) -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            clients: Arc::new(RwLock::new(HashMap::new())),
            jobs: Arc::new(RwLock::new(HashMap::new())),
            chunks: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(config),
        }
    }

    pub async fn register_node(&self, registration: NodeRegistration) -> Uuid {
        let node_id = Uuid::new_v4();
        let node_info = NodeInfo {
            id: node_id,
            address: registration.address,
            status: NodeStatus::Online,
            last_heartbeat: Utc::now(),
            capabilities: registration.capabilities,
            current_chunks: Vec::new(),
            total_processed: 0,
            total_failed: 0,
        };

        let mut nodes = self.nodes.write().await;
        nodes.insert(node_id, node_info);
        info!("Registered new node: {} at {}", node_id, registration.address);
        
        node_id
    }

    pub async fn register_client(&self, registration: ClientRegistration) -> Uuid {
        let client_id = Uuid::new_v4();
        let client_info = ClientInfo {
            id: client_id,
            address: registration.address,
            status: ClientStatus::Connected,
            last_heartbeat: Utc::now(),
            active_jobs: Vec::new(),
        };

        let mut clients = self.clients.write().await;
        clients.insert(client_id, client_info);
        info!("Registered new client: {} at {}", client_id, registration.address);
        
        client_id
    }

    pub async fn update_node_heartbeat(&self, node_id: Uuid, status: NodeStatus) -> bool {
        let mut nodes = self.nodes.write().await;
        if let Some(node) = nodes.get_mut(&node_id) {
            node.last_heartbeat = Utc::now();
            node.status = status;
            debug!("Updated heartbeat for node: {}", node_id);
            true
        } else {
            warn!("Attempted to update heartbeat for unknown node: {}", node_id);
            false
        }
    }

    pub async fn update_client_heartbeat(&self, client_id: Uuid, status: ClientStatus) -> bool {
        let mut clients = self.clients.write().await;
        if let Some(client) = clients.get_mut(&client_id) {
            client.last_heartbeat = Utc::now();
            client.status = status;
            debug!("Updated heartbeat for client: {}", client_id);
            true
        } else {
            warn!("Attempted to update heartbeat for unknown client: {}", client_id);
            false
        }
    }

    pub async fn assign_chunk(&self, job_id: Uuid, chunk_index: u32, node_id: Uuid) -> Option<Uuid> {
        let chunk_id = Uuid::new_v4();
        let assignment = ChunkAssignment {
            chunk_id,
            job_id,
            chunk_index,
            node_id,
            status: ChunkStatus::Assigned,
            assigned_at: Utc::now(),
            started_at: None,
            completed_at: None,
            progress_percent: 0,
        };

        let mut chunks = self.chunks.write().await;
        chunks.insert(chunk_id, assignment);

        let mut nodes = self.nodes.write().await;
        if let Some(node) = nodes.get_mut(&node_id) {
            node.current_chunks.push(assignment.clone());
            info!("Assigned chunk {} to node {}", chunk_id, node_id);
            Some(chunk_id)
        } else {
            warn!("Attempted to assign chunk to unknown node: {}", node_id);
            chunks.remove(&chunk_id);
            None
        }
    }

    pub async fn update_chunk_status(&self, chunk_id: Uuid, update: ChunkUpdate) -> bool {
        let mut chunks = self.chunks.write().await;
        if let Some(chunk) = chunks.get_mut(&chunk_id) {
            chunk.status = update.status.clone();
            chunk.progress_percent = update.progress_percent;
            
            match update.status {
                ChunkStatus::InProgress => {
                    if chunk.started_at.is_none() {
                        chunk.started_at = Some(Utc::now());
                    }
                },
                ChunkStatus::Completed | ChunkStatus::Failed(_) | ChunkStatus::Cancelled => {
                    chunk.completed_at = Some(Utc::now());
                },
                _ => {}
            }

            debug!("Updated chunk {} status to {:?}", chunk_id, update.status);
            true
        } else {
            warn!("Attempted to update status for unknown chunk: {}", chunk_id);
            false
        }
    }

    pub async fn create_job(&self, client_id: Uuid, video_file: String, encoder_parameters: Vec<String>) -> Uuid {
        let job_id = Uuid::new_v4();
        let job_info = JobInfo {
            id: job_id,
            client_id,
            video_file,
            total_chunks: 0,
            completed_chunks: 0,
            failed_chunks: 0,
            status: JobStatus::Queued,
            started_at: Utc::now(),
            estimated_completion: None,
            encoder_parameters,
        };

        let mut jobs = self.jobs.write().await;
        jobs.insert(job_id, job_info);

        let mut clients = self.clients.write().await;
        if let Some(client) = clients.get_mut(&client_id) {
            client.active_jobs.push(job_info.clone());
        }

        info!("Created new job: {} for client: {}", job_id, client_id);
        job_id
    }

    pub async fn get_dashboard_data(&self) -> DashboardData {
        let nodes = self.nodes.read().await.clone();
        let clients = self.clients.read().await.clone();
        let jobs = self.jobs.read().await.clone();
        let chunks = self.chunks.read().await.clone();

        let stats = self.calculate_stats(&nodes, &clients, &jobs, &chunks).await;

        DashboardData {
            nodes,
            clients,
            jobs,
            chunks,
            stats,
            last_updated: Utc::now(),
        }
    }

    async fn calculate_stats(
        &self,
        nodes: &HashMap<Uuid, NodeInfo>,
        clients: &HashMap<Uuid, ClientInfo>,
        jobs: &HashMap<Uuid, JobInfo>,
        chunks: &HashMap<Uuid, ChunkAssignment>,
    ) -> SystemStats {
        let total_nodes = nodes.len() as u32;
        let active_nodes = nodes.values()
            .filter(|n| matches!(n.status, NodeStatus::Online | NodeStatus::Busy))
            .count() as u32;

        let total_clients = clients.len() as u32;
        let active_jobs = jobs.values()
            .filter(|j| matches!(j.status, JobStatus::InProgress | JobStatus::Queued))
            .count() as u32;

        let total_chunks_processed = nodes.values()
            .map(|n| n.total_processed)
            .sum::<u64>();

        let completed_chunks: Vec<_> = chunks.values()
            .filter(|c| matches!(c.status, ChunkStatus::Completed))
            .collect();

        let average_chunk_time = if !completed_chunks.is_empty() {
            let total_time: f64 = completed_chunks.iter()
                .filter_map(|c| {
                    if let (Some(started), Some(completed)) = (c.started_at, c.completed_at) {
                        Some((completed - started).num_milliseconds() as f64 / 1000.0)
                    } else {
                        None
                    }
                })
                .sum();
            total_time / completed_chunks.len() as f64
        } else {
            0.0
        };

        let system_load = (active_nodes as f64) / (total_nodes.max(1) as f64);

        SystemStats {
            total_nodes,
            active_nodes,
            total_clients,
            active_jobs,
            total_chunks_processed,
            average_chunk_time,
            system_load,
        }
    }

    pub fn start_cleanup_task(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                self.cleanup_stale_entries().await;
            }
        })
    }

    async fn cleanup_stale_entries(&self) {
        let now = Utc::now();
        let node_timeout = Duration::from_secs(self.config.server.node_timeout_seconds);
        let client_timeout = Duration::from_secs(self.config.server.client_timeout_seconds);

        {
            let mut nodes = self.nodes.write().await;
            let stale_nodes: Vec<Uuid> = nodes.iter()
                .filter(|(_, node)| {
                    (now - node.last_heartbeat).to_std().unwrap_or_default() > node_timeout
                })
                .map(|(id, _)| *id)
                .collect();

            for node_id in stale_nodes {
                if let Some(mut node) = nodes.remove(&node_id) {
                    node.status = NodeStatus::Offline;
                    nodes.insert(node_id, node);
                    warn!("Marked node {} as offline due to timeout", node_id);
                }
            }
        }

        {
            let mut clients = self.clients.write().await;
            let stale_clients: Vec<Uuid> = clients.iter()
                .filter(|(_, client)| {
                    (now - client.last_heartbeat).to_std().unwrap_or_default() > client_timeout
                })
                .map(|(id, _)| *id)
                .collect();

            for client_id in stale_clients {
                if let Some(mut client) = clients.remove(&client_id) {
                    client.status = ClientStatus::Disconnected;
                    clients.insert(client_id, client);
                    warn!("Marked client {} as disconnected due to timeout", client_id);
                }
            }
        }
    }
}