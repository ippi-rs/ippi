use axum::{
    extract::State,
    response::Json,
};
use crate::web::AppState;

pub async fn get_config(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "web": {
            "host": state.config.web.host,
            "port": state.config.web.port,
        },
        "kvm": state.config.kvm.as_ref().map(|k| serde_json::json!({
            "enabled": k.enabled,
        })),
        "p2p": state.config.p2p.as_ref().map(|p| serde_json::json!({
            "enabled": p.enabled,
        })),
        "webrtc": state.config.webrtc.as_ref().map(|w| serde_json::json!({
            "enabled": w.enabled,
        })),
        "cloud_init": state.config.cloud_init.as_ref().map(|c| serde_json::json!({
            "enabled": c.enabled,
        })),
        "version": crate::VERSION,
    }))
}