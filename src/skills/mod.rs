pub mod manager;
pub mod marketplace;

use crate::config::Config;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub category: String,
    pub install_path: Option<std::path::PathBuf>,
    pub enabled: bool,
}

pub async fn install_skill(config: &Config, name: &str) -> Result<()> {
    let skill = marketplace::download_skill(config, name).await?;
    manager::install_skill_files(&skill)?;
    println!("Skill '{}' installed successfully.", name);
    Ok(())
}

pub async fn publish_skill(config: &Config, path: &str) -> Result<()> {
    let manifest = manager::load_skill_manifest(path)?;
    marketplace::upload_skill(config, &manifest).await?;
    println!("Skill published from '{}'.", path);
    Ok(())
}
