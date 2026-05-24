mod agent;
mod cli;
mod collaboration;
mod compliance;
mod config;
mod cortex;
mod crypto;
mod deploy;
mod docs_gen;
mod personalization;
mod desktop;
mod server;
mod skills;
mod tui;
mod web_assets;

use clap::Parser;
use cli::{Cli, Commands, SentinelCommands, SkillCommands};
use config::Config;
use tracing_subscriber::EnvFilter;

#[cfg(target_os = "windows")]
fn enable_ansi_support() {
    use windows::Win32::System::Console::*;
    unsafe {
        if let Ok(handle) = GetStdHandle(STD_OUTPUT_HANDLE) {
            let mut mode = CONSOLE_MODE::default();
            if GetConsoleMode(handle, &mut mode).is_ok() {
                let _ = SetConsoleMode(handle, CONSOLE_MODE(mode.0 | 0x0004));
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    #[cfg(target_os = "windows")]
    enable_ansi_support();

    let cli = Cli::parse();
    let config = Config::load().unwrap_or_default();

    match cli.command {
        None => {
            desktop::run(std::sync::Arc::new(config), 9420).await?;
        }
        Some(Commands::Omni) => {
            tui::run(config).await?;
        }
        Some(Commands::Serve { port }) => {
            server::start(config, port).await?;
        }
        Some(Commands::Desktop { port }) => {
            desktop::run(std::sync::Arc::new(config), port).await?;
        }
        Some(Commands::Task { description }) => {
            agent::run_single_task(&config, &description).await?;
        }
        Some(Commands::Swarm { task }) => {
            agent::swarm::run_swarm(&config, &task).await?;
        }
        Some(Commands::Sentinel(sub)) => match sub {
            SentinelCommands::Watch => {
                agent::sentinel::watch_ci(&config).await?;
            }
            SentinelCommands::ProdWatch => {
                agent::incident::watch_production(&config).await?;
            }
        },
        Some(Commands::Skill(sub)) => match sub {
            SkillCommands::Install { name } => {
                skills::install_skill(&config, &name).await?;
            }
            SkillCommands::Publish { path } => {
                skills::publish_skill(&config, &path).await?;
            }
        },
        Some(Commands::Review) => {
            agent::review::install_webhook(&config).await?;
        }
        Some(Commands::Heal) => {
            agent::heal::run_self_healing(&config).await?;
        }
        Some(Commands::Release { description }) => {
            agent::release::run_release_pipeline(&config, &description).await?;
        }
        Some(Commands::Archeologist {
            migrate,
            target_lang,
        }) => {
            agent::archeologist::run_migration(&config, &migrate, &target_lang).await?;
        }
        Some(Commands::Onboard) => {
            agent::onboarding::run_onboarding(&config).await?;
        }
        Some(Commands::Guardian) => {
            agent::guardian::run_guardian_daemon(&config).await?;
        }
        Some(Commands::Why { query }) => {
            agent::historian::query_decisions(&config, &query).await?;
        }
        Some(Commands::Find { query }) => {
            cortex::search::semantic_search(&config, &query).await?;
        }
        Some(Commands::Share) => {
            collaboration::create_session(&config).await?;
        }
        Some(Commands::Join { code }) => {
            collaboration::join_session(&config, &code).await?;
        }
        Some(Commands::Config) => {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
            let config_path = config::Config::config_path();
            let status = std::process::Command::new(editor)
                .arg(&config_path)
                .status()?;
            if !status.success() {
                println!("Failed to open config in editor");
            }
        }
        Some(Commands::Docs) => {
            docs_gen::generate_all_docs(&config).await?;
        }
        Some(Commands::Deploy) => {
            deploy::run_deployment(&config).await?;
        }
    }

    Ok(())
}
