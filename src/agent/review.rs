use crate::agent::llm_client::LlmClient;
use crate::config::Config;
use anyhow::Result;
use std::collections::HashMap;

pub struct ReviewConfig {
    pub repo_url: String,
    pub github_token: Option<String>,
    pub review_rules: Vec<String>,
    pub auto_approve: bool,
    pub min_approvals: u32,
}

pub struct PrReview {
    pub pr_number: u64,
    pub title: String,
    pub author: String,
    pub diff: String,
    pub comments: Vec<ReviewComment>,
    pub status: ReviewStatus,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReviewStatus {
    Pending,
    Approved,
    ChangesRequested,
    Dismissed,
}

pub struct ReviewComment {
    pub path: String,
    pub line: u64,
    pub body: String,
    pub severity: String,
}

pub async fn install_webhook(config: &Config) -> Result<()> {
    let webhook_url = format!("http://localhost:{}/api/webhook/review", config.web_port);

    println!("Installing PR review webhook...");
    println!("Webhook URL: {}", webhook_url);

    let output = std::process::Command::new("gh")
        .args([
            "api",
            "/repos/:owner/:repo/hooks",
            "--method", "POST",
            "--field", &format!("config[url]={}", webhook_url),
            "--field", "config[content_type]=json",
            "--field", "events[]=pull_request",
            "--field", "active=true",
        ])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            println!("Webhook installed successfully.");
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            if stderr.contains("already exists") {
                println!("Webhook already exists.");
            } else {
                println!("Note: Could not auto-install webhook via gh CLI.");
                println!("Please create a webhook manually pointing to: {}", webhook_url);
                println!("Events: Pull request");
            }
        }
        Err(_) => {
            println!("gh CLI not available. Please install GitHub CLI or create webhook manually.");
            println!("Webhook URL: {}", webhook_url);
        }
    }

    println!("PR review automation configured.");
    Ok(())
}

pub async fn review_pr(
    config: &Config,
    pr_number: u64,
    repo: &str,
    sender: Option<tokio::sync::mpsc::UnboundedSender<String>>,
) -> Result<PrReview> {
    println!("Reviewing PR #{} from repo '{}'...", pr_number, repo);

    let diff = fetch_pr_diff(repo, pr_number).await?;
    let files = parse_diff_files(&diff);

    let profile = config.get_active_profile()?;

    if let Some(ref tx) = sender {
        let _ = tx.send(format!("Reviewing {} files in PR #{}", files.len(), pr_number));
    }

    let mut llm = LlmClient::new(profile);
    let mut all_comments = Vec::new();

    for (path, file_diff) in &files {
        if let Some(ref tx) = sender {
            let _ = tx.send(format!("Reviewing: {}", path));
        }

        let prompt = format!(
            r#"Review the following code diff for potential issues.
Focus on: bugs, performance problems, security vulnerabilities, style violations, and logic errors.

File: {}
Diff:
```diff
{}
```

For each issue found, specify:
- The line number
- Severity (critical, warning, suggestion)
- Description of the issue

Return as JSON array of objects with fields: line, severity, message"#,
            path, file_diff
        );

        match llm.chat(&prompt, &[]).await {
            Ok(response) => {
                if let Ok(issues) = serde_json::from_str::<Vec<serde_json::Value>>(&response) {
                    for issue in issues {
                        let line = issue["line"].as_u64().unwrap_or(1);
                        let severity = issue["severity"].as_str().unwrap_or("suggestion");
                        let message = issue["message"].as_str().unwrap_or("No details");

                        all_comments.push(ReviewComment {
                            path: path.clone(),
                            line,
                            body: message.to_string(),
                            severity: severity.to_string(),
                        });
                    }
                }
            }
            Err(e) => {
                tracing::warn!("LLM review failed for {}: {}", path, e);
            }
        }
    }

    let summary = format!(
        "Reviewed PR #{}: {} files, {} comments",
        pr_number,
        files.len(),
        all_comments.len()
    );

    if let Some(ref tx) = sender {
        let _ = tx.send(format!("ReviewComplete: {}", summary));
    }

    Ok(PrReview {
        pr_number,
        title: format!("PR #{}", pr_number),
        author: "unknown".to_string(),
        diff,
        comments: all_comments,
        status: ReviewStatus::Pending,
        summary,
    })
}

async fn fetch_pr_diff(repo: &str, pr_number: u64) -> Result<String> {
    let api_url = format!("https://api.github.com/repos/{}/pulls/{}", repo, pr_number);

    let client = reqwest::Client::new();
    let response = client
        .get(&api_url)
        .header("Accept", "application/vnd.github.v3.diff")
        .header("User-Agent", "omnicode-review")
        .send()
        .await?;

    let diff = response.text().await?;
    Ok(diff)
}

fn parse_diff_files(diff: &str) -> HashMap<String, String> {
    let mut files = HashMap::new();
    let mut current_file = String::new();
    let mut current_diff = String::new();

    for line in diff.lines() {
        if line.starts_with("diff --git") {
            if !current_file.is_empty() {
                files.insert(current_file.clone(), current_diff.clone());
            }
            current_diff.clear();

            if let Some(path) = line.split_whitespace().nth(2) {
                current_file = path.trim_start_matches("a/").to_string();
            }
        }
        current_diff.push_str(line);
        current_diff.push('\n');
    }

    if !current_file.is_empty() {
        files.insert(current_file, current_diff);
    }

    files
}

pub async fn post_comments(comments: &[ReviewComment], repo: &str, pr_number: u64) -> Result<()> {
    for comment in comments {
        let api_url = format!(
            "https://api.github.com/repos/{}/pulls/{}/comments",
            repo, pr_number
        );

        let body = serde_json::json!({
            "body": comment.body,
            "path": comment.path,
            "line": comment.line,
        });

        let client = reqwest::Client::new();
        let response = client
            .post(&api_url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "omnicode-review")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            tracing::warn!("Failed to post comment: {}", response.status());
        }
    }

    Ok(())
}
