use crate::agent::llm_client::LlmClient;
use crate::agent::tools::{self, ToolResult};
use crate::config::Config;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct SwarmAgent {
    pub name: String,
    pub role: String,
    pub model: String,
    pub status: String,
    pub current_task: String,
    pub tokens_used: u64,
    pub output: String,
}

pub struct SwarmResult {
    pub summary: String,
    pub agent_results: Vec<SwarmAgentResult>,
    pub conflicts_resolved: usize,
    pub success: bool,
}

pub struct SwarmAgentResult {
    pub agent_name: String,
    pub role: String,
    pub output: String,
    pub tokens_used: u64,
    pub success: bool,
}

pub async fn run_swarm(config: &Config, task: &str) -> Result<()> {
    println!("OmniCode Swarm Mode");
    println!("Task: {}\n", task);

    let profile = config.get_active_profile()?;

    let agents = vec![
        SwarmAgent {
            name: "coordinator".to_string(),
            role: "coordinator".to_string(),
            model: profile.model.clone(),
            status: "planning".to_string(),
            current_task: task.to_string(),
            tokens_used: 0,
            output: String::new(),
        },
        SwarmAgent {
            name: "architect".to_string(),
            role: "design".to_string(),
            model: profile.model.clone(),
            status: "waiting".to_string(),
            current_task: format!("Design architecture for: {}", task),
            tokens_used: 0,
            output: String::new(),
        },
        SwarmAgent {
            name: "coder".to_string(),
            role: "implementation".to_string(),
            model: profile.model.clone(),
            status: "waiting".to_string(),
            current_task: format!("Implement solution for: {}", task),
            tokens_used: 0,
            output: String::new(),
        },
        SwarmAgent {
            name: "reviewer".to_string(),
            role: "review".to_string(),
            model: profile.model.clone(),
            status: "waiting".to_string(),
            current_task: String::new(),
            tokens_used: 0,
            output: String::new(),
        },
        SwarmAgent {
            name: "tester".to_string(),
            role: "testing".to_string(),
            model: profile.model.clone(),
            status: "waiting".to_string(),
            current_task: String::new(),
            tokens_used: 0,
            output: String::new(),
        },
    ];

    let results = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    for agent in agents {
        let profile = config.get_active_profile()?;
        let results_clone = results.clone();
        let task_clone = agent.current_task.clone();
        let name_clone = agent.name.clone();
        let role_clone = agent.role.clone();

        let handle = tokio::spawn(async move {
            let mut llm = LlmClient::new(profile);

            let prompt = format!(
                r#"You are a {} agent in a swarm, role: {}.
Your task: {}

Work on this task and provide your output.
Be thorough and specific."#,
                name_clone, role_clone, task_clone
            );

            let start = std::time::Instant::now();

            match llm.chat(&prompt, &[]).await {
                Ok(output) => {
                    let tokens = (output.len() / 4) as u64; // rough estimate

                    let mut r = results_clone.lock().await;
                    r.push(SwarmAgentResult {
                        agent_name: name_clone,
                        role: role_clone,
                        output,
                        tokens_used: tokens,
                        success: true,
                    });
                }
                Err(e) => {
                    let mut r = results_clone.lock().await;
                    r.push(SwarmAgentResult {
                        agent_name: name_clone,
                        role: role_clone,
                        output: format!("Error: {}", e),
                        tokens_used: 0,
                        success: false,
                    });
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all agents
    for handle in handles {
        let _ = handle.await;
    }

    // Collect and display results
    let final_results = results.lock().await;
    let mut total_tokens = 0;

    println!("\n=== Swarm Results ===");
    for result in final_results.iter() {
        println!("\n--- {} ({}) ---", result.agent_name, result.role);
        if result.success {
            println!("Status: ✅ Success");
            println!("Tokens: {}", result.tokens_used);
            let preview: String = result.output.chars().take(300).collect();
            println!("Output:\n{}", preview);
            if result.output.len() > 300 {
                println!("... ({} more chars)", result.output.len() - 300);
            }
        } else {
            println!("Status: ❌ Failed");
            println!("Error: {}", result.output);
        }
        total_tokens += result.tokens_used;
    }

    println!("\n=== Summary ===");
    println!("Total agents: {}", final_results.len());
    println!("Successful: {}", final_results.iter().filter(|r| r.success).count());
    println!("Failed: {}", final_results.iter().filter(|r| !r.success).count());
    println!("Total tokens: {}", total_tokens);

    Ok(())
}

pub async fn run_swarm_with_sender(
    config: &Config,
    task: &str,
    sender: tokio::sync::mpsc::UnboundedSender<String>,
) -> Result<SwarmResult> {
    let _ = sender.send(format!("Swarm starting for task: {}", task));
    run_swarm(config, task).await?;
    let _ = sender.send("Swarm complete".to_string());

    Ok(SwarmResult {
        summary: format!("Swarm completed for: {}", task),
        agent_results: Vec::new(),
        conflicts_resolved: 0,
        success: true,
    })
}
