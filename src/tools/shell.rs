//! Shell command execution tool

use super::{Tool, ToolContext, ToolResult};
use crate::error::Result;
use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;

/// Shell command execution tool
pub struct ShellTool {
    timeout_secs: u64,
}

impl ShellTool {
    pub fn new() -> Self {
        Self { timeout_secs: 30 }
    }

    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self { timeout_secs }
    }
}

impl Default for ShellTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ShellTool {
    fn name(&self) -> &str {
        "shell_execute"
    }

    fn description(&self) -> &str {
        "Execute a shell command and return its output. Use for running system commands, listing files, searching, etc."
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute"
                }
            },
            "required": ["command"]
        })
    }

    fn is_read_only(&self) -> bool {
        // Shell commands can be destructive, so not read-only
        false
    }

    async fn execute(&self, args: serde_json::Value, ctx: &ToolContext, _config: &crate::config::Config) -> Result<ToolResult> {
        let command = args["command"]
            .as_str()
            .ok_or_else(|| crate::error::ToolError::InvalidArgs("Missing command".to_string()))?;

        tracing::info!("Executing shell command: {}", command);

        // Determine shell based on OS
        let (shell, shell_arg) = if cfg!(target_os = "windows") {
            ("cmd", "/C")
        } else {
            ("sh", "-c")
        };

        // Execute command with timeout
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(self.timeout_secs),
            Command::new(shell)
                .arg(shell_arg)
                .arg(command)
                .current_dir(&ctx.working_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output(),
        )
        .await
        .map_err(|_| crate::error::ToolError::Timeout)??;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            Ok(ToolResult::success(stdout)
                .with_metadata("exit_code", serde_json::json!(0))
                .with_metadata("stderr", serde_json::json!(stderr)))
        } else {
            let exit_code = output.status.code().unwrap_or(-1);
            Ok(ToolResult::error(format!(
                "Command failed with exit code {}: {}",
                exit_code, stderr
            ))
            .with_metadata("exit_code", serde_json::json!(exit_code))
            .with_metadata("stdout", serde_json::json!(stdout)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shell_execute_success() {
        let tool = ShellTool::new();
        let ctx = ToolContext::default();
        let config = crate::config::Config::default();

        let command = if cfg!(target_os = "windows") {
            "echo hello"
        } else {
            "echo hello"
        };

        let result = tool
            .execute(serde_json::json!({"command": command}), &ctx, &config)
            .await
            .unwrap();

        assert!(result.success);
        assert!(result.output.contains("hello"));
    }

    #[tokio::test]
    async fn test_shell_execute_failure() {
        let tool = ShellTool::new();
        let ctx = ToolContext::default();
        let config = crate::config::Config::default();

        let result = tool
            .execute(
                serde_json::json!({"command": "nonexistent_command_xyz"}),
                &ctx,
                &config,
            )
            .await
            .unwrap();

        assert!(!result.success);
    }

    #[tokio::test]
    async fn test_shell_timeout() {
        let tool = ShellTool::with_timeout(2);
        let ctx = ToolContext::default();
        let config = crate::config::Config::default();

        // Command that takes longer than timeout
        let command = if cfg!(target_os = "windows") {
            "ping -n 6 127.0.0.1 > nul"
        } else {
            "sleep 5"
        };

        let result = tool
            .execute(serde_json::json!({"command": command}), &ctx, &config)
            .await;

        assert!(result.is_err());
    }
}
