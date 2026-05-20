use crate::config::{Profile, Provider};
use crate::agent::tools::ToolDef;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

pub struct LlmClient {
    pub client: Client,
    pub profile: Profile,
    pub history: Vec<Message>,
}

impl LlmClient {
    pub fn new(profile: Profile) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("failed to build HTTP client"),
            profile,
            history: Vec::new(),
        }
    }

    pub async fn chat(&mut self, user_message: &str, tools: &[ToolDef]) -> Result<String> {
        self.history.push(Message {
            role: "user".to_string(),
            content: user_message.to_string(),
        });

        let mut messages = vec![Message {
            role: "system".to_string(),
            content: self.system_prompt(),
        }];
        messages.extend(self.history.clone());

        let mut request_body = serde_json::json!({
            "model": self.profile.model,
            "messages": messages,
            "max_tokens": self.profile.max_tokens,
            "temperature": self.profile.temperature,
        });

        if !tools.is_empty() {
            let tool_defs: Vec<serde_json::Value> = tools
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "description": t.description,
                            "parameters": t.parameters,
                        }
                    })
                })
                .collect();
            request_body["tools"] = serde_json::Value::Array(tool_defs);
        }

        let response = self
            .client
            .post(&self.profile.endpoint)
            .header("Authorization", format!("Bearer {}", self.profile.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(anyhow::anyhow!("LLM API error {}: {}", status, body));
        }

        let json: serde_json::Value = serde_json::from_str(&body)?;

        let choice = &json["choices"][0]["message"];
        let content = choice["content"].as_str().unwrap_or("").to_string();

        let tool_calls = choice["tool_calls"].as_array();
        if let Some(calls) = tool_calls {
            for call in calls {
                let tool_call = ToolCall {
                    id: call["id"].as_str().unwrap_or("").to_string(),
                    name: call["function"]["name"].as_str().unwrap_or("").to_string(),
                    arguments: call["function"]["arguments"]
                        .as_str()
                        .unwrap_or("{}")
                        .to_string(),
                };
                self.history.push(Message {
                    role: "assistant".to_string(),
                    content: format!("Calling tool: {}", tool_call.name),
                });
            }
        }

        if !content.is_empty() {
            self.history.push(Message {
                role: "assistant".to_string(),
                content: content.clone(),
            });
        }

        Ok(content)
    }

    pub async fn chat_streaming(
        &mut self,
        user_message: &str,
        tx: mpsc::UnboundedSender<String>,
    ) -> Result<()> {
        self.history.push(Message {
            role: "user".to_string(),
            content: user_message.to_string(),
        });

        let mut messages = vec![
            Message {
                role: "system".to_string(),
                content: self.system_prompt(),
            },
        ];
        messages.extend(self.history.clone());

        let request_body = serde_json::json!({
            "model": self.profile.model,
            "messages": messages,
            "max_tokens": self.profile.max_tokens,
            "temperature": self.profile.temperature,
            "stream": true,
        });

        let response = self
            .client
            .post(&self.profile.endpoint)
            .header("Authorization", format!("Bearer {}", self.profile.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        use futures::StreamExt;
        let mut full_response = String::new();
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let text = String::from_utf8_lossy(&chunk);
            for line in text.lines() {
                if line.starts_with("data: ") && !line.contains("[DONE]") {
                    let data = &line[6..];
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                            let _ = tx.send(content.to_string());
                            full_response.push_str(content);
                        }
                    }
                }
            }
        }

        self.history.push(Message {
            role: "assistant".to_string(),
            content: full_response,
        });

        Ok(())
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    fn system_prompt(&self) -> String {
        format!(
            r#"You are OmniCode, an elite AI coding assistant. You help users write, debug, and improve code.
You are working in a Rust project. Always provide complete, production-ready code.
When making changes, explain your reasoning clearly.
Follow best practices for the language and framework in use.
Always consider security implications of your suggestions.
Current model: {}
Provider: {}"#,
            self.profile.model, self.profile.provider
        )
    }
}
