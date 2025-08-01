use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::{models::*, state::ConstellationState};

pub async fn register_node(
    State(state): State<ConstellationState>,
    Json(registration): Json<NodeRegistration>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let node_id = state.register_node(registration).await;

    Ok(Json(json!({
        "node_id": node_id,
        "status": "registered"
    })))
}

pub async fn register_client(
    State(state): State<ConstellationState>,
    Json(registration): Json<ClientRegistration>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let client_id = state.register_client(registration).await;

    Ok(Json(json!({
        "client_id": client_id,
        "status": "registered"
    })))
}

pub async fn node_heartbeat(
    Path(node_id): Path<Uuid>,
    State(state): State<ConstellationState>,
    Json(heartbeat): Json<HeartbeatRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let status = match heartbeat.status.as_str() {
        "online" => NodeStatus::Online,
        "busy" => NodeStatus::Busy,
        "offline" => NodeStatus::Offline,
        error_msg => NodeStatus::Error(error_msg.to_string()),
    };

    if state.update_node_heartbeat(node_id, status).await {
        Ok(Json(json!({ "status": "updated" })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn client_heartbeat(
    Path(client_id): Path<Uuid>,
    State(state): State<ConstellationState>,
    Json(heartbeat): Json<HeartbeatRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let status = match heartbeat.status.as_str() {
        "connected" => ClientStatus::Connected,
        "processing" => ClientStatus::Processing,
        "disconnected" => ClientStatus::Disconnected,
        _ => ClientStatus::Disconnected,
    };

    if state.update_client_heartbeat(client_id, status).await {
        Ok(Json(json!({ "status": "updated" })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn create_job(
    State(state): State<ConstellationState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let client_id = request["client_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let video_file = request["video_file"].as_str().ok_or(StatusCode::BAD_REQUEST)?.to_string();

    let encoder_parameters = request["encoder_parameters"]
        .as_array()
        .ok_or(StatusCode::BAD_REQUEST)?
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();

    let job_id = state.create_job(client_id, video_file, encoder_parameters).await;

    Ok(Json(json!({
        "job_id": job_id,
        "status": "created"
    })))
}

pub async fn update_job(
    Path(job_id): Path<Uuid>,
    State(state): State<ConstellationState>,
    Json(update): Json<JobUpdate>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut jobs = state.jobs.write().await;
    if let Some(job) = jobs.get_mut(&job_id) {
        job.status = update.status;
        if let Some(total) = update.total_chunks {
            job.total_chunks = total;
        }
        if let Some(completed) = update.completed_chunks {
            job.completed_chunks = completed;
        }
        if let Some(failed) = update.failed_chunks {
            job.failed_chunks = failed;
        }

        Ok(Json(json!({ "status": "updated" })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn update_chunk(
    Path(chunk_id): Path<Uuid>,
    State(state): State<ConstellationState>,
    Json(update): Json<ChunkUpdate>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if state.update_chunk_status(chunk_id, update).await {
        Ok(Json(json!({ "status": "updated" })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn get_dashboard_data(State(state): State<ConstellationState>) -> Json<DashboardData> {
    Json(state.get_dashboard_data().await)
}

pub async fn get_status(State(state): State<ConstellationState>) -> Json<serde_json::Value> {
    let nodes_count = state.nodes.read().await.len();
    let clients_count = state.clients.read().await.len();
    let jobs_count = state.jobs.read().await.len();
    let chunks_count = state.chunks.read().await.len();

    Json(json!({
        "service": "constellation",
        "status": "running",
        "nodes": nodes_count,
        "clients": clients_count,
        "jobs": jobs_count,
        "chunks": chunks_count,
        "uptime": "N/A"
    }))
}

pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "ferris-swarm-constellation",
        "version": "1.0"
    }))
}
