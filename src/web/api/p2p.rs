use crate::web::AppState;
use axum::{extract::State, response::Json, routing::get, Router};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/peers", get(get_peers))
        .route("/status", get(get_status))
}

async fn get_peers(
    State(_state): State<AppState>,
) -> Json<serde_json::Value> {
    #[cfg(feature = "p2p")]
    {
        if let Some(manager) = &state.p2p_manager {
            match manager.get_peer_count().await {
                Ok(count) => {
                    return Json(serde_json::json!({ "count": count }));
                }
                Err(e) => {
                    tracing::warn!("Failed to get peer count: {}", e);
                }
            }
        }
    }
    
    Json(serde_json::json!({ "count": 0 }))
}

async fn get_status(
    State(_state): State<AppState>,
) -> Json<serde_json::Value> {
    #[cfg(feature = "p2p")]
    {
        if let Some(manager) = &state.p2p_manager {
            match manager.get_stats().await {
                Ok(stats) => {
                    return Json(serde_json::json!({
                        "enabled": stats.enabled,
                        "listening": stats.listening,
                        "peer_id": stats.peer_id,
                        "bootstrap_connected": stats.bootstrap_connected,
                        "listen_addrs": stats.listen_addrs,
                    }));
                }
                Err(e) => {
                    tracing::warn!("Failed to get P2P stats: {}", e);
                }
            }
        }
    }
    
    Json(serde_json::json!({
        "enabled": false,
        "listening": false,
        "peer_id": null,
        "bootstrap_connected": false,
        "listen_addrs": [],
    }))
}