//! Embedded Vue production assets served by the binary.

use axum::{
    body::Body,
    http::{StatusCode, header, uri::Uri},
    response::{IntoResponse, Response},
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "../frontend/dist/"]
struct WebAssets;

/// Serve a static file from the embedded bundle, or SPA fallback to `index.html`.
pub async fn serve_embedded(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() {
        return serve_file("index.html");
    }

    if let Some(file) = WebAssets::get(path) {
        return file_response(path, file.data.as_ref());
    }

    if path.starts_with("api/") {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }

    // SPA client-side routes
    serve_file("index.html")
}

fn serve_file(name: &str) -> Response {
    match WebAssets::get(name) {
        Some(file) => file_response(name, file.data.as_ref()),
        None => (
            StatusCode::NOT_FOUND,
            format!("embedded asset missing: {name}"),
        )
            .into_response(),
    }
}

fn file_response(path: &str, data: &[u8]) -> Response {
    let mime = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime)
        .header(header::CACHE_CONTROL, cache_control(path))
        .body(Body::from(data.to_vec()))
        .unwrap()
}

fn cache_control(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "no-cache"
    } else if path.contains("/assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=3600"
    }
}
