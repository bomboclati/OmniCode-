use anyhow::Result;

pub async fn build_image() -> Result<()> {
    println!("  Building Docker image...");
    let output = std::process::Command::new("docker")
        .args(["build", "-t", "omnicode:latest", "."])
        .output();

    match output {
        Ok(o) if o.status.success() => Ok(()),
        Ok(o) => Err(anyhow::anyhow!(
            "Docker build failed: {}",
            String::from_utf8_lossy(&o.stderr)
        )),
        Err(e) => Err(anyhow::anyhow!("Docker not available: {}", e)),
    }
}

pub async fn push_image() -> Result<()> {
    println!("  Pushing image to registry...");
    Ok(())
}
