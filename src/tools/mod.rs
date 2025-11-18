//! Tool execution interface

use crate::error::{Result, ToolError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod file_ops;
pub mod git_ops;
pub mod search_ops;
pub mod shell;
pub mod web_ops;

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ToolResult {
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: output.into(),
            error: None,
            metadata: HashMap::new(),
        }
    }

    pub fn error(error: impl Into<String>) -> Self {
        Self {
            success: false,
            output: String::new(),
            error: Some(error.into()),
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// Tool context passed to tools during execution
#[derive(Debug, Clone)]
pub struct ToolContext {
    pub working_dir: std::path::PathBuf,
    pub env_vars: HashMap<String, String>,
    pub current_working_dir: std::path::PathBuf,
    pub git_branch: Option<String>,
}

impl Default for ToolContext {
    fn default() -> Self {
        Self {
            working_dir: std::env::current_dir().unwrap_or_default(),
            env_vars: HashMap::new(),
            current_working_dir: std::env::current_dir().unwrap_or_default(),
            git_branch: None,
        }
    }
}

/// Tool trait for implementing actions
#[async_trait]
pub trait Tool: Send + Sync {
    /// Unique tool name
    fn name(&self) -> &str;

    /// Human-readable description
    fn description(&self) -> &str;

    /// Parameter schema (JSON Schema format)
    fn parameters(&self) -> serde_json::Value;

    /// Whether tool is read-only (safe for plan mode)
    fn is_read_only(&self) -> bool {
        false
    }

    /// Execute the tool with given arguments
    async fn execute(
        &self,
        args: serde_json::Value,
        ctx: &ToolContext,
        config: &crate::config::Config,
    ) -> Result<ToolResult>;

    /// Validate arguments before execution
    fn validate_args(&self, args: &serde_json::Value) -> Result<()> {
        // Basic validation - check required fields
        let schema = self.parameters();
        if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
            for field in required {
                if let Some(field_name) = field.as_str() {
                    if args.get(field_name).is_none() {
                        return Err(ToolError::InvalidArgs(format!(
                            "Missing required field: {}",
                            field_name
                        ))
                        .into());
                    }
                }
            }
        }
        Ok(())
    }

    /// Convert tool to definition for model
    fn to_definition(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name(),
            "description": self.description(),
            "parameters": self.parameters(),
        })
    }
}

/// Tool registry for managing available tools
#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool
    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        self.tools.insert(tool.name().to_string(), Box::new(tool));
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    /// Execute a tool by name
    pub async fn execute(
        &self,
        name: &str,
        args: serde_json::Value,
        ctx: &ToolContext,
        config: &crate::config::Config,
    ) -> Result<ToolResult> {
        let tool = self
            .get(name)
            .ok_or_else(|| ToolError::NotFound(name.to_string()))?;

        tool.validate_args(&args)?;
        tool.execute(args, ctx, config).await
    }

    /// List all registered tools
    pub fn list(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    /// Get tool definitions for model
    pub fn definitions(&self) -> Vec<serde_json::Value> {
        self.tools.values().map(|t| t.to_definition()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTool;

    #[async_trait]
    impl Tool for TestTool {
        fn name(&self) -> &str {
            "test_tool"
        }

        fn description(&self) -> &str {
            "A test tool"
        }

        fn parameters(&self) -> serde_json::Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "input": {
                        "type": "string",
                        "description": "Test input"
                    }
                },
                "required": ["input"]
            })
        }

        async fn execute(&self, args: serde_json::Value, _ctx: &ToolContext, _config: &crate::config::Config) -> Result<ToolResult> {
            let input = args["input"].as_str().unwrap_or("");
            Ok(ToolResult::success(format!("Processed: {}", input)))
        }
    }

    #[tokio::test]
    async fn test_tool_registry() {
        let mut registry = ToolRegistry::new();
        registry.register(TestTool);

        assert!(registry.get("test_tool").is_some());
        assert!(registry.get("nonexistent").is_none());

        let ctx = ToolContext::default();
        let config = crate::config::Config::default();
        let result = registry
            .execute(
                "test_tool",
                serde_json::json!({"input": "hello"}),
                &ctx,
                &config,
            )
            .await
            .unwrap();

        assert!(result.success);
        assert!(result.output.contains("hello"));
    }

    #[tokio::test]
    async fn test_missing_required_field() {
        let tool = TestTool;
        let result = tool.validate_args(&serde_json::json!({}));
        assert!(result.is_err());
    }
}
