use axum::{Json, Router, extract::Path, routing::{get, post}};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::config::Config;
use tower_http::services::ServeDir;
use crate::server::http::{serve_index, serve_sw, serve_web_asset, serve_logo_asset, serve_icon_asset};

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub history: Option<Vec<ChatMessage>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub response: String,
    pub success: bool,
}

#[derive(Debug, Deserialize)]
pub struct TaskRequest {
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub version: String,
    pub mode: String,
    pub uptime_secs: u64,
}

pub fn create_router(config: &Config) -> Router {
    let start_time = std::time::Instant::now();
    let mode = if config.offline_mode { "offline" } else { "online" };

    Router::new()
        .route("/", get(serve_index))
        .route("/api/status", get(move || {
            let uptime = start_time.elapsed().as_secs();
            let mode = mode;
            async move {
                Json(StatusResponse {
                    status: "running".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    mode: mode.to_string(),
                    uptime_secs: uptime,
                })
            }
        }))
        .route("/api/agent/chat", post(chat_handler))
        .route("/api/agent/swarm", post(swarm_handler))
        .route("/api/task", post(task_handler))
        .route("/api/files", get(list_files))
        .route("/api/files/{*path}", get(get_file))
        .route("/api/diff", get(get_diff))
        .route("/api/branch", get(get_branch))
        .route("/api/reviews", get(list_reviews))
        .route("/api/incidents", get(list_incidents))
        .route("/api/dependencies", get(list_dependencies))
        .route("/api/decisions", get(list_decisions))
        .route("/api/skills", get(list_skills))
        .route("/api/config", get(get_config).post(update_config))
        .route("/api/cortex/search", post(search_handler))
        .route("/api/cortex/search", get(search_get_handler))
        .route("/api/compliance/log", get(compliance_log_handler))
        .route("/api/docs", get(docs_handler))
        .route("/api/heal", post(heal_handler))
        .route("/api/release", post(release_handler))
        .route("/assets/logo/{*path}", get(serve_logo_asset))
        .route("/assets/icon/{*path}", get(serve_icon_asset))
        .route("/assets/css/{*path}", get(serve_web_asset))
        .route("/assets/js/{*path}", get(serve_web_asset))
        .route("/sw.js", get(serve_sw))
        .nest_service("/assets", ServeDir::new("src/web_ui"))
}

pub async fn chat_handler(Json(req): Json<ChatRequest>) -> Json<ChatResponse> {
    Json(ChatResponse {
        response: format!("Echo: {}", req.message),
        success: true,
    })
}

pub async fn task_handler(Json(req): Json<TaskRequest>) -> Json<ChatResponse> {
    Json(ChatResponse {
        response: format!("Task received: {}", req.description),
        success: true,
    })
}

pub async fn swarm_handler(Json(req): Json<TaskRequest>) -> Json<ChatResponse> {
    Json(ChatResponse {
        response: format!("Swarm task received: {}", req.description),
        success: true,
    })
}

pub async fn list_files() -> Json<Vec<String>> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(".") {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if !name.starts_with('.') {
                    files.push(name.to_string());
                }
            }
        }
    }
    Json(files)
}

pub async fn get_file(Path(path): Path<String>) -> Json<HashMap<String, String>> {
    let mut result = HashMap::new();
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            result.insert("content".to_string(), content);
            result.insert("error".to_string(), String::new());
        }
        Err(e) => {
            result.insert("content".to_string(), String::new());
            result.insert("error".to_string(), e.to_string());
        }
    }
    Json(result)
}

pub async fn get_diff() -> Json<String> {
    let diff = std::process::Command::new("git")
        .args(["diff"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    Json(diff)
}

pub async fn get_branch() -> Json<String> {
    let branch = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "main".to_string());
    Json(branch)
}

pub async fn list_reviews() -> Json<Vec<HashMap<String, String>>> {
    Json(Vec::new())
}

pub async fn list_incidents() -> Json<Vec<HashMap<String, String>>> {
    Json(Vec::new())
}

pub async fn list_dependencies() -> Json<Vec<HashMap<String, String>>> {
    Json(Vec::new())
}

pub async fn list_decisions() -> Json<Vec<HashMap<String, String>>> {
    Json(Vec::new())
}

pub async fn list_skills() -> Json<Vec<HashMap<String, String>>> {
    Json(Vec::new())
}

pub async fn get_config() -> Json<HashMap<String, String>> {
    let mut config = HashMap::new();
    config.insert("status".to_string(), "ok".to_string());
    Json(config)
}

pub async fn update_config(Json(body): Json<HashMap<String, String>>) -> Json<HashMap<String, String>> {
    let mut result = HashMap::new();
    result.insert("status".to_string(), "updated".to_string());
    Json(result)
}

pub async fn search_handler(Json(req): Json<SearchRequest>) -> Json<Vec<HashMap<String, String>>> {
    let results = Vec::new();
    Json(results)
}

pub async fn search_get_handler(axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>) -> Json<Vec<HashMap<String, String>>> {
    let results = Vec::new();
    Json(results)
}

pub async fn compliance_log_handler() -> Json<Vec<HashMap<String, String>>> {
    Json(Vec::new())
}

pub async fn docs_handler() -> Json<String> {
    Json("# Documentation\n\nGenerated documentation for OmniCode.".to_string())
}

pub async fn heal_handler() -> Json<ChatResponse> {
    Json(ChatResponse {
        response: "Self-healing initiated".to_string(),
        success: true,
    })
}

pub async fn release_handler(Json(req): Json<TaskRequest>) -> Json<ChatResponse> {
    Json(ChatResponse {
        response: format!("Release pipeline started for: {}", req.description),
        success: true,
    })
}
