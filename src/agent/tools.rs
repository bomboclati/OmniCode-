use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub output: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub error: Option<String>,
}

impl ToolResult {
    pub fn success(output: &str) -> Self {
        Self {
            tool_call_id: String::new(),
            output: output.to_string(),
            stdout: output.to_string(),
            stderr: String::new(),
            exit_code: 0,
            error: None,
        }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            tool_call_id: String::new(),
            output: msg.to_string(),
            stdout: String::new(),
            stderr: msg.to_string(),
            exit_code: 1,
            error: Some(msg.to_string()),
        }
    }
}

pub fn get_available_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "execute_command".to_string(),
            description: "Execute a shell command in the project directory".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "cmd": {
                        "type": "string",
                        "description": "The command to execute"
                    }
                },
                "required": ["cmd"]
            }),
        },
        ToolDef {
            name: "read_file".to_string(),
            description: "Read the contents of a file".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDef {
            name: "write_file".to_string(),
            description: "Write content to a file".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write"
                    }
                },
                "required": ["path", "content"]
            }),
        },
        ToolDef {
            name: "search_codebase".to_string(),
            description: "Semantic search in the codebase".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    }
                },
                "required": ["query"]
            }),
        },
        ToolDef {
            name: "grep_codebase".to_string(),
            description: "Regex search in the codebase".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Regex pattern"
                    }
                },
                "required": ["pattern"]
            }),
        },
        ToolDef {
            name: "git_diff".to_string(),
            description: "Get the current git diff".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDef {
            name: "git_commit".to_string(),
            description: "Create a git commit".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Commit message"
                    }
                },
                "required": ["message"]
            }),
        },
        ToolDef {
            name: "git_create_pr".to_string(),
            description: "Create a pull request".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "PR title"
                    },
                    "body": {
                        "type": "string",
                        "description": "PR body"
                    }
                },
                "required": ["title", "body"]
            }),
        },
        ToolDef {
            name: "git_branch".to_string(),
            description: "Create a new git branch".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Branch name"
                    }
                },
                "required": ["name"]
            }),
        },
        ToolDef {
            name: "list_files".to_string(),
            description: "List files in a directory".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "dir": {
                        "type": "string",
                        "description": "Directory path"
                    }
                },
                "required": ["dir"]
            }),
        },
        ToolDef {
            name: "run_tests".to_string(),
            description: "Run the project tests".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDef {
            name: "read_lints".to_string(),
            description: "Read linter warnings".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
    ]
}

pub async fn execute_command(cmd: &str, cwd: &Path) -> ToolResult {
    let mut parts = cmd.split_whitespace();
    let program = parts.next().unwrap_or("");
    let args: Vec<&str> = parts.collect();

    match Command::new(program).args(&args).current_dir(cwd).output() {
        Ok(output) => ToolResult {
            tool_call_id: String::new(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
        },
        Err(e) => ToolResult::error(&format!("Failed to execute command: {}", e)),
    }
}

pub async fn read_file(path: &Path) -> ToolResult {
    match std::fs::read_to_string(path) {
        Ok(content) => ToolResult::success(&content),
        Err(e) => ToolResult::error(&format!("Failed to read file: {}", e)),
    }
}

pub async fn write_file(path: &Path, content: &str) -> ToolResult {
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return ToolResult::error(&format!("Failed to create directory: {}", e));
        }
    }
    match std::fs::write(path, content) {
        Ok(_) => ToolResult::success(&format!("Written {} bytes to {}", content.len(), path.display())),
        Err(e) => ToolResult::error(&format!("Failed to write file: {}", e)),
    }
}

pub async fn grep_codebase(pattern: &str) -> ToolResult {
    let cwd = std::env::current_dir().unwrap_or_default();
    let output = Command::new("grep")
        .args(["-rn", "--include=*.rs", "--include=*.ts", "--include=*.js", "--include=*.py", pattern])
        .current_dir(&cwd)
        .output();

    match output {
        Ok(output) => ToolResult {
            tool_call_id: String::new(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            error: if output.status.success() {
                None
            } else {
                Some("No matches found".to_string())
            },
        },
        Err(e) => ToolResult::error(&format!("Failed to grep: {}", e)),
    }
}

pub async fn git_diff() -> ToolResult {
    let cwd = std::env::current_dir().unwrap_or_default();
    let output = Command::new("git")
        .args(["diff"])
        .current_dir(&cwd)
        .output();

    match output {
        Ok(output) => ToolResult {
            tool_call_id: String::new(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: 0,
            error: None,
        },
        Err(e) => ToolResult::error(&format!("Failed to run git diff: {}", e)),
    }
}

pub async fn git_commit(message: &str) -> ToolResult {
    let cwd = std::env::current_dir().unwrap_or_default();
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(&cwd)
        .output();

    match output {
        Ok(output) => ToolResult {
            tool_call_id: String::new(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
        },
        Err(e) => ToolResult::error(&format!("Failed to commit: {}", e)),
    }
}

pub async fn git_create_pr(title: &str, body: &str) -> ToolResult {
    let cwd = std::env::current_dir().unwrap_or_default();
    let output = Command::new("gh")
        .args(["pr", "create", "--title", title, "--body", body])
        .current_dir(&cwd)
        .output();

    match output {
        Ok(output) => ToolResult {
            tool_call_id: String::new(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
        },
        Err(e) => ToolResult::error(&format!("Failed to create PR: {}", e)),
    }
}

pub async fn git_branch(name: &str) -> ToolResult {
    let cwd = std::env::current_dir().unwrap_or_default();
    let output = Command::new("git")
        .args(["checkout", "-b", name])
        .current_dir(&cwd)
        .output();

    match output {
        Ok(output) => ToolResult {
            tool_call_id: String::new(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
        },
        Err(e) => ToolResult::error(&format!("Failed to create branch: {}", e)),
    }
}

pub async fn list_files(dir: &Path) -> ToolResult {
    match std::fs::read_dir(dir) {
        Ok(entries) => {
            let mut files = Vec::new();
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if path.is_dir() {
                    files.push(format!("{}/", name));
                } else {
                    files.push(name.to_string());
                }
            }
            files.sort();
            ToolResult::success(&files.join("\n"))
        }
        Err(e) => ToolResult::error(&format!("Failed to list directory: {}", e)),
    }
}

pub async fn run_tests() -> ToolResult {
    let cwd = std::env::current_dir().unwrap_or_default();
    let output = Command::new("cargo")
        .args(["test"])
        .current_dir(&cwd)
        .output();

    match output {
        Ok(output) => ToolResult {
            tool_call_id: String::new(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
        },
        Err(e) => ToolResult::error(&format!("Failed to run tests: {}", e)),
    }
}

pub async fn read_lints() -> ToolResult {
    let cwd = std::env::current_dir().unwrap_or_default();
    let output = Command::new("cargo")
        .args(["clippy", "--", "-D", "warnings"])
        .current_dir(&cwd)
        .output();

    match output {
        Ok(output) => ToolResult {
            tool_call_id: String::new(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
        },
        Err(e) => ToolResult::error(&format!("Failed to run clippy: {}", e)),
    }
}
