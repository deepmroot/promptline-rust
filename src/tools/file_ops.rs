//! File operation tools

use super::{Tool, ToolContext, ToolResult};
use crate::error::{Result, ToolError};
use async_trait::async_trait;

use crate::util::diff::display_diff;
use dialoguer::Confirm;

/// File read tool
pub struct FileReadTool;

impl FileReadTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileReadTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FileReadTool {
    fn name(&self) -> &str {
        "file_read"
    }

    fn description(&self) -> &str {
        "Read the contents of a file. Use this to examine source code, configuration files, or any text file."
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read"
                }
            },
            "required": ["path"]
        })
    }

    fn is_read_only(&self) -> bool {
        true
    }

    async fn execute(&self, args: serde_json::Value, ctx: &ToolContext, _config: &crate::config::Config) -> Result<ToolResult> {
        let path_str = args["path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgs("Missing path".to_string()))?;

        // Resolve path relative to working directory
        let path = if std::path::Path::new(path_str).is_absolute() {
            std::path::PathBuf::from(path_str)
        } else {
            ctx.working_dir.join(path_str)
        };

        tracing::info!("Reading file: {} (resolved from {})", path.display(), path_str);

        // Check if file exists
        if !path.exists() {
            return Ok(ToolResult::error(format!("File not found: {}", path.display())));
        }

        // Check file size (limit to 1MB for safety)
        let metadata = tokio::fs::metadata(&path).await?;
        if metadata.len() > 1_000_000 {
            return Ok(ToolResult::error(format!(
                "File too large: {} bytes (max 1MB)",
                metadata.len()
            )));
        }

        // Read file
        let content = tokio::fs::read_to_string(&path).await.map_err(|e| {
            ToolError::ExecutionFailed(format!("Failed to read file: {}", e))
        })?;

        Ok(ToolResult::success(content)
            .with_metadata("path", serde_json::json!(path))
            .with_metadata("size", serde_json::json!(metadata.len())))
    }
}

/// File write tool
pub struct FileWriteTool;

impl FileWriteTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileWriteTool {
    fn default() -> Self {
        Self::new()
    }
}

// ... (rest of the file is the same until FileWriteTool::execute)

#[async_trait]
impl Tool for FileWriteTool {
    fn name(&self) -> &str {
        "file_write"
    }

    fn description(&self) -> &str {
        "Write content to a file. Creates the file if it doesn't exist, or overwrites if it does. Parent directories must exist."
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, args: serde_json::Value, ctx: &ToolContext, config: &crate::config::Config) -> Result<ToolResult> {
        let path_str = args["path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgs("Missing path".to_string()))?;

        let content = args["content"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgs("Missing content".to_string()))?;

        // Resolve path relative to working directory
        let path = if std::path::Path::new(path_str).is_absolute() {
            std::path::PathBuf::from(path_str)
        } else {
            ctx.working_dir.join(path_str)
        };

        tracing::info!("Writing to file: {} (resolved from {})", path.display(), path_str);

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                tokio::fs::create_dir_all(parent).await.map_err(|e| {
                    ToolError::ExecutionFailed(format!("Failed to create parent directory {}: {}", parent.display(), e))
                })?;
            }
        }

        // If file exists, generate and display diff
        if path.exists() {
            let original_content = tokio::fs::read_to_string(&path).await.unwrap_or_default();
            display_diff(path_str, &original_content, content);

            if config.safety.require_diff_preview {
                let confirmation = Confirm::new()
                    .with_prompt("Apply these changes?")
                    .default(false)
                    .interact()?;

                if !confirmation {
                    return Ok(ToolResult::error("User denied file write.".to_string()));
                }
            }
        }

        // Write file
        tokio::fs::write(&path, content).await.map_err(|e| {
            ToolError::ExecutionFailed(format!("Failed to write file: {}", e))
        })?;

        Ok(ToolResult::success(format!("Successfully wrote {} bytes to {}", content.len(), path.display()))
            .with_metadata("path", serde_json::json!(path))
            .with_metadata("bytes_written", serde_json::json!(content.len())))
    }
}

/// File list tool
pub struct FileListTool;

impl FileListTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileListTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FileListTool {
    fn name(&self) -> &str {
        "file_list"
    }

    fn description(&self) -> &str {
        "List files in a directory. Useful for exploring the project structure."
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the directory to list (defaults to current directory)"
                }
            }
        })
    }

    fn is_read_only(&self) -> bool {
        true
    }

    async fn execute(&self, args: serde_json::Value, ctx: &ToolContext, _config: &crate::config::Config) -> Result<ToolResult> {
        let path_str = args["path"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| ".".to_string());

        // Resolve path relative to working directory
        let path = if std::path::Path::new(&path_str).is_absolute() {
            std::path::PathBuf::from(&path_str)
        } else {
            ctx.working_dir.join(&path_str)
        };

        tracing::info!("Listing directory: {} (resolved from {})", path.display(), path_str);

        if !path.exists() {
             return Ok(ToolResult::error(format!("Directory not found: {}", path.display())));
        }

        let mut entries = tokio::fs::read_dir(&path).await.map_err(|e| {
            ToolError::ExecutionFailed(format!("Failed to read directory {}: {}", path.display(), e))
        })?;

        let mut files = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            let file_type = if metadata.is_dir() {
                "dir"
            } else if metadata.is_symlink() {
                "link"
            } else {
                "file"
            };

            files.push(format!(
                "{:<10} {:<10} {}",
                file_type,
                metadata.len(),
                entry.file_name().to_string_lossy()
            ));
        }

        let output = if files.is_empty() {
            "Directory is empty".to_string()
        } else {
            format!(
                "Found {} items:\n{}",
                files.len(),
                files.join("\n")
            )
        };

        Ok(ToolResult::success(output).with_metadata("path", serde_json::json!(path)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_read() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "Hello, World!").unwrap();

        let tool = FileReadTool::new();
        let ctx = ToolContext::default();
        let config = crate::config::Config::default();

        let result = tool
            .execute(
                serde_json::json!({"path": file_path.to_str().unwrap()}),
                &ctx,
                &config,
            )
            .await
            .unwrap();

        assert!(result.success);
        assert_eq!(result.output, "Hello, World!");
    }

    #[tokio::test]
    async fn test_file_write() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("output.txt");

        let tool = FileWriteTool::new();
        let ctx = ToolContext::default();
        let config = crate::config::Config::default();

        let result = tool
            .execute(
                serde_json::json!({
                    "path": file_path.to_str().unwrap(),
                    "content": "Test content"
                }),
                &ctx,
                &config,
            )
            .await
            .unwrap();

        assert!(result.success);

        // Verify file was written
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Test content");
    }

    #[tokio::test]
    async fn test_file_list() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("file1.txt"), "").unwrap();
        std::fs::write(temp_dir.path().join("file2.txt"), "").unwrap();

        let tool = FileListTool::new();
        let ctx = ToolContext::default();
        let config = crate::config::Config::default();

        let result = tool
            .execute(
                serde_json::json!({"path": temp_dir.path().to_str().unwrap()}),
                &ctx,
                &config,
            )
            .await
            .unwrap();

        assert!(result.success);
        assert!(result.output.contains("file1.txt"));
        assert!(result.output.contains("file2.txt"));
    }
}
