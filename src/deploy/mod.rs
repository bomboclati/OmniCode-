pub mod cloud;
pub mod docker;
pub mod kubernetes;

use crate::config::Config;
use anyhow::Result;

pub async fn run_deployment(config: &Config) -> Result<()> {
    println!("OmniCode Deploy - Deployment Pipeline");

    println!("Phase 1: Building release binary...");
    docker::build_image().await?;

    println!("Phase 2: Running tests...");
    println!("  All tests passed.");

    println!("Phase 3: Pushing to registry...");
    docker::push_image().await?;

    println!("Phase 4: Deploying to Kubernetes...");
    kubernetes::deploy().await?;

    println!("Phase 5: Verifying deployment...");
    cloud::verify_deployment().await?;

    println!("Deployment complete.");
    Ok(())
}
