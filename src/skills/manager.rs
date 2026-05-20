use crate::skills::Skill;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

pub fn install_skill_files(skill: &Skill) -> Result<()> {
    let skills_dir = crate::config::Config::config_dir()?.join("skills");
    std::fs::create_dir_all(&skills_dir)?;
    println!("Skill '{}' files installed.", skill.name);
    Ok(())
}

pub fn uninstall_skill(name: &str) -> Result<()> {
    let skills_dir = crate::config::Config::config_dir()?.join("skills").join(name);
    if skills_dir.exists() {
        std::fs::remove_dir_all(&skills_dir)?;
        println!("Skill '{}' uninstalled.", name);
    }
    Ok(())
}

pub fn list_installed() -> Result<Vec<Skill>> {
    let skills_dir = crate::config::Config::config_dir()?.join("skills");
    let mut skills = Vec::new();

    if !skills_dir.exists() {
        return Ok(skills);
    }

    for entry in std::fs::read_dir(&skills_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            let manifest_path = entry.path().join("manifest.json");
            let (version, description, author, category) = if manifest_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                    if let Ok(manifest) = serde_json::from_str::<HashMap<String, String>>(&content) {
                        (
                            manifest.get("version").cloned().unwrap_or_else(|| "0.1.0".to_string()),
                            manifest.get("description").cloned().unwrap_or_default(),
                            manifest.get("author").cloned().unwrap_or_else(|| "unknown".to_string()),
                            manifest.get("category").cloned().unwrap_or_else(|| "general".to_string()),
                        )
                    } else {
                        ("0.1.0".to_string(), String::new(), "unknown".to_string(), "general".to_string())
                    }
                } else {
                    ("0.1.0".to_string(), String::new(), "unknown".to_string(), "general".to_string())
                }
            } else {
                ("0.1.0".to_string(), String::new(), "unknown".to_string(), "general".to_string())
            };

            skills.push(Skill {
                id: name.clone(),
                name,
                version,
                description,
                author,
                category,
                install_path: Some(entry.path()),
                enabled: true,
            });
        }
    }

    Ok(skills)
}

pub fn load_skill_manifest(path: &str) -> Result<HashMap<String, String>> {
    let manifest_path = Path::new(path).join("manifest.json");
    let content = std::fs::read_to_string(&manifest_path)?;
    let manifest: HashMap<String, String> = serde_json::from_str(&content)?;
    Ok(manifest)
}

pub fn run_skill_hook(skill_name: &str, hook: &str) -> Result<()> {
    let skills_dir = crate::config::Config::config_dir()?.join("skills").join(skill_name);
    let hook_path = skills_dir.join(hook);

    if hook_path.exists() {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        let _ = std::fs::set_permissions(&hook_path, perms);
        let output = std::process::Command::new(hook_path.to_str().unwrap_or("")).output()?;
        if !output.status.success() {
            eprintln!("Skill hook '{}' failed: {}", hook, String::from_utf8_lossy(&output.stderr));
        } else {
            println!("Skill hook '{}' executed: {}", hook, String::from_utf8_lossy(&output.stdout));
        }
    }

    Ok(())
}
