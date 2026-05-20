use crate::agent::llm_client::LlmClient;
use crate::agent::tools;
use crate::config::Config;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

pub enum CiProvider {
    GitHubActions,
    Jenkins,
    CircleCI,
    Custom(String),
}

pub struct Sentinel {
    pub ci_provider: CiProvider,
    pub poll_interval: Duration,
    pub repo_owner: String,
    pub repo_name: String,
    pub github_token: Option<String>,
}

pub struct SentinelState {
    pub is_running: bool,
    pub last_check: Option<chrono::DateTime<chrono::Local>>,
    pub last_known_status: String,
    pub alerts: Vec<String>,
    pub active_branch: String,
}

impl Sentinel {
    pub fn new(config: &Config) -> Self {
        let repo_url = std::process::Command::new("git")
            .args(["config", "--get", "remote.origin.url"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let (owner, name) = Self::parse_repo_url(&repo_url);

        Self {
            ci_provider: CiProvider::GitHubActions,
            poll_interval: Duration::from_secs(30),
            repo_owner: owner,
            repo_name: name,
            github_token: None,
        }
    }

    pub async fn watch_ci(&self, sender: Option<tokio::sync::mpsc::UnboundedSender<String>>) -> Result<()> {
        let state = Arc::new(Mutex::new(SentinelState {
            is_running: true,
            last_check: None,
            last_known_status: "unknown".to_string(),
            alerts: Vec::new(),
            active_branch: get_current_branch(),
        }));

        println!("Sentinel watching CI/CD for {}/{}...", self.repo_owner, self.repo_name);

        let state_clone = state.clone();
        let sender_clone = sender.clone();

        let poll_task = tokio::spawn(async move {
            loop {
                let s = state_clone.lock().await;
                if !s.is_running {
                    break;
                }
                drop(s);

                tokio::time::sleep(self.poll_interval).await;

                match self.check_ci_status().await {
                    Ok(status) => {
                        let mut s = state_clone.lock().await;
                        s.last_check = Some(chrono::Local::now());

                        if status != s.last_known_status {
                            let msg = format!("CI status changed: {} -> {}", s.last_known_status, status);
                            s.alerts.push(msg.clone());
                            s.last_known_status = status.clone();

                            if let Some(ref tx) = sender_clone {
                                let _ = tx.send(format!("SentinelAlert: {}", msg));
                            }

                            if status == "failure" {
                                let _ = self.handle_failure(&s.active_branch, sender_clone.clone()).await;
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("CI check error: {}", e);
                    }
                }
            }
        });

        tokio::signal::ctrl_c().await?;
        let mut s = state.lock().await;
        s.is_running = false;

        poll_task.await?;
        println!("Sentinel stopped.");
        Ok(())
    }

    async fn check_ci_status(&self) -> Result<String> {
        let api_url = format!(
            "https://api.github.com/repos/{}/{}/actions/runs?branch={}&per_page=1",
            self.repo_owner, self.repo_name, get_current_branch()
        );

        let client = reqwest::Client::new();
        let mut req = client.get(&api_url)
            .header("User-Agent", "omnicode-sentinel")
            .header("Accept", "application/vnd.github.v3+json");

        if let Some(ref token) = self.github_token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        let response = req.send().await?;
        let json: serde_json::Value = response.json().await?;

        let status = json["workflow_runs"][0]["conclusion"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        Ok(status)
    }

    async fn handle_failure(&self, branch: &str, sender: Option<tokio::sync::mpsc::UnboundedSender<String>>) -> Result<()> {
        println!("CI failure detected on branch '{}'. Attempting auto-fix...", branch);

        let output = std::process::Command::new("git")
            .args(["checkout", branch])
            .output()?;

        if let Some(ref tx) = sender {
            let _ = tx.send("SentinelAlert: Auto-fix initiated".to_string());
        }

        let profile = config_for_llm().await;
        if let Ok(profile) = profile {
            let mut llm = LlmClient::new(profile);

            let test_output = std::process::Command::new("cargo")
                .args(["test", "2>&1"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_default();

            let prompt = format!(
                "A CI build failed. Here is the test output:\n{}\nPlease analyze and fix the issue.",
                test_output
            );

            match llm.chat(&prompt, &[]).await {
                Ok(fix) => {
                    println!("Suggested fix: {}", fix);
                    let _ = std::process::Command::new("git")
                        .args(["commit", "-am", "Auto-fix: CI failure resolution"])
                        .output();
                    let _ = std::process::Command::new("git")
                        .args(["push"])
                        .output();
                }
                Err(e) => {
                    tracing::error!("Failed to get LLM fix: {}", e);
                }
            }
        }

        Ok(())
    }

    fn parse_repo_url(url: &str) -> (String, String) {
        let url = url.trim().trim_end_matches(".git");
        if url.contains("github.com") {
            if let Some(path) = url.split("github.com").nth(1) {
                let parts: Vec<&str> = path.trim_start_matches(':').trim_start_matches('/').split('/').collect();
                if parts.len() >= 2 {
                    return (parts[0].to_string(), parts[1].to_string());
                }
            }
        }
        ("unknown".to_string(), "unknown".to_string())
    }
}

pub async fn watch_ci(config: &Config) -> Result<()> {
    let sentinel = Sentinel::new(config);
    sentinel.watch_ci(None).await
}

pub async fn watch_ci_with_sender(
    config: &Config,
    sender: tokio::sync::mpsc::UnboundedSender<String>,
) -> Result<()> {
    let sentinel = Sentinel::new(config);
    sentinel.watch_ci(Some(sender)).await
}

fn get_current_branch() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "main".to_string())
}

async fn config_for_llm() -> Result<crate::config::Profile> {
    let config_path = crate::config::Config::config_path();
    if config_path.exists() {
        if let Ok(cfg) = crate::config::Config::load() {
            if let Ok(profile) = cfg.get_active_profile() {
                return Ok(profile);
            }
        }
    }
    Err(anyhow::anyhow!("No config available"))
}
