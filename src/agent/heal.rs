use crate::agent::llm_client::LlmClient;
use crate::config::Config;
use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub struct HealResult {
    pub total_failures: usize,
    pub fixed: usize,
    pub unfixable: Vec<String>,
    pub summary: String,
}

pub async fn run_self_healing(config: &Config) -> Result<()> {
    println!("Running self-healing on test failures...");
    let result = heal_tests(Path::new("."), config).await?;

    println!("\nSelf-healing complete:");
    println!("  Total failures: {}", result.total_failures);
    println!("  Fixed: {}", result.fixed);
    println!("  Unfixable: {}", result.unfixable.len());

    for u in &result.unfixable {
        println!("  - {}", u);
    }

    println!("\nSummary: {}", result.summary);
    Ok(())
}

pub async fn heal_tests(project_root: &Path, config: &Config) -> Result<HealResult> {
    let test_output = run_tests(project_root)?;
    let failures = parse_test_failures(&test_output);

    if failures.is_empty() {
        return Ok(HealResult {
            total_failures: 0,
            fixed: 0,
            unfixable: Vec::new(),
            summary: "All tests pass. No healing needed.".to_string(),
        });
    }

    println!("Found {} test failures.", failures.len());

    let profile = config.get_active_profile()?;
    let mut llm = LlmClient::new(profile);
    let mut fixed = 0;
    let mut unfixable = Vec::new();

    for (test_name, error_msg) in &failures {
        println!("Analyzing failure: {}", test_name);

        let source_context = find_test_source(project_root, test_name);

        let prompt = format!(
            r#"A test is failing. Analyze the failure and suggest a fix.

Test name: {}
Error: {}

Source context:
{}

Provide the fix as a complete replacement for the failing test or source code.
Return JSON with fields:
- "problem": brief description
- "fix_type": "test" or "source"
- "file_path": path to the file that needs fixing
- "fixed_code": the complete replacement code
- "explanation": why this fix works"#,
            test_name, error_msg, source_context
        );

        match llm.chat(&prompt, &[]).await {
            Ok(response) => {
                if let Ok(fix) = serde_json::from_str::<serde_json::Value>(&response) {
                    let file_path = fix["file_path"].as_str().unwrap_or("");
                    let fixed_code = fix["fixed_code"].as_str().unwrap_or("");

                    if !file_path.is_empty() && !fixed_code.is_empty() {
                        let full_path = project_root.join(file_path);
                        if let Err(e) = std::fs::write(&full_path, fixed_code) {
                            tracing::warn!("Failed to write fix: {}", e);
                            unfixable.push(format!("{} (write error)", test_name));
                        } else {
                            fixed += 1;
                            println!("  Fixed: {}", test_name);
                        }
                    } else {
                        unfixable.push(format!("{} (no fix generated)", test_name));
                    }
                } else {
                    unfixable.push(format!("{} (parse error)", test_name));
                }
            }
            Err(e) => {
                tracing::warn!("LLM error for {}: {}", test_name, e);
                unfixable.push(format!("{} (LLM error)", test_name));
            }
        }
    }

    if fixed > 0 {
        println!("Re-running tests after fixes...");
        let final_output = run_tests(project_root)?;
        let remaining = parse_test_failures(&final_output);

        if remaining.is_empty() {
            println!("All fixes verified!");
        } else {
            println!("{} failures remain after fixes.", remaining.len());
        }
    }

    Ok(HealResult {
        total_failures: failures.len(),
        fixed,
        unfixable,
        summary: format!("Fixed {}/{} failures", fixed, failures.len()),
    })
}

fn run_tests(project_root: &Path) -> Result<String> {
    let output = Command::new("cargo")
        .args(["test", "--no-fail-fast"])
        .current_dir(project_root)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(format!("{}\n{}", stdout, stderr))
}

fn parse_test_failures(test_output: &str) -> Vec<(String, String)> {
    let mut failures = Vec::new();
    let mut current_test = String::new();
    let mut current_error = String::new();
    let mut capturing = false;

    for line in test_output.lines() {
        if line.starts_with("----") && line.contains("FAILED") {
            if !current_test.is_empty() {
                failures.push((current_test.clone(), current_error.clone()));
            }
            current_test = line.to_string();
            current_error.clear();
            capturing = true;
        } else if capturing {
            if line.starts_with("----") || line.starts_with("test ") {
                failures.push((current_test.clone(), current_error.clone()));
                current_test = line.to_string();
                current_error.clear();
                capturing = true;
            } else {
                current_error.push_str(line);
                current_error.push('\n');
            }
        }
    }

    if !current_test.is_empty() {
        failures.push((current_test, current_error));
    }

    failures
}

fn find_test_source(project_root: &Path, test_name: &str) -> String {
    let test_name_clean = test_name
        .split_whitespace()
        .last()
        .unwrap_or(test_name)
        .trim();

    let output = Command::new("grep")
        .args([
            "-rn",
            &format!("#[test]"),
            "--include=*.rs",
            project_root.to_str().unwrap_or("."),
        ])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    output
}

pub async fn analyze_failure(test_output: &str) -> Result<String> {
    let mut analysis = String::new();
    let failures = parse_test_failures(test_output);

    for (test, error) in &failures {
        analysis.push_str(&format!("## Failure: {}\n", test));
        analysis.push_str(&format!("Error:\n```\n{}\n```\n\n", error));
    }

    if analysis.is_empty() {
        analysis = "No specific failures identified.".to_string();
    } else {
        analysis = format!("Found {} failure(s):\n\n{}", failures.len(), analysis);
    }

    Ok(analysis)
}
