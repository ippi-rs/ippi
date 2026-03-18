use crate::{Config, Result};
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub mod api;
pub mod assets;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
}

pub async fn serve(config: Config) -> Result<()> {
    let app = create_app(config.clone()).await?;
    let addr = format!("{}:{}", config.web.host, config.web.port);
    tracing::info!("Starting web server on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await
        .map_err(|e| crate::Error::Web(format!("Failed to bind to {}: {}", addr, e)))?;
    axum::serve(listener, app.into_make_service()).await
        .map_err(|e| crate::Error::Web(format!("Failed to serve: {}", e)))?;
    Ok(())
}

#[cfg(test)]
pub async fn serve_test_app(config: Config) -> Result<Router> {
    Ok(create_app(config).await?)
}

async fn create_app(config: Config) -> Result<Router> {
    let state = AppState {
        config: Arc::new(config.clone()),
    };
    let cors = if config.web.cors_origins.iter().any(|s| s == "*") {
        CorsLayer::new().allow_origin(tower_http::cors::Any)
    } else {
        CorsLayer::new().allow_origin(
            config.web.cors_origins
                .iter()
                .map(|s| s.parse().unwrap())
                .collect::<Vec<_>>()
        )
    };
    let cors = cors
        .allow_methods(vec![axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers(vec![axum::http::header::CONTENT_TYPE]);

    Ok(Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .nest("/api", api::router())
        .fallback(axum::routing::get(assets::serve_static))
        .layer(cors)
        .with_state(state))
}

async fn root() -> Html<&'static str> {
    #[cfg(feature = "frontend-embedded")]
    {
        Html(include_str!("../../dist/index.html"))
    }
    #[cfg(not(feature = "frontend-embedded"))]
    {
        Html(r#"<!DOCTYPE html><html><body>
        <h1>IPPI</h1>
        <p>Backend running - Frontend not embedded</p>
        </body></html>"#)
    }
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({
        "status": "ok",
        "version": crate::VERSION,
        "name": crate::NAME,
    })))
}
