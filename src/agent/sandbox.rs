use crate::agent::tools::ToolResult;
use crate::config::SandboxMode;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

pub enum Sandbox {
    Direct(PathBuf),
    GitWorktree(PathBuf, PathBuf),
    Docker(String),
    Firecracker(String),
}

impl Sandbox {
    pub async fn create(project_root: PathBuf, mode: &SandboxMode) -> Result<Self> {
        match mode {
            SandboxMode::Local => {
                println!("Warning: Running in direct mode without sandbox isolation");
                Ok(Sandbox::Direct(project_root.clone()))
            }
            SandboxMode::Docker => {
                let container_id = format!("omnicode-{}", uuid::Uuid::new_v4());
                println!("Creating Docker sandbox: {}", container_id);
                Ok(Sandbox::Docker(container_id))
            }
            SandboxMode::Firecracker => {
                let vm_id = format!("omnicode-vm-{}", uuid::Uuid::new_v4());
                println!("Creating Firecracker sandbox: {}", vm_id);
                Ok(Sandbox::Firecracker(vm_id))
            }
        }
    }

    pub async fn create_worktree(project_root: &Path) -> Result<Self> {
        let worktree_name = format!("/tmp/omni-sandbox-{}", uuid::Uuid::new_v4());
        let worktree_path = PathBuf::from(&worktree_name);

        let output = Command::new("git")
            .args([
                "worktree",
                "add",
                &worktree_name,
            ])
            .current_dir(project_root)
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to create worktree: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(Sandbox::GitWorktree(
            project_root.to_path_buf(),
            worktree_path,
        ))
    }

    pub async fn execute(&self, cmd: &str) -> ToolResult {
        let cwd = self.work_dir();

        let mut parts = cmd.split_whitespace();
        let program = parts.next().unwrap_or("echo");
        let args: Vec<&str> = parts.collect();

        match Command::new(program)
            .args(&args)
            .current_dir(&cwd)
            .output()
        {
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
            Err(e) => ToolResult::error(&format!("Failed to execute: {}", e)),
        }
    }

    pub async fn cleanup(&self) -> Result<()> {
        match self {
            Sandbox::Direct(_) => Ok(()),
            Sandbox::GitWorktree(project_root, worktree_path) => {
                Command::new("git")
                    .args(["worktree", "remove", worktree_path.to_str().unwrap_or("")])
                    .current_dir(project_root)
                    .output()?;
                Ok(())
            }
            Sandbox::Docker(container_id) => {
                Command::new("docker")
                    .args(["rm", "-f", container_id])
                    .output()?;
                Ok(())
            }
            Sandbox::Firecracker(vm_id) => {
                println!("Cleaning up Firecracker VM: {}", vm_id);
                Ok(())
            }
        }
    }

    pub async fn get_diff(&self) -> Result<String> {
        match self {
            Sandbox::Direct(cwd) | Sandbox::GitWorktree(_, cwd) => {
                let output = Command::new("git")
                    .args(["diff"])
                    .current_dir(cwd)
                    .output()?;
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            }
            Sandbox::Docker(_) | Sandbox::Firecracker(_) => {
                Ok("Diff not available for this sandbox type".to_string())
            }
        }
    }

    pub fn work_dir(&self) -> PathBuf {
        match self {
            Sandbox::Direct(path) => path.clone(),
            Sandbox::GitWorktree(_, worktree) => worktree.clone(),
            Sandbox::Docker(_) => PathBuf::from("/workspace"),
            Sandbox::Firecracker(_) => PathBuf::from("/workspace"),
        }
    }
}
