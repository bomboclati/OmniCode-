use crate::agent::llm_client::{LlmClient, Message};
use crate::agent::planner::{self, Plan};
use crate::agent::sandbox::Sandbox;
use crate::agent::tools::{self, ToolDef, ToolResult};
use crate::cortex::Cortex;
use anyhow::Result;
use std::path::Path;

use crate::agent::AgentResult;

pub struct AgentLoop {
    llm: LlmClient,
    sandbox: Sandbox,
    cortex: Cortex,
    tools: Vec<ToolDef>,
}

impl AgentLoop {
    pub fn new(llm: LlmClient, sandbox: Sandbox, cortex: Cortex, tools: Vec<ToolDef>) -> Self {
        Self {
            llm,
            sandbox,
            cortex,
            tools,
        }
    }

    pub async fn run(&mut self, task: &str) -> Result<AgentResult> {
        println!("Planning task: {}", task);

        let plan = planner::generate_plan(task, &self.cortex, &mut self.llm).await;
        println!(
            "Plan complexity: {:?}, Steps: {}",
            plan.estimated_complexity,
            plan.steps.len()
        );

        for step in &plan.steps {
            println!("  - {} (using {})", step.description, step.tool_to_use);
        }

        let plan_json = serde_json::to_string_pretty(&plan).unwrap_or_default();

        let mut iteration = 0;
        let max_iterations = 50;
        let mut files_changed = Vec::new();
        let mut conversation = vec![
            Message {
                role: "system".to_string(),
                content: format!(
                    r#"You are an AI coding agent. Execute the following plan step by step.
Use the available tools to accomplish each step.
When all steps are complete, provide "TASK_COMPLETE" in your response.

Plan:
{}

Task: {}"#,
                    plan_json, task
                ),
            },
            Message {
                role: "user".to_string(),
                content: "Begin executing the plan.".to_string(),
            },
        ];

        while iteration < max_iterations {
            iteration += 1;

            let request_body = serde_json::json!({
                "model": self.llm.profile.model,
                "messages": conversation,
                "max_tokens": self.llm.profile.max_tokens,
                "temperature": self.llm.profile.temperature,
                "tools": self.tools.iter().map(|t| {
                    serde_json::json!({
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "description": t.description,
                            "parameters": t.parameters,
                        }
                    })
                }).collect::<Vec<_>>(),
            });

            let response = self
                .llm
                .client
                .post(&self.llm.profile.endpoint)
                .header("Authorization", format!("Bearer {}", self.llm.profile.api_key))
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await?;

            let body = response.text().await?;
            let json: serde_json::Value = serde_json::from_str(&body)?;

            let choice = &json["choices"][0]["message"];
            let content = choice["content"].as_str().unwrap_or("").to_string();
            let tool_calls = choice["tool_calls"].as_array();

            if let Some(calls) = tool_calls {
                for call in calls {
                    let name = call["function"]["name"].as_str().unwrap_or("");
                    let args_str = call["function"]["arguments"].as_str().unwrap_or("{}");
                    let call_id = call["id"].as_str().unwrap_or("");

                    let result = match name {
                        "execute_command" => {
                            let cmd = serde_json::from_str::<serde_json::Value>(args_str)
                                .ok()
                                .and_then(|v| v["cmd"].as_str().map(|s| s.to_string()))
                                .unwrap_or_else(|| "echo 'no command'".to_string());
                            tools::execute_command(&cmd, &self.sandbox.work_dir()).await
                        }
                        "read_file" => {
                            let path = serde_json::from_str::<serde_json::Value>(args_str)
                                .ok()
                                .and_then(|v| v["path"].as_str().map(|s| Path::new(s).to_path_buf()))
                                .unwrap_or_else(|| Path::new(".").to_path_buf());
                            tools::read_file(&path).await
                        }
                        "write_file" => {
                            let json: serde_json::Value = serde_json::from_str(args_str).unwrap_or_default();
                            let path = json["path"].as_str().unwrap_or("./output.txt");
                            let content = json["content"].as_str().unwrap_or("");
                            tools::write_file(Path::new(path), content).await
                        }
                        "grep_codebase" => {
                            tools::grep_codebase(
                                serde_json::from_str::<serde_json::Value>(args_str)
                                    .ok()
                                    .and_then(|v| v["pattern"].as_str().map(|s| s.to_string()))
                                    .unwrap_or_default()
                                    .as_str(),
                            ).await
                        }
                        "git_diff" => tools::git_diff().await,
                        "git_commit" => {
                            let msg = serde_json::from_str::<serde_json::Value>(args_str)
                                .ok()
                                .and_then(|v| v["message"].as_str().map(|s| s.to_string()))
                                .unwrap_or_else(|| "auto commit".to_string());
                            tools::git_commit(&msg).await
                        }
                        "git_branch" => {
                            let name = serde_json::from_str::<serde_json::Value>(args_str)
                                .ok()
                                .and_then(|v| v["name"].as_str().map(|s| s.to_string()))
                                .unwrap_or_else(|| "feature-branch".to_string());
                            tools::git_branch(&name).await
                        }
                        "list_files" => {
                            let dir = serde_json::from_str::<serde_json::Value>(args_str)
                                .ok()
                                .and_then(|v| v["dir"].as_str().map(|s| Path::new(s).to_path_buf()))
                                .unwrap_or_else(|| Path::new(".").to_path_buf());
                            tools::list_files(&dir).await
                        }
                        "run_tests" => tools::run_tests().await,
                        "read_lints" => tools::read_lints().await,
                        _ => ToolResult::error(&format!("Unknown tool: {}", name)),
                    };

                    conversation.push(Message {
                        role: "assistant".to_string(),
                        content: format!("Called tool: {}", name),
                    });
                    conversation.push(Message {
                        role: "tool".to_string(),
                        content: serde_json::to_string(&result).unwrap_or_default(),
                    });

                    if let Some(path) = extract_file_path(name, args_str) {
                        if !files_changed.contains(&path) {
                            files_changed.push(path);
                        }
                    }
                }
            } else {
                conversation.push(Message {
                    role: "assistant".to_string(),
                    content: content.clone(),
                });
            }

            if content.contains("TASK_COMPLETE") || content.contains("PLAN_COMPLETE") {
                println!("Task completed in {} iterations.", iteration);
                break;
            }

            if iteration >= max_iterations {
                println!("Reached max iterations ({}).", max_iterations);
                break;
            }
        }

        let diff = self.sandbox.get_diff().await.unwrap_or_default();

        Ok(AgentResult {
            summary: format!("Task completed in {} iterations", iteration),
            files_changed,
            diff,
            success: true,
        })
    }
}

fn extract_file_path(tool_name: &str, args: &str) -> Option<String> {
    match tool_name {
        "write_file" | "read_file" => {
            serde_json::from_str::<serde_json::Value>(args)
                .ok()
                .and_then(|v| v["path"].as_str().map(|s| s.to_string()))
        }
        _ => None,
    }
}
