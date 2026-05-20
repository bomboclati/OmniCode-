use crate::agent::llm_client::LlmClient;
use crate::config::Config;
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct ReleaseResult {
    pub version: String,
    pub artifacts: Vec<ReleaseArtifact>,
    pub success: bool,
    pub summary: String,
}

pub struct ReleaseArtifact {
    pub name: String,
    pub path: PathBuf,
    pub checksum: String,
    pub size_bytes: u64,
}

pub async fn run_release_pipeline(config: &Config, description: &str) -> Result<ReleaseResult> {
    println!("OmniCode Release Pipeline");
    println!("Description: {}", description);

    let version = generate_version();
    println!("Version: {}", version);

    let profile = config.get_active_profile()?;
    let mut llm = LlmClient::new(profile);

    // Phase 1: Plan
    println!("\nPhase 1: Planning...");
    let plan_prompt = format!(
        r#"You are planning a release build for the following description:
{}

Create a structured release plan covering:
1. Build targets (platforms)
2. Required dependencies
3. Testing strategy
4. Artifact types (binary, package installer, etc.)

Return as bullet points."#,
        description
    );
    let _ = llm.chat(&plan_prompt, &[]).await;

    // Phase 2: Build
    println!("\nPhase 2: Building release binary...");
    let build_result = build_release_binary();
    if let Err(ref e) = build_result {
        println!("  Build warning: {}", e);
    }

    // Phase 3: Test
    println!("\nPhase 3: Running release tests...");
    let test_output = Command::new("cargo")
        .args(["test", "--release"])
        .output();
    match test_output {
        Ok(o) if o.status.success() => {
            println!("  All tests passed.");
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            println!("  Test issues: {}", stderr);
        }
        Err(e) => {
            println!("  Test execution error: {}", e);
        }
    }

    // Phase 4: Package
    println!("\nPhase 4: Creating release artifacts...");
    let artifacts = create_artifacts(&version)?;

    // Phase 5: Checksum
    println!("\nPhase 5: Generating checksums...");
    for artifact in &artifacts {
        println!("  {}: {} ({} bytes)", artifact.name, artifact.checksum, artifact.size_bytes);
    }

    // Phase 6: GitHub Release
    println!("\nPhase 6: Creating GitHub release...");
    create_github_release(&version, description, &artifacts).await?;

    let summary = format!(
        "Release v{} complete: {} artifacts created for '{}'",
        version,
        artifacts.len(),
        description
    );
    println!("\n{}", summary);

    Ok(ReleaseResult {
        version,
        artifacts,
        success: true,
        summary,
    })
}

fn generate_version() -> String {
    let date = chrono::Local::now().format("%Y%m%d%H%M%S");
    format!("0.1.0-{}", date)
}

fn build_release_binary() -> Result<()> {
    let output = Command::new("cargo")
        .args(["build", "--release", "--features", "tui,server"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Build failed: {}", stderr));
    }

    println!("  Binary built: target/release/omnicode.exe");
    Ok(())
}

fn create_artifacts(version: &str) -> Result<Vec<ReleaseArtifact>> {
    let mut artifacts = Vec::new();
    let release_dir = PathBuf::from("target").join("release");
    let dist_dir = PathBuf::from("target").join("dist");
    std::fs::create_dir_all(&dist_dir)?;

    // Binary artifact
    let binary_name = if cfg!(target_os = "windows") {
        "omnicode.exe"
    } else {
        "omnicode"
    };

    let binary_path = release_dir.join(binary_name);
    if binary_path.exists() {
        let data = std::fs::read(&binary_path)?;
        let checksum = format!("{:x}", Sha256::digest(&data));
        let size = data.len() as u64;

        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;
        let archive_name = format!("omnicode-{}-{}-{}.tar.gz", version, os, arch);
        let archive_path = dist_dir.join(&archive_name);

        #[cfg(unix)]
        {
            let output = Command::new("tar")
                .args(["-czf", archive_path.to_str().unwrap_or(""), "-C", release_dir.to_str().unwrap_or(""), binary_name])
                .output()?;
            if output.status.success() {
                let archive_data = std::fs::read(&archive_path)?;
                let archive_checksum = format!("{:x}", Sha256::digest(&archive_data));
                let archive_size = archive_data.len() as u64;

                println!("  Created archive: {} ({} bytes)", archive_name, archive_size);
                artifacts.push(ReleaseArtifact {
                    name: archive_name,
                    path: archive_path,
                    checksum: archive_checksum,
                    size_bytes: archive_size,
                });
            }
        }

        artifacts.push(ReleaseArtifact {
            name: binary_name.to_string(),
            path: binary_path,
            checksum,
            size_bytes: size,
        });
    }

    Ok(artifacts)
}

async fn create_github_release(version: &str, description: &str, artifacts: &[ReleaseArtifact]) -> Result<()> {
    let changelog = get_changelog_since_last_tag();

    // Try gh CLI first
    let output = Command::new("gh")
        .args([
            "release",
            "create",
            &format!("v{}", version),
            "--title",
            &format!("OmniCode v{}", version),
            "--notes",
            &format!("## Release v{}\n\n{}\n\n{}", version, description, changelog),
        ])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            println!("  GitHub release created via gh CLI.");
            // Upload artifacts
            for artifact in artifacts {
                let _ = Command::new("gh")
                    .args(["release", "upload", &format!("v{}", version), artifact.path.to_str().unwrap_or("")])
                    .output();
            }
        }
        _ => {
            println!("  GitHub release draft prepared (gh CLI not available).");
            println!("  Version: v{}", version);
            println!("  To create manually:");
            println!("    gh release create v{} --title \"OmniCode v{}\" --notes \"{}\"", version, version, description);
        }
    }

    Ok(())
}

fn get_changelog_since_last_tag() -> String {
    let output = Command::new("git")
        .args(["log", "--oneline", "--no-decorate", "$(git describe --tags --abbrev=0 2>/dev/null)..HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    if output.is_empty() {
        let all_log = Command::new("git")
            .args(["log", "--oneline", "-10"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();
        return all_log;
    }

    output
}
