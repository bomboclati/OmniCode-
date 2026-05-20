use crate::agent::llm_client::LlmClient;
use crate::config::Config;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

pub struct GuardianState {
    pub is_running: bool,
    pub dependencies: HashMap<String, DependencyInfo>,
    pub updates_available: Vec<DependencyUpdate>,
}

#[derive(Clone)]
pub struct DependencyInfo {
    pub package: String,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub ecosystem: String,
}

pub struct DependencyUpdate {
    pub package: String,
    pub current_version: String,
    pub new_version: String,
    pub breaking_change: bool,
    pub security_advisory: Option<String>,
    pub changelog_summary: String,
    pub affected_files: Vec<String>,
}

pub async fn run_guardian_daemon(config: &Config) -> Result<()> {
    println!("OmniCode Dependency Guardian - Daemon Mode");
    println!("Monitoring dependencies for updates and vulnerabilities...\n");

    let project_root = std::env::current_dir()?;
    let profile = config.get_active_profile()?;
    let mut llm = LlmClient::new(profile);

    let mut state = parse_dependencies(&project_root);
    println!("Found {} dependencies:", state.dependencies.len());
    for (name, info) in &state.dependencies {
        println!("  {} @ {} ({})", name, info.current_version, info.ecosystem);
    }

    let poll_interval = std::time::Duration::from_secs(3600); // Check every hour
    let mut iteration = 0;

    loop {
        iteration += 1;
        println!("\n[Guardian] Check #{} - {}", iteration, chrono::Local::now().format("%H:%M:%S"));

        let mut any_updates = false;

        for (name, info) in &state.dependencies.clone() {
            match check_for_updates(name, &info.current_version, &info.ecosystem).await {
                Ok(Some(update)) => {
                    any_updates = true;
                    println!("\n  ⚠️  Update available: {} {} → {}", name, update.current_version, update.new_version);

                    if update.breaking_change {
                        println!("      Breaking change detected!");

                        // Try auto-fix in sandbox
                        println!("      Attempting auto-upgrade...");

                        let upgrade_prompt = format!(
                            r#"Create a migration plan for upgrading {} from {} to {}.
This is a breaking change. Provide:
1. What changed in the API
2. Migration steps
3. Required code changes

Use this format:
- Concise bullet points
- Include code examples where relevant"#,
                            name, update.current_version, update.new_version
                        );

                        if let Ok(plan) = llm.chat(&upgrade_prompt, &[]).await {
                            println!("      Upgrade plan:\n{}", plan);
                        }
                    }

                    state.updates_available.push(update);
                }
                Ok(None) => {} // Up to date
                Err(e) => {
                    tracing::warn!("  Error checking {}: {}", name, e);
                }
            }
        }

        if !any_updates {
            println!("  All dependencies up to date.");
        }

        tokio::time::sleep(poll_interval).await;
    }
}

fn parse_dependencies(project_root: &Path) -> GuardianState {
    let mut state = GuardianState {
        is_running: true,
        dependencies: HashMap::new(),
        updates_available: Vec::new(),
    };

    // Check Cargo.toml
    let cargo_path = project_root.join("Cargo.toml");
    if cargo_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&cargo_path) {
            if let Ok(config) = content.parse::<toml::Value>() {
                if let Some(deps) = config.get("dependencies").and_then(|d| d.as_table()) {
                    for (name, value) in deps {
                        let version = if let Some(t) = value.as_str() {
                            t.to_string()
                        } else if let Some(t) = value.get("version").and_then(|v| v.as_str()) {
                            t.to_string()
                        } else {
                            "unknown".to_string()
                        };

                        state.dependencies.insert(name.clone(), DependencyInfo {
                            package: name.clone(),
                            current_version: version,
                            latest_version: None,
                            ecosystem: "cargo".to_string(),
                        });
                    }
                }
            }
        }
    }

    // Check package.json
    let pkg_path = project_root.join("package.json");
    if pkg_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&pkg_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
                    for (name, value) in deps {
                        let version = value.as_str().unwrap_or("unknown").to_string();
                        state.dependencies.insert(name.clone(), DependencyInfo {
                            package: name.clone(),
                            current_version: version,
                            latest_version: None,
                            ecosystem: "npm".to_string(),
                        });
                    }
                }
            }
        }
    }

    state
}

pub async fn check_for_updates(
    package: &str,
    current_version: &str,
    ecosystem: &str,
) -> Result<Option<DependencyUpdate>> {
    let latest_version = match ecosystem {
        "cargo" => check_crates_io(package).await?,
        "npm" => check_npm_registry(package).await?,
        _ => return Ok(None),
    };

    if let Some(latest) = latest_version {
        if latest != current_version {
            return Ok(Some(DependencyUpdate {
                package: package.to_string(),
                current_version: current_version.to_string(),
                new_version: latest.clone(),
                breaking_change: is_major_change(current_version, &latest),
                security_advisory: None,
                changelog_summary: String::new(),
                affected_files: find_affected_files(package),
            }));
        }
    }

    Ok(None)
}

async fn check_crates_io(package: &str) -> Result<Option<String>> {
    let url = format!("https://crates.io/api/v1/crates/{}", package);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "omnicode-guardian")
        .send()
        .await?;

    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        if let Some(version) = json["crate"]["max_stable_version"].as_str() {
            return Ok(Some(version.to_string()));
        }
    }

    Ok(None)
}

async fn check_npm_registry(package: &str) -> Result<Option<String>> {
    let url = format!("https://registry.npmjs.org/{}", package);
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;

    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        if let Some(version) = json["dist-tags"]["latest"].as_str() {
            return Ok(Some(version.to_string()));
        }
    }

    Ok(None)
}

fn is_major_change(current: &str, latest: &str) -> bool {
    let current_major = current.split('.').next().unwrap_or("0").parse::<u32>().unwrap_or(0);
    let latest_major = latest.split('.').next().unwrap_or("0").parse::<u32>().unwrap_or(0);
    latest_major > current_major
}

fn find_affected_files(package: &str) -> Vec<String> {
    let mut files = Vec::new();

    // Search for imports/usages of the package in the codebase
    if let Ok(output) = Command::new("grep")
        .args(["-rn", "--include=*.rs", "--include=*.ts", "--include=*.js", &format!("use {}|from '{}'|\"{}\"", package, package, package)])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some(file) = line.split(':').next() {
                if !files.contains(&file.to_string()) {
                    files.push(file.to_string());
                }
            }
        }
    }

    files
}
