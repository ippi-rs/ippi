use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[cfg(feature = "frontend-embedded")]
use axum::http::header;

#[cfg(feature = "frontend-embedded")]
use rust_embed::RustEmbed;

#[cfg(feature = "frontend-embedded")]
#[derive(RustEmbed)]
#[folder = "dist/"]
struct Asset;

pub async fn serve_static(_uri: axum::http::Uri) -> Response {
    #[cfg(feature = "frontend-embedded")]
    {
        let mut path = _uri.path().trim_start_matches('/').to_string();
        if path.is_empty() {
            path = "index.html".to_string();
        }
        match Asset::get(&path) {
            Some(content) => {
                let mime = mime_guess::from_path(&path).first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
            }
            None => match Asset::get("index.html") {
                Some(content) => {
                    ([(header::CONTENT_TYPE, "text/html")], content.data).into_response()
                }
                None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
            },
        }
    }
    #[cfg(not(feature = "frontend-embedded"))]
    {
        (StatusCode::NOT_FOUND, "Frontend not embedded").into_response()
    }
}
