use crate::agent::llm_client::LlmClient;
use crate::agent::sandbox::Sandbox;
use crate::agent::tools;
use crate::config::Config;
use crate::cortex::Cortex;
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

pub struct ArcheologistReport {
    pub total_files: usize,
    pub total_lines: usize,
    pub languages: HashMap<String, usize>,
    pub dead_code_candidates: Vec<String>,
    pub vulnerabilities: Vec<String>,
    pub bottlenecks: Vec<String>,
    pub architecture: String,
    pub diagram: String,
}

pub async fn analyze_legacy(project_root: &Path, config: &Config) -> Result<ArcheologistReport> {
    println!("Analyzing legacy codebase in '{}'...", project_root.display());

    let profile = config.get_active_profile()?;
    let mut llm = LlmClient::new(profile);

    let mut total_files = 0;
    let mut total_lines = 0;
    let mut languages = HashMap::new();
    let mut all_code = String::new();

    for entry in WalkDir::new(project_root)
        .into_iter()
        .filter_entry(|e| !is_ignored(e))
    {
        let entry = entry?;
        if entry.file_type().is_file() {
            total_files += 1;
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                let line_count = content.lines().count();
                total_lines += line_count;

                let ext = entry.path()
                    .extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                *languages.entry(ext).or_insert(0) += 1;

                if all_code.len() < 50000 {
                    all_code.push_str(&format!(
                        "\n--- {} ---\n{}\n",
                        entry.path().display(),
                        content
                    ));
                }
            }
        }
    }

    let prompt = format!(
        r#"Analyze this codebase architecture and identify:
1. Dead code candidates (unused functions, modules, imports)
2. Security vulnerabilities (hardcoded secrets, unsafe code, injection risks)
3. Performance bottlenecks (inefficient algorithms, N+1 queries, blocking calls)
4. Overall architecture description
5. Mermaid architecture diagram

Codebase (first 50k chars):
{}

Return JSON with fields:
- "dead_code": array of strings
- "vulnerabilities": array of strings
- "bottlenecks": array of strings
- "architecture": string description
- "diagram": mermaid graph code"#,
        all_code
    );

    let mut dead_code = Vec::new();
    let mut vulnerabilities = Vec::new();
    let mut bottlenecks = Vec::new();
    let mut architecture = String::new();
    let mut diagram = String::new();

    if let Ok(response) = llm.chat(&prompt, &[]).await {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
            dead_code = json["dead_code"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            vulnerabilities = json["vulnerabilities"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            bottlenecks = json["bottlenecks"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            architecture = json["architecture"].as_str().unwrap_or("").to_string();
            diagram = json["diagram"].as_str().unwrap_or("").to_string();
        }
    }

    // Generate codebase bible
    let bible_path = project_root.join("docs").join("codebase-bible");
    std::fs::create_dir_all(&bible_path)?;

    let bible_content = format!(
        r#"# Codebase Bible

## Overview
- Total files: {}
- Total lines: {}
- Languages: {:?}

## Architecture
{}

## Diagram
```mermaid
{}
```

## Areas of Concern
### Dead Code Candidates
{}

### Vulnerabilities
{}

### Bottlenecks
{}
"#,
        total_files, total_lines, languages,
        architecture,
        diagram,
        dead_code.iter().map(|d| format!("- {}\n", d)).collect::<String>(),
        vulnerabilities.iter().map(|v| format!("- {}\n", v)).collect::<String>(),
        bottlenecks.iter().map(|b| format!("- {}\n", b)).collect::<String>(),
    );

    std::fs::write(bible_path.join("README.md"), &bible_content)?;
    println!("Codebase bible generated at: {:?}", bible_path);

    Ok(ArcheologistReport {
        total_files,
        total_lines,
        languages,
        dead_code_candidates: dead_code,
        vulnerabilities,
        bottlenecks,
        architecture,
        diagram,
    })
}

pub async fn migrate_codebase(
    source_lang: &str,
    target_lang: &str,
    project_root: &Path,
    config: &Config,
) -> Result<MigrationResult> {
    println!("Migrating from {} to {}...", source_lang, target_lang);

    let profile = config.get_active_profile()?;
    let mut llm = LlmClient::new(profile);

    let mut source_files = Vec::new();
    let ext_map = get_language_extension(source_lang);

    for entry in WalkDir::new(project_root).into_iter().filter_entry(|e| !is_ignored(e)) {
        let entry = entry?;
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext_map.contains(&ext.to_string_lossy().to_string()) {
                    source_files.push(entry.path().to_path_buf());
                }
            }
        }
    }

    println!("Found {} source files to migrate.", source_files.len());

    let mut migrated = 0;
    let mut errors = Vec::new();
    let sandbox = Sandbox::create(project_root.to_path_buf(), &config.sandbox_mode).await?;

    for file_path in &source_files {
        let content = std::fs::read_to_string(file_path)?;
        let relative = file_path.strip_prefix(project_root).unwrap_or(file_path);

        println!("  Migrating: {}", relative.display());

        let prompt = format!(
            r#"Translate this {} code to {}.
Preserve all functionality, comments, and logic.
Output ONLY the translated code, no explanation.

Source file: {}
Language: {} -> {}

Code:
```{}
{}
```"#,
            source_lang, target_lang,
            relative.display(),
            source_lang, target_lang,
            source_lang, content
        );

        match llm.chat(&prompt, &[]).await {
            Ok(translated) => {
                let new_ext = get_target_extension(target_lang);
                let new_path = file_path.with_extension(&new_ext);

                if let Err(e) = std::fs::write(&new_path, &translated) {
                    errors.push(format!("{}: {}", relative.display(), e));
                } else {
                    migrated += 1;
                }
            }
            Err(e) => {
                errors.push(format!("{}: LLM error - {}", relative.display(), e));
            }
        }
    }

    Ok(MigrationResult {
        total_files: source_files.len(),
        migrated,
        skipped: source_files.len() - migrated - errors.len(),
        errors,
    })
}

