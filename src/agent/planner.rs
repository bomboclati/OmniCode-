use crate::agent::llm_client::LlmClient;
use crate::cortex::Cortex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Complexity {
    Simple,
    Medium,
    Complex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub description: String,
    pub tool_to_use: String,
    pub expected_outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub steps: Vec<PlanStep>,
    pub estimated_complexity: Complexity,
}

impl Plan {
    pub fn new(steps: Vec<PlanStep>, complexity: Complexity) -> Self {
        Self {
            steps,
            estimated_complexity: complexity,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

pub async fn generate_plan(task: &str, cortex: &Cortex, llm: &mut LlmClient) -> Plan {
    let context = cortex.search(task).await.unwrap_or_default();

    let prompt = format!(
        r#"Break down the following task into a step-by-step plan.
Each step should specify which tool to use and the expected outcome.

Task: {}

Available tools: execute_command, read_file, write_file, search_codebase, grep_codebase, git_diff, git_commit, git_create_pr, git_branch, list_files, run_tests, read_lints

Context from codebase:
{}

Respond with a JSON object containing:
- "steps": array of objects with "description", "tool_to_use", "expected_outcome"
- "complexity": one of "Simple", "Medium", "Complex"

Return ONLY valid JSON, no markdown formatting."#,
        task, context
    );

    match llm.chat(&prompt, &[]).await {
        Ok(response) => {
            if let Ok(plan) = serde_json::from_str::<Plan>(&response) {
                plan
            } else {
                Plan {
                    steps: vec![PlanStep {
                        description: task.to_string(),
                        tool_to_use: "execute_command".to_string(),
                        expected_outcome: "Task completed".to_string(),
                    }],
                    estimated_complexity: Complexity::Medium,
                }
            }
        }
        Err(_) => Plan {
            steps: vec![PlanStep {
                description: task.to_string(),
                tool_to_use: "execute_command".to_string(),
                expected_outcome: "Task completed".to_string(),
            }],
            estimated_complexity: Complexity::Medium,
        },
    }
}
