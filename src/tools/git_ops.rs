//! Git operation tools

use super::{Tool, ToolContext, ToolResult};
use crate::error::{Result, ToolError};
use async_trait::async_trait;
use tokio::process::Command;

/// Git Status tool
pub struct GitStatusTool;

impl GitStatusTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GitStatusTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitStatusTool {
    fn name(&self) -> &str {
        "git_status"
    }

    fn description(&self) -> &str {
        "Get the status of the Git repository (e.g., modified, untracked files). Returns output of `git status --porcelain`."
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
        })
    }

    fn is_read_only(&self) -> bool {
        true
    }

    async fn execute(&self, _args: serde_json::Value, ctx: &ToolContext, _config: &crate::config::Config) -> Result<ToolResult> {
        tracing::info!("Executing git status");

        let output = Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .current_dir(&ctx.working_dir)
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            Ok(ToolResult::success(stdout))
        } else {
            Err(ToolError::ExecutionFailed(format!("git status failed: {}", stderr)).into())
        }
    }
}

/// Git Diff tool
pub struct GitDiffTool;

impl GitDiffTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GitDiffTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitDiffTool {
    fn name(&self) -> &str {
        "git_diff"
    }

    fn description(&self) -> &str {
        "Get the diff of changes in the Git repository. Can specify a file path to diff a single file. Returns output of `git diff`."
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Optional: Path to a specific file to diff"
                }
            }
        })
    }

    fn is_read_only(&self) -> bool {
        true
    }

    async fn execute(&self, args: serde_json::Value, ctx: &ToolContext, _config: &crate::config::Config) -> Result<ToolResult> {
        tracing::info!("Executing git diff");

        let mut command = Command::new("git");
        command.arg("diff").current_dir(&ctx.working_dir);

        if let Some(path) = args["path"].as_str() {
            command.arg(path);
        }

        let output = command.output().await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            Ok(ToolResult::success(stdout))
        } else {
            Err(ToolError::ExecutionFailed(format!("git diff failed: {}", stderr)).into())
        }
    }
}

/// Git Commit tool
pub struct GitCommitTool;

impl GitCommitTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GitCommitTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitCommitTool {
    fn name(&self) -> &str {
        "git_commit"
    }

    fn description(&self) -> &str {
        "Commit changes to the Git repository. Requires a commit message."
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "The commit message"
                }
            },
            "required": ["message"]
        })
    }

    fn is_read_only(&self) -> bool {
        false
    }

    async fn execute(&self, args: serde_json::Value, ctx: &ToolContext, _config: &crate::config::Config) -> Result<ToolResult> {
        let message = args["message"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgs("Missing commit message".to_string()))?;

        tracing::info!("Executing git commit with message: {}", message);

        let output = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(message)
            .current_dir(&ctx.working_dir)
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            Ok(ToolResult::success(stdout))
        } else {
            Err(ToolError::ExecutionFailed(format!("git commit failed: {}", stderr)).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    async fn setup_git_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        Command::new("git")
            .arg("init")
            .current_dir(repo_path)
            .output()
            .await
            .unwrap();

        // Configure dummy user for commit
        Command::new("git")
            .arg("config")
            .arg("user.email")
            .arg("test@example.com")
            .current_dir(repo_path)
            .output()
            .await
            .unwrap();
        Command::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Test User")
            .current_dir(repo_path)
            .output()
            .await
            .unwrap();

        temp_dir
    }

    #[tokio::test]
    async fn test_git_status() {
        let temp_dir = setup_git_repo().await;
        let repo_path = temp_dir.path();

        fs::write(repo_path.join("test.txt"), "hello").unwrap();

        let tool = GitStatusTool::new();
        let ctx = ToolContext {
            working_dir: repo_path.to_path_buf(),
            ..Default::default()
        };
        let config = crate::config::Config::default();

        let result = tool.execute(serde_json::json!({}), &ctx, &config).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("?? test.txt"));
    }

    #[tokio::test]
    async fn test_git_diff() {
        let temp_dir = setup_git_repo().await;
        let repo_path = temp_dir.path();

        fs::write(repo_path.join("test.txt"), "initial content").unwrap();
        Command::new("git")
            .arg("add")
            .arg("test.txt")
            .current_dir(repo_path)
            .output()
            .await
            .unwrap();
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("Initial commit")
            .current_dir(repo_path)
            .output()
            .await
            .unwrap();

        fs::write(repo_path.join("test.txt"), "modified content").unwrap();

        let tool = GitDiffTool::new();
        let ctx = ToolContext {
            working_dir: repo_path.to_path_buf(),
            ..Default::default()
        };
        let config = crate::config::Config::default();

        let result = tool.execute(serde_json::json!({"path": "test.txt"}), &ctx, &config).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("-initial content"));
        assert!(result.output.contains("+modified content"));
    }

    #[tokio::test]
    async fn test_git_commit() {
        let temp_dir = setup_git_repo().await;
        let repo_path = temp_dir.path();

        fs::write(repo_path.join("test.txt"), "content").unwrap();
        Command::new("git")
            .arg("add")
            .arg("test.txt")
            .current_dir(repo_path)
            .output()
            .await
            .unwrap();

        let tool = GitCommitTool::new();
        let ctx = ToolContext {
            working_dir: repo_path.to_path_buf(),
            ..Default::default()
        };
        let config = crate::config::Config::default();

        let result = tool.execute(serde_json::json!({"message": "Test commit"}), &ctx, &config).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Test commit"));

        let log_output = Command::new("git")
            .arg("log")
            .arg("--oneline")
            .current_dir(repo_path)
            .output()
            .await
            .unwrap();
        let log_str = String::from_utf8_lossy(&log_output.stdout).to_string();
        assert!(log_str.contains("Test commit"));
    }
}
