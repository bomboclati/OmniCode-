pub mod archeologist;
pub mod guardian;
pub mod heal;
pub mod historian;
pub mod incident;
pub mod llm_client;
pub mod loop;
pub mod onboarding;
pub mod planner;
pub mod release;
pub mod review;
pub mod sandbox;
pub mod sentinel;
pub mod swarm;
pub mod tools;

use crate::config::Config;
use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum AgentState {
    Idle,
    Planning,
    Executing,
    WaitingForApproval,
    Error,
    Completed,
}

#[derive(Debug, Clone)]
pub struct AgentResult {
    pub summary: String,
    pub files_changed: Vec<String>,
    pub diff: String,
    pub success: bool,
}

pub async fn run_single_task(config: &Config, task: &str) -> Result<()> {
    println!("OmniCode Agent - Processing task: {}", task);

    let profile = config.get_active_profile()?;
    println!("Using provider: {} with model: {}", profile.provider, profile.model);

    let mut llm = llm_client::LlmClient::new(profile);
    let sandbox = sandbox::Sandbox::create(std::env::current_dir()?, &config.sandbox_mode).await?;
    let cortex = cortex::Cortex::new(std::env::current_dir()?)?;
    let tools = tools::get_available_tools();

    let mut agent_loop = loop::AgentLoop::new(llm, sandbox, cortex, tools);
    let result = agent_loop.run(task).await?;

    println!("\n=== Task Complete ===");
    println!("Summary: {}", result.summary);
    println!("Files changed: {}", result.files_changed.len());

    if !result.diff.is_empty() {
        println!("\nDiff:\n{}", result.diff);
    }

    Ok(())
}
