use axum::{
    body::Body,
    http::{header, StatusCode},
    response::Response,
};
use rust_embed::RustEmbed;

fn mime_for_path(path: &str) -> &'static str {
    if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".js") {
        "application/javascript"
    } else if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".html") {
        "text/html"
    } else if path.ends_with(".wasm") {
        "application/wasm"
    } else {
        "application/octet-stream"
    }
}

pub async fn serve_index() -> Response<Body> {
    serve_embedded("index.html", "text/html; charset=utf-8")
}

pub async fn serve_sw() -> Response<Body> {
    serve_embedded("sw.js", "application/javascript")
}

pub async fn serve_asset(axum::extract::Path(path): axum::extract::Path<String>) -> Response<Body> {
    let mime = mime_for_path(&path);
    let embedded_path = format!("assets/{}", path);
    match crate::web_assets::WebAssets::get(&embedded_path) {
        Some(content) => {
            let data = content.data.to_vec();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime)
                .header(header::CACHE_CONTROL, "public, max-age=3600")
                .body(Body::from(data))
                .unwrap()
        }
        None => {
            serve_embedded(&path, mime)
        }
    }
}

fn serve_embedded(path: &str, content_type: &str) -> Response<Body> {
    match crate::web_assets::WebAssets::get(path) {
        Some(content) => {
            let data = content.data.to_vec();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::CACHE_CONTROL, "public, max-age=3600")
                .body(Body::from(data))
                .unwrap()
        }
        None => not_found(path),
    }
}

fn not_found(path: &str) -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from(format!("404 - Not found: {}", path)))
        .unwrap()
}
