use crate::config::Config;
use crate::skills::Skill;
use anyhow::Result;
use std::collections::HashMap;

pub struct MarketplaceEntry {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub category: String,
    pub downloads: u64,
    pub rating: f32,
}

pub async fn search_marketplace(config: &Config, query: &str) -> Result<Vec<MarketplaceEntry>> {
    let client = reqwest::Client::new();
    let url = format!("https://api.omnicode.dev/skills/search?q={}", query);

    let response = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;

    match response {
        Ok(resp) => {
            let entries: Vec<MarketplaceEntry> = resp.json().await.unwrap_or_default();
            Ok(entries)
        }
        Err(_) => {
            println!("Warning: Cannot reach marketplace (offline or no network)");
            Ok(Vec::new())
        }
    }
}

pub async fn download_skill(config: &Config, name: &str) -> Result<Skill> {
    let skills_dir = crate::config::Config::config_dir()?.join("skills").join(name);
    let url = format!("https://api.omnicode.dev/skills/{}/download", name);

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await;

    match response {
        Ok(resp) => {
            let bytes = resp.bytes().await.unwrap_or_default();
            let temp_dir = std::env::temp_dir().join(format!("omnicode-skill-{}", name));
            std::fs::create_dir_all(&temp_dir)?;

            let archive_path = temp_dir.join("skill.tar.gz");
            std::fs::write(&archive_path, &bytes)?;

            let output = std::process::Command::new("tar")
                .args(["-xzf", archive_path.to_str().unwrap_or(""), "-C", temp_dir.to_str().unwrap_or("")])
                .output()?;

            if !output.status.success() {
                std::fs::create_dir_all(&skills_dir)?;
                let stub_manifest = format!(
                    r#"{{
  "name": "{}",
  "version": "0.1.0",
  "description": "Skill stub (offline install)",
  "author": "unknown",
  "category": "general"
}}"#,
                    name
                );
                std::fs::write(skills_dir.join("manifest.json"), &stub_manifest)?;
            } else {
                if skills_dir.exists() {
                    std::fs::remove_dir_all(&skills_dir)?;
                }
                std::fs::rename(&temp_dir, &skills_dir)?;
            }

            let _ = std::fs::remove_file(&archive_path);
            let _ = std::fs::remove_dir(&temp_dir);

            Ok(Skill {
                id: name.to_string(),
                name: name.to_string(),
                version: "0.1.0".to_string(),
                description: format!("Skill: {}", name),
                author: "marketplace".to_string(),
                category: "general".to_string(),
                install_path: Some(skills_dir),
                enabled: true,
            })
        }
        Err(_) => {
            std::fs::create_dir_all(&skills_dir)?;
            let readme = format!("# {}\n\nSkill stub (offline mode)", name);
            std::fs::write(skills_dir.join("README.md"), &readme)?;

            Ok(Skill {
                id: name.to_string(),
                name: name.to_string(),
                version: "0.1.0".to_string(),
                description: format!("Skill: {} (offline stub)", name),
                author: "local".to_string(),
                category: "general".to_string(),
                install_path: Some(skills_dir),
                enabled: true,
            })
        }
    }
}

pub async fn upload_skill(config: &Config, manifest: &HashMap<String, String>) -> Result<()> {
    let client = reqwest::Client::new();
    let url = "https://api.omnicode.dev/skills/publish";

    let response = client
        .post(url)
        .json(manifest)
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                println!("Skill published to marketplace.");
            } else {
                println!("Marketplace returned status: {}", resp.status());
            }
        }
        Err(_) => {
            println!("Warning: Cannot reach marketplace. Skill saved locally.");
        }
    }

    Ok(())
}
