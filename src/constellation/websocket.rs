use axum::extract::ws::{Message, WebSocket};
use futures::{sink::SinkExt, stream::StreamExt};
use serde_json::json;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

use crate::constellation::state::ConstellationState;

pub async fn websocket_handler(socket: WebSocket, state: ConstellationState) {
    info!("New WebSocket connection established");
    
    let (mut sender, mut receiver) = socket.split();
    let state_clone = state.clone();

    let send_task = tokio::spawn(async move {
        let mut interval = interval(Duration::from_millis(state_clone.config.dashboard.refresh_interval_ms));
        
        loop {
            interval.tick().await;
            
            let dashboard_data = state_clone.get_dashboard_data().await;
            let message = json!({
                "type": "dashboard_update",
                "data": dashboard_data
            });
            
            if let Ok(text) = serde_json::to_string(&message) {
                if sender.send(Message::Text(text)).await.is_err() {
                    debug!("WebSocket send failed, client disconnected");
                    break;
                }
            }
        }
    });

    let receive_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    debug!("Received WebSocket message: {}", text);
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                        handle_websocket_message(parsed, &state).await;
                    }
                },
                Ok(Message::Close(_)) => {
                    debug!("WebSocket connection closed");
                    break;
                },
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                },
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = send_task => {
            debug!("WebSocket send task completed");
        },
        _ = receive_task => {
            debug!("WebSocket receive task completed");
        }
    }

    info!("WebSocket connection closed");
}

async fn handle_websocket_message(message: serde_json::Value, _state: &ConstellationState) {
    let msg_type = message.get("type").and_then(|v| v.as_str());
    
    match msg_type {
        Some("ping") => {
            debug!("Received ping from WebSocket client");
        },
        Some("request_update") => {
            debug!("Manual update requested via WebSocket");
        },
        Some("subscribe") => {
            if let Some(topic) = message.get("topic").and_then(|v| v.as_str()) {
                debug!("Client subscribed to topic: {}", topic);
            }
        },
        _ => {
            warn!("Unknown WebSocket message type: {:?}", msg_type);
        }
    }
}