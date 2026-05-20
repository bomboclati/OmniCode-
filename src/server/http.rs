use axum::response::Html;

pub async fn serve_index() -> Html<String> {
    let html = include_str!("../web_ui/index.html");
    Html(html.to_string())
}

pub async fn serve_asset(axum::extract::Path(path): axum::extract::Path<String>) -> Html<String> {
    let asset_path = format!("src/web_ui/{}", path);
    match std::fs::read_to_string(&asset_path) {
        Ok(content) => Html(content),
        Err(_) => Html(format!("<h1>404</h1><p>Asset not found: {}</p>", path)),
    }
}
