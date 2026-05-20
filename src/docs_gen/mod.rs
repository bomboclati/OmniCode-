pub mod markdown;
pub mod mermaid;

use crate::config::Config;
use anyhow::Result;

pub async fn generate_all_docs(config: &Config) -> Result<()> {
    let docs_dir = std::env::current_dir()?.join("docs");
    std::fs::create_dir_all(&docs_dir)?;

    println!("Generating project documentation...");

    let readme = markdown::generate_readme()?;
    std::fs::write(docs_dir.join("README.md"), &readme)?;

    let api_docs = markdown::generate_api_docs()?;
    std::fs::write(docs_dir.join("API.md"), &api_docs)?;

    let arch_diagram = mermaid::generate_architecture_diagram()?;
    std::fs::write(docs_dir.join("architecture.md"), &arch_diagram)?;

    let seq_diagram = mermaid::generate_sequence_diagram()?;
    std::fs::write(docs_dir.join("sequences.md"), &seq_diagram)?;

    let changelog = generate_changelog()?;
    std::fs::write(docs_dir.join("CHANGELOG.md"), &changelog)?;

    println!("Documentation generated in '{:?}'.", docs_dir);
    Ok(())
}

pub async fn regenerate_docs(config: &Config, section: &str) -> Result<()> {
    match section {
        "readme" => {
            let readme = markdown::generate_readme()?;
            std::fs::write("docs/README.md", &readme)?;
        }
        "api" => {
            let api_docs = markdown::generate_api_docs()?;
            std::fs::write("docs/API.md", &api_docs)?;
        }
        "diagrams" => {
            let arch = mermaid::generate_architecture_diagram()?;
            let seq = mermaid::generate_sequence_diagram()?;
            std::fs::write("docs/architecture.md", &arch)?;
            std::fs::write("docs/sequences.md", &seq)?;
        }
        _ => {
            println!("Unknown docs section: {}", section);
        }
    }
    Ok(())
}

fn generate_changelog() -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["log", "--oneline", "--decorate", "--all"])
        .output()?;

    let log = String::from_utf8_lossy(&output.stdout);
    let mut changelog = String::new();
    changelog.push_str("# Changelog\n\n");
    for line in log.lines().take(100) {
        changelog.push_str(&format!("- {}\n", line));
    }
    Ok(changelog)
}
