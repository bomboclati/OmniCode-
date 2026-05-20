use axum::{
    body::Body,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};

pub async fn serve_index() -> Response<Body> {
    let html = include_str!("../web_ui/index.html");
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(html.to_string()))
        .unwrap()
}

pub async fn serve_web_asset(axum::extract::Path(path): axum::extract::Path<String>) -> Response<Body> {
    let asset_path = format!("src/web_ui/{}", path);
    match std::fs::read(&asset_path) {
        Ok(content) => {
            let mime = mime_for_path(&path);
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime)
                .header(header::CACHE_CONTROL, "public, max-age=3600")
                .body(Body::from(content))
                .unwrap()
        }
        Err(_) => not_found(&path),
    }
}

pub async fn serve_logo_asset(axum::extract::Path(path): axum::extract::Path<String>) -> Response<Body> {
    let asset_path = format!("assets/logo/{}", path);
    match std::fs::read(&asset_path) {
        Ok(content) => {
            let mime = mime_for_path(&path);
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime)
                .header(header::CACHE_CONTROL, "public, max-age=86400")
                .header(header::CONTENT_DISPOSITION, format!("inline; filename=\"{}\"", path.rsplit('/').next().unwrap_or(&path)))
                .body(Body::from(content))
                .unwrap()
        }
        Err(_) => not_found(&path),
    }
}

pub async fn serve_icon_asset(axum::extract::Path(path): axum::extract::Path<String>) -> Response<Body> {
    let asset_path = format!("assets/icon/{}", path);
    match std::fs::read(&asset_path) {
        Ok(content) => {
            let mime = mime_for_path(&path);
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime)
                .header(header::CACHE_CONTROL, "public, max-age=86400")
                .body(Body::from(content))
                .unwrap()
        }
        Err(_) => not_found(&path),
    }
}

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

fn not_found(path: &str) -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from(format!("404 - Not found: {}", path)))
        .unwrap()
}
