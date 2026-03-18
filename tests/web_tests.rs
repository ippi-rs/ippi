use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use ippi::{Config, web};
use tower::ServiceExt;

#[tokio::test]
async fn test_health_endpoint() {
    let config = Config::default();
    let app = web::serve_test_app(config).await.unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), 1_000_000).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "ok");
    assert_eq!(json["name"], "ippi");
}

#[tokio::test]
async fn test_config_endpoint() {
    let config = Config::default();
    let app = web::serve_test_app(config).await.unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/config")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), 1_000_000).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["web"]["host"], "0.0.0.0");
    assert_eq!(json["web"]["port"], 8080);
    assert_eq!(json["version"], env!("CARGO_PKG_VERSION"));
}

#[tokio::test]
async fn test_root_endpoint() {
    let config = Config::default();
    let app = web::serve_test_app(config).await.unwrap();

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), 1_000_000).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);

    assert!(body_str.contains("<!DOCTYPE html>"));
}

#[tokio::test]
async fn test_not_found() {
    let config = Config::default();
    let app = web::serve_test_app(config).await.unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // SPA fallback serves index.html for unknown routes
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1_000_000).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("<!DOCTYPE html>"));
}
