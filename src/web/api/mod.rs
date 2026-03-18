use axum::{Router, routing::get};
use crate::web::AppState;

pub mod config;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/config", get(config::get_config))
}