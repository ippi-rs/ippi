use crate::web::AppState;
use axum::{Router, routing::get};

pub mod config;
pub mod p2p;
pub mod kvm;

pub fn router() -> Router<AppState> {
    let router = Router::new()
        .route("/config", get(config::get_config));
    
    #[cfg(feature = "p2p")]
    let router = router.nest("/p2p", p2p::router());
    
    #[cfg(feature = "kvm")]
    let router = router.nest("/kvm", kvm::router());
    
    router
}
