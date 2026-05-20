use crate::agent::llm_client::LlmClient;
use crate::config::Config;
use anyhow::Result;
use std::collections::HashMap;
use std::time::Duration;

pub struct IncidentWhisperer {
    pub is_active: bool,
    pub monitored_services: Vec<String>,
    pub alert_sources: Vec<AlertSource>,
    pub alert_history: Vec<IncidentRecord>,
    pub config: Config,
}

pub enum AlertSource {
    Sentry,
    Datadog,
    Prometheus,
    Custom(String, String),
}

pub struct IncidentRecord {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub service: String,
    pub severity: String,
    pub message: String,
    pub stack_trace: String,
    pub correlated_commit: String,
    pub auto_fix_attempted: bool,
    pub fix_status: String,
    pub pr_link: String,
}

pub async fn watch_production(config: &Config) -> Result<()> {
    let mut whisperer = IncidentWhisperer {
        is_active: true,
        monitored_services: vec![
            "api".to_string(),
            "web".to_string(),
            "worker".to_string(),
            "database".to_string(),
        ],
        alert_sources: vec![
            AlertSource::Sentry,
            AlertSource::Custom("log_file".to_string(), "logs/app.log".to_string()),
        ],
        alert_history: Vec::new(),
        config: config.clone(),
    };

    println!("OmniCode Incident Whisperer - Production Watch");
    println!("Monitoring {} services...", whisperer.monitored_services.len());
    for service in &whisperer.monitored_services {
        println!("  - {}", service);
    }

    let profile = config.get_active_profile()?;
    let mut llm = LlmClient::new(profile);

    let poll_interval = Duration::from_secs(30);
    let mut iteration = 0;

    loop {
        tokio::time::sleep(poll_interval).await;
        iteration += 1;

        // Check log files for errors
        for source in &whisperer.alert_sources {
            if let AlertSource::Custom(_, path) = source {
                if let Ok(content) = std::fs::read_to_string(path) {
                    let recent_lines: Vec<&str> = content.lines().rev().take(50).collect();
                    for line in &recent_lines {
                        if line.contains("ERROR") || line.contains("FATAL") || line.contains("CRITICAL") || line.contains("panic") {
                            let id = uuid::Uuid::new_v4().to_string();
                            let now = chrono::Local::now();

                            let severity = if line.contains("FATAL") || line.contains("panic") {
                                "critical".to_string()
                            } else if line.contains("ERROR") {
                                "error".to_string()
                            } else {
                                "warning".to_string()
                            };

                            let correlated = find_correlated_commit(line);

                            println!("[{}] {} - {}", now.format("%H:%M:%S"), severity, line);

                            let record = IncidentRecord {
                                id,
                                timestamp: now,
                                service: path.to_string(),
                                severity,
                                message: line.to_string(),
                                stack_trace: String::new(),
                                correlated_commit: correlated.clone(),
                                auto_fix_attempted: false,
                                fix_status: "detected".to_string(),
                                pr_link: String::new(),
                            };

                            // Auto-fix
                            let fix_prompt = format!(
                                r#"A production incident was detected.
Error: {}
Correlated commit: {}

Analyze the root cause and suggest a fix. Include:
1. Root cause analysis
2. Fix implementation (code changes)
3. Testing strategy"#,
                                line, correlated
                            );

                            if let Ok(fix) = llm.chat(&fix_prompt, &[]).await {
                                println!("  Auto-fix analysis: {}", fix);

                                let branch_name = format!("hotfix/incident-{}", now.format("%Y%m%d%H%M%S"));
                                let _ = std::process::Command::new("git")
                                    .args(["checkout", "-b", &branch_name])
                                    .output();

                                let msg = format!("hotfix: {}", line);
                                let _ = std::process::Command::new("git")
                                    .args(["commit", "-am", &msg])
                                    .output();
                            }

                            whisperer.alert_history.push(record);
                        }
                    }
                }
            }
        }

        if iteration % 10 == 0 {
            println!("[Whisperer] Heartbeat - {} incidents tracked", whisperer.alert_history.len());
        }
    }
}

fn find_correlated_commit(error_line: &str) -> String {
    let output = std::process::Command::new("git")
        .args(["log", "--oneline", "-20", "--all"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    for line in output.lines() {
        if line.to_lowercase().contains("deploy") || line.to_lowercase().contains("release") {
            return line.to_string();
        }
    }

    output.lines().next().unwrap_or("unknown").to_string()
}

pub async fn analyze_incident(incident: &str) -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["log", "--oneline", "-5"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    Ok(format!(
        r#"Incident Analysis Report
=========================
Incident: {}

Recent Commits:
{}

Root Cause: Under investigation
Suggested Fix: Pending analysis
Impact: Requires immediate attention
"#,
        incident, output
    ))
}