pub async fn run_migration(config: &Config, source_lang: &str, target_lang: &str) -> Result<()> {
    let project_root = std::env::current_dir()?;
    let result = migrate_codebase(source_lang, target_lang, &project_root, config).await?;

    println!("\nMigration complete:");
    println!("  Total files: {}", result.total_files);
    println!("  Migrated: {}", result.migrated);
    println!("  Skipped: {}", result.skipped);
    println!("  Errors: {}", result.errors.len());

    for e in &result.errors {
        println!("  - {}", e);
    }

    Ok(())
}

pub struct MigrationResult {
    pub total_files: usize,
    pub migrated: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

fn is_ignored(entry: &walkdir::DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    name.starts_with('.') || name == "target" || name == "node_modules" || name == "vendor"
}

fn get_language_extension(lang: &str) -> Vec<String> {
    match lang.to_lowercase().as_str() {
        "rust" | "rs" => vec!["rs".to_string()],
        "python" | "py" => vec!["py".to_string()],
        "typescript" | "ts" => vec!["ts".to_string(), "tsx".to_string()],
        "javascript" | "js" => vec!["js".to_string(), "jsx".to_string()],
        "go" => vec!["go".to_string()],
        "java" => vec!["java".to_string()],
        "c" => vec!["c".to_string(), "h".to_string()],
        "cpp" | "c++" => vec!["cpp".to_string(), "hpp".to_string(), "cc".to_string()],
        "ruby" | "rb" => vec!["rb".to_string()],
        "php" => vec!["php".to_string()],
        "csharp" | "cs" => vec!["cs".to_string()],
        _ => vec![lang.to_string()],
    }
}

fn get_target_extension(lang: &str) -> String {
    match lang.to_lowercase().as_str() {
        "rust" | "rs" => "rs".to_string(),
        "python" | "py" => "py".to_string(),
        "typescript" | "ts" => "ts".to_string(),
        "javascript" | "js" => "js".to_string(),
        "go" => "go".to_string(),
        "java" => "java".to_string(),
        "c" => "c".to_string(),
        "cpp" | "c++" => "cpp".to_string(),
        "ruby" | "rb" => "rb".to_string(),
        "php" => "php".to_string(),
        "csharp" | "cs" => "cs".to_string(),
        _ => lang.to_string(),
    }
}
