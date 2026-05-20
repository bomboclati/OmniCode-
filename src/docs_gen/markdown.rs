use anyhow::Result;
use std::path::Path;

pub fn generate_readme() -> Result<String> {
    let project_name = get_project_name();
    let description = get_cargo_field("description").unwrap_or_else(|| "A Rust project".to_string());
    let version = get_cargo_field("version").unwrap_or_else(|| "0.1.0".to_string());

    let mut readme = String::new();
    readme.push_str(&format!("# {}\n\n", project_name));
    readme.push_str(&format!("{}\n\n", description));
    readme.push_str("## Getting Started\n\n");
    readme.push_str("### Prerequisites\n\n");
    readme.push_str("- Rust 2021 edition\n");
    readme.push_str("- Cargo\n\n");
    readme.push_str("### Installation\n\n");
    readme.push_str("```bash\n");
    readme.push_str(&format!("cargo install {}\n", project_name));
    readme.push_str("```\n\n");
    readme.push_str("## Usage\n\n");
    readme.push_str("```bash\n");
    readme.push_str(&format!("{} [OPTIONS] <COMMAND>\n", project_name));
    readme.push_str("```\n\n");
    readme.push_str("## Commands\n\n");
    readme.push_str("- `omni` - Launch the TUI\n");
    readme.push_str("- `serve` - Start the web server\n");
    readme.push_str("- `<task>` - Run a single agent task\n");
    readme.push_str("- `swarm <task>` - Run swarm mode\n");
    readme.push_str("- `sentinel watch` - CI/CD sentinel\n");
    readme.push_str("- `review` - PR review automation\n");
    readme.push_str("- `heal` - Self-healing tests\n");
    readme.push_str("- `release <desc>` - Prompt-to-binary\n");
    readme.push_str("- `onboard` - Onboarding wizard\n");
    readme.push_str("- `guardian` - Dependency guardian\n");
    readme.push_str("- `why <query>` - Decision historian\n");
    readme.push_str("- `find <query>` - Code search\n");
    readme.push_str("- `docs` - Regenerate documentation\n");
    readme.push_str("- `deploy` - Deployment pipeline\n\n");
    readme.push_str("## Configuration\n\n");
    readme.push_str(&format!("Configuration is stored at `~/.omnicode/config.toml` (or `%APPDATA%/omnicode/config.toml` on Windows).\n\n"));
    readme.push_str("## Version\n\n");
    readme.push_str(&format!("Current version: v{}\n", version));
    readme.push_str("\n## License\n\nMIT\n");

    Ok(readme)
}

pub fn generate_api_docs() -> Result<String> {
    let mut docs = String::new();
    docs.push_str("# API Documentation\n\n");
    docs.push_str("## Web Server\n\n");
    docs.push_str("The OmniCode web server runs on `localhost:9420` by default.\n\n");
    docs.push_str("### Endpoints\n\n");
    docs.push_str("| Method | Path | Description |\n");
    docs.push_str("|--------|------|-------------|\n");
    docs.push_str("| GET | `/` | Serve web UI |\n");
    docs.push_str("| GET | `/assets/*` | Static assets |\n");
    docs.push_str("| GET | `/ws` | WebSocket connection |\n");
    docs.push_str("| GET | `/api/status` | Server status |\n");
    docs.push_str("| POST | `/api/agent/chat` | Send chat message |\n");
    docs.push_str("| POST | `/api/agent/swarm` | Start swarm task |\n");
    docs.push_str("| GET | `/api/cortex/search` | Semantic search |\n");
    docs.push_str("| GET | `/api/files` | List files |\n");
    docs.push_str("| GET | `/api/files/{path}` | Read file |\n");
    docs.push_str("| GET | `/api/diff` | Get git diff |\n");
    docs.push_str("| GET | `/api/branch` | Get current branch |\n\n");
    docs.push_str("## WebSocket Protocol\n\n");
    docs.push_str("Messages are JSON with fields: `type`, `content`, `timestamp`.\n\n");
    docs.push_str("### Message Types\n\n");
    docs.push_str("- `chat_message` - Chat with agent\n");
    docs.push_str("- `file_change` - File modification\n");
    docs.push_str("- `cursor_update` - Collaboration cursor\n");
    docs.push_str("- `agent_status` - Agent status update\n");
    docs.push_str("- `voice_transcript` - Voice input\n");

    Ok(docs)
}

fn get_project_name() -> String {
    if let Ok(content) = std::fs::read_to_string("Cargo.toml") {
        for line in content.lines() {
            if let Some(name) = line.strip_prefix("name = ") {
                return name.trim_matches('"').to_string();
            }
        }
    }
    Path::new(".").file_name().unwrap_or_default().to_string_lossy().to_string()
}

fn get_cargo_field(field: &str) -> Option<String> {
    if let Ok(content) = std::fs::read_to_string("Cargo.toml") {
        for line in content.lines() {
            if let Some(val) = line.strip_prefix(&format!("{} = ", field)) {
                return Some(val.trim_matches('"').to_string());
            }
        }
    }
    None
}
