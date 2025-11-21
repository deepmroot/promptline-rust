//! Codebase search tool

use super::{Tool, ToolContext, ToolResult};
use crate::error::{Result, ToolError};
use async_trait::async_trait;
use tokio::process::Command;
use std::path::PathBuf;

/// Codebase Search tool
pub struct CodebaseSearchTool;

impl CodebaseSearchTool {
    pub fn new() -> Self {
        Self
    }

    async fn search_with_powershell(&self, pattern: &str, path: &PathBuf, ctx: &ToolContext) -> Result<ToolResult> {
        // Use PowerShell Select-String for Windows
        let ps_command = format!(
            "Get-ChildItem -Path '{}' -Recurse -File | Select-String -Pattern '{}' | ForEach-Object {{ \"$($_.Path):$($_.LineNumber):$($_.Line)\" }}",
            path.display(),
            pattern.replace("'", "''") // Escape single quotes
        );

        let output = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(&ps_command)
            .current_dir(&ctx.working_dir)
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        
        if output.status.success() {
            Ok(ToolResult::success(stdout))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(ToolError::ExecutionFailed(format!("PowerShell search failed: {}", stderr)).into())
        }
    }
}

impl Default for CodebaseSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for CodebaseSearchTool {
    fn name(&self) -> &str {
        "codebase_search"
    }

    fn description(&self) -> &str {
        "Search for a regular expression pattern within the content of files in a specified directory (or current working directory). Uses `ripgrep` if available, otherwise falls back to `grep`. Returns the lines containing matches, along with their file paths and line numbers."
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "The regular expression pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Optional: The path to the directory to search within. Defaults to current working directory."
                }
            },
            "required": ["pattern"]
        })
    }

    fn is_read_only(&self) -> bool {
        true
    }

    async fn execute(&self, args: serde_json::Value, ctx: &ToolContext, _config: &crate::config::Config) -> Result<ToolResult> {
        let pattern = args["pattern"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgs("Missing search pattern".to_string()))?;

        let path = args["path"]
            .as_str()
            .map(PathBuf::from)
            .unwrap_or_else(|| ctx.working_dir.clone());

        tracing::info!("Searching for pattern '{}' in '{}'", pattern, path.display());

        let mut command;
        let mut args_vec = Vec::new();

        // Check for ripgrep
        if Command::new("rg").arg("--version").output().await.is_ok() {
            command = Command::new("rg");
            args_vec.push("--line-number"); // Show line numbers
            args_vec.push("--with-filename"); // Show filename
            args_vec.push("--color=never"); // Disable color output for easier parsing
            args_vec.push(pattern);
            args_vec.push(path.to_str().unwrap_or("."));
        } else if Command::new("grep").arg("--version").output().await.is_ok() {
            // Fallback to grep (Unix/Linux)
            command = Command::new("grep");
            args_vec.push("-n"); // Show line numbers
            args_vec.push("-r"); // Recursive search
            args_vec.push(pattern);
            args_vec.push(path.to_str().unwrap_or("."));
        } else {
            // Windows fallback: Use PowerShell Select-String
            return self.search_with_powershell(pattern, &path, ctx).await;
        }

        let output = command
            .args(&args_vec)
            .current_dir(&ctx.working_dir)
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() || output.status.code() == Some(1) { // grep/rg returns 1 for no matches
            Ok(ToolResult::success(stdout))
        } else {
            Err(ToolError::ExecutionFailed(format!("Codebase search failed: {}", stderr)).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_codebase_search_success() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        fs::write(dir_path.join("file1.txt"), "hello world").unwrap();
        fs::write(dir_path.join("file2.rs"), "fn main() {\n    println!(\"hello\");\n}").unwrap();

        let tool = CodebaseSearchTool::new();
        let ctx = ToolContext {
            working_dir: dir_path.to_path_buf(),
            ..Default::default()
        };
        let config = crate::config::Config::default();

        let result = tool.execute(serde_json::json!({"pattern": "hello"}), &ctx, &config).await;
        // Skip test if ripgrep is not installed
        if let Err(e) = result {
            if e.to_string().contains("program not found") {
                eprintln!("Skipping test: ripgrep not installed");
                return;
            }
            panic!("Unexpected error: {}", e);
        }
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.output.contains("file1.txt") || result.output.contains("hello"));
    }

    #[tokio::test]
    async fn test_codebase_search_no_match() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        fs::write(dir_path.join("file1.txt"), "foo bar").unwrap();

        let tool = CodebaseSearchTool::new();
        let ctx = ToolContext {
            working_dir: dir_path.to_path_buf(),
            ..Default::default()
        };
        let config = crate::config::Config::default();

        let result = tool.execute(serde_json::json!({"pattern": "nonexistent"}), &ctx, &config).await;
        // Skip test if ripgrep is not installed
        if let Err(e) = result {
            if e.to_string().contains("program not found") {
                eprintln!("Skipping test: ripgrep not installed");
                return;
            }
            panic!("Unexpected error: {}", e);
        }
        let result = result.unwrap();
        assert!(result.success); // grep/rg returns success (exit code 1) for no matches
        assert!(result.output.is_empty() || !result.output.contains("nonexistent"));
    }

    #[tokio::test]
    async fn test_codebase_search_invalid_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        let tool = CodebaseSearchTool::new();
        let ctx = ToolContext {
            working_dir: dir_path.to_path_buf(),
            ..Default::default()
        };
        let config = crate::config::Config::default();

        let result = tool.execute(serde_json::json!({"pattern": "["}), &ctx, &config).await; // Invalid regex pattern
        // Skip test if ripgrep is not installed
        if let Err(e) = &result {
            if e.to_string().contains("program not found") {
                eprintln!("Skipping test: ripgrep not installed");
                return;
            }
        }
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Codebase search failed") || err.to_string().contains("error"));
    }
}
