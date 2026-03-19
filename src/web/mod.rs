use crate::{Config, Result};
use axum::{
    Router,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::get,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[cfg(feature = "kvm")]
use crate::kvm::KvmManager;
#[cfg(feature = "p2p")]
use crate::p2p::P2pManager;
#[cfg(feature = "webrtc")]
use crate::webrtc::WebRtcManager;

pub mod api;
pub mod assets;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    #[cfg(feature = "kvm")]
    pub kvm_manager: Option<Arc<KvmManager>>,
    #[cfg(feature = "p2p")]
    pub p2p_manager: Option<Arc<P2pManager>>,
    #[cfg(feature = "webrtc")]
    pub webrtc_manager: Option<Arc<WebRtcManager>>,
}

pub async fn serve(config: Config) -> Result<()> {
    let app = create_app(config.clone()).await?;
    let addr = format!("{}:{}", config.web.host, config.web.port);
    tracing::info!("Starting web server on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| crate::Error::Web(format!("Failed to bind to {}: {}", addr, e)))?;
    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|e| crate::Error::Web(format!("Failed to serve: {}", e)))?;
    Ok(())
}

pub async fn serve_test_app(config: Config) -> Result<Router> {
    create_app(config).await
}

async fn create_app(config: Config) -> Result<Router> {
    let config_arc = Arc::new(config.clone());
    
    #[cfg(feature = "kvm")]
    let kvm_manager = if config.kvm.as_ref().map(|k| k.enabled).unwrap_or(false) {
        match KvmManager::new(config_arc.clone()) {
            Ok(manager) => {
                let manager = Arc::new(manager);
                if let Err(e) = manager.initialize().await {
                    tracing::warn!("Failed to initialize KVM manager: {}", e);
                    None
                } else {
                    Some(manager)
                }
            }
            Err(e) => {
                tracing::warn!("Failed to create KVM manager: {}", e);
                None
            }
        }
    } else {
        None
    };

    #[cfg(feature = "p2p")]
    let p2p_manager = if config.p2p.as_ref().map(|p| p.enabled).unwrap_or(false) {
        match P2pManager::new(config_arc.clone()) {
            Ok(manager) => {
                let manager = Arc::new(manager);
                if let Err(e) = manager.initialize().await {
                    tracing::warn!("Failed to initialize P2P manager: {}", e);
                    None
                } else {
                    Some(manager)
                }
            }
            Err(e) => {
                tracing::warn!("Failed to create P2P manager: {}", e);
                None
            }
        }
    } else {
        None
    };

    #[cfg(feature = "webrtc")]
    let webrtc_manager = if config.webrtc.as_ref().map(|w| w.enabled).unwrap_or(false) {
        match WebRtcManager::new(config_arc.clone()) {
            Ok(manager) => {
                let manager = Arc::new(manager);
                if let Err(e) = manager.initialize().await {
                    tracing::warn!("Failed to initialize WebRTC manager: {}", e);
                    None
                } else {
                    Some(manager)
                }
            }
            Err(e) => {
                tracing::warn!("Failed to create WebRTC manager: {}", e);
                None
            }
        }
    } else {
        None
    };

    let state = AppState {
        config: config_arc,
        #[cfg(feature = "kvm")]
        kvm_manager,
        #[cfg(feature = "p2p")]
        p2p_manager,
        #[cfg(feature = "webrtc")]
        webrtc_manager,
    };
    let cors = if config.web.cors_origins.iter().any(|s| s == "*") {
        CorsLayer::new().allow_origin(tower_http::cors::Any)
    } else {
        CorsLayer::new().allow_origin(
            config
                .web
                .cors_origins
                .iter()
                .map(|s| s.parse().unwrap())
                .collect::<Vec<_>>(),
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
        Html(
            r#"<!DOCTYPE html><html><body>
        <h1>IPPI</h1>
        <p>Backend running - Frontend not embedded</p>
        </body></html>"#,
        )
    }
}

async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "version": crate::VERSION,
            "name": crate::NAME,
        })),
    )
}
