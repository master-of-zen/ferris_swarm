use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstellationConfig {
    pub server:     ServerConfig,
    pub dashboard:  DashboardConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub bind_address:               SocketAddr,
    pub max_connections:            usize,
    pub heartbeat_interval_seconds: u64,
    pub node_timeout_seconds:       u64,
    pub client_timeout_seconds:     u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub title:               String,
    pub refresh_interval_ms: u64,
    pub max_log_entries:     usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enable_metrics:          bool,
    pub metrics_retention_hours: u64,
    pub alert_thresholds:        AlertThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub node_offline_minutes:       u64,
    pub job_stalled_minutes:        u64,
    pub chunk_failure_rate_percent: f64,
}

impl Default for ConstellationConfig {
    fn default() -> Self {
        Self {
            server:     ServerConfig {
                bind_address:               "0.0.0.0:3030".parse().unwrap(),
                max_connections:            1000,
                heartbeat_interval_seconds: 30,
                node_timeout_seconds:       120,
                client_timeout_seconds:     300,
            },
            dashboard:  DashboardConfig {
                title:               "Ferris Swarm Constellation".to_string(),
                refresh_interval_ms: 1000,
                max_log_entries:     1000,
            },
            monitoring: MonitoringConfig {
                enable_metrics:          true,
                metrics_retention_hours: 24,
                alert_thresholds:        AlertThresholds {
                    node_offline_minutes:       5,
                    job_stalled_minutes:        30,
                    chunk_failure_rate_percent: 10.0,
                },
            },
        }
    }
}
