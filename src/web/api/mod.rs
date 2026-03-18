use crate::web::AppState;
use axum::{Router, routing::get};

pub mod config;

pub fn router() -> Router<AppState> {
    Router::new().route("/config", get(config::get_config))
}
