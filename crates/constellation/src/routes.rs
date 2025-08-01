use axum::{
    extract::{State, WebSocketUpgrade},
    response::Html,
    routing::{get, post, put},
    Router,
};
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::{handlers::*, state::ConstellationState, websocket::websocket_handler};

pub fn create_router(state: ConstellationState) -> Router {
    let api_routes = Router::new()
        .route("/nodes", post(register_node))
        .route("/nodes/:id/heartbeat", put(node_heartbeat))
        .route("/clients", post(register_client))
        .route("/clients/:id/heartbeat", put(client_heartbeat))
        .route("/jobs", post(create_job))
        .route("/jobs/:id", put(update_job))
        .route("/chunks/:id", put(update_chunk))
        .route("/dashboard/data", get(get_dashboard_data))
        .route("/status", get(get_status))
        .route("/health", get(health_check));

    Router::new()
        .route("/", get(dashboard))
        .route("/ws", get(websocket_upgrade))
        .nest("/api", api_routes)
        .nest_service("/static", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn dashboard() -> Html<&'static str> {
    Html(include_str!("../../../static/dashboard.html"))
}

async fn websocket_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<ConstellationState>,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| websocket_handler(socket, state))
}
