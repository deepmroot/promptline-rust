//! Configuration management for PromptLine

use crate::error::{ConfigError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Model configuration
    #[serde(default)]
    pub models: ModelConfig,

    /// Tool permissions
    #[serde(default)]
    pub tools: ToolPermissions,

    /// Safety configuration
    #[serde(default)]
    pub safety: SafetyConfig,

    /// Agent behavior
    #[serde(default)]
    pub agent: AgentConfig,
}

/// Model provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Default model to use
    #[serde(default = "default_model")]
    pub default: String,

    /// Provider configurations
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
}

/// Provider-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// API key (can use env var syntax: ${VAR_NAME})
    pub api_key: Option<String>,

    /// Available models
    #[serde(default)]
    pub models: Vec<String>,

    /// Default parameters
    #[serde(default)]
    pub default_params: ModelParams,

    /// Base URL for the API (optional)
    pub base_url: Option<String>,
}

/// Model parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelParams {
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
}

/// Tool permission levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionLevel {
    /// Automatically execute (read-only tools)
    Allow,
    /// Ask user for approval (default)
    Ask,
    /// Never allow
    Deny,
}

/// Tool permissions configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPermissions {
    #[serde(default = "default_permission")]
    pub file_read: PermissionLevel,

    #[serde(default = "default_permission")]
    pub file_write: PermissionLevel,

    #[serde(default = "default_permission")]
    pub file_delete: PermissionLevel,

    #[serde(default = "default_permission")]
    pub shell_execute: PermissionLevel,

    #[serde(default = "default_permission")]
    pub git_status: PermissionLevel,

    #[serde(default = "default_permission")]
    pub git_diff: PermissionLevel,

    #[serde(default = "default_permission")]
    pub git_commit: PermissionLevel,

    #[serde(default = "default_permission")]
    pub web_get: PermissionLevel,

    #[serde(default = "default_permission")]
    pub codebase_search: PermissionLevel,
}

impl ToolPermissions {
    pub fn get_tool_permission(&self, tool_name: &str) -> PermissionLevel {
        match tool_name {
            "file_read" => self.file_read,
            "file_write" => self.file_write,
            "file_delete" => self.file_delete,
            "shell_execute" => self.shell_execute,
            "git_status" => self.git_status,
            "git_diff" => self.git_diff,
            "git_commit" => self.git_commit,
            "web_get" => self.web_get,
            "codebase_search" => self.codebase_search,
            _ => PermissionLevel::Ask, // Default to Ask for unknown tools
        }
    }
}

/// Safety configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    /// Require approval for actions
    #[serde(default = "default_true")]
    pub require_approval: bool,

    /// Show diffs before file changes
    #[serde(default = "default_true")]
    pub require_diff_preview: bool,

    /// Maximum agent iterations
    #[serde(default = "default_max_iterations")]
    pub max_iterations: usize,

    /// Dangerous command patterns
    #[serde(default = "default_dangerous_commands")]
    pub dangerous_commands: Vec<String>,

    /// Allowed commands (if specified, only these are allowed)
    pub allowed_commands: Option<Vec<String>>,

    /// Denied commands (these are never allowed)
    pub denied_commands: Option<Vec<String>>,

    /// Protected file patterns
    #[serde(default = "default_protected_patterns")]
    pub protected_patterns: Vec<String>,

    /// Enable backups before file changes
    #[serde(default = "default_true")]
    pub enable_backups: bool,
}

/// Agent behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Default mode (plan or agent)
    #[serde(default = "default_mode")]
    pub default_mode: String,

    /// Use chain-of-thought reasoning
    #[serde(default = "default_true")]
    pub use_chain_of_thought: bool,

    /// Explain actions before execution
    #[serde(default = "default_true")]
    pub explain_before_action: bool,

    /// Default system prompt template to use
    pub default_system_prompt_template: Option<String>,
}

impl Config {
    /// Load configuration from file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            ConfigError::NotFound(format!("Failed to read config file: {}", e))
        })?;

        let mut config: Config = serde_yaml::from_str(&content)?;

        // Expand environment variables
        config.expand_env_vars()?;

        Ok(config)
    }

    /// Load configuration with priority:
    /// 1. Provided path
    /// 2. Project config (./.promptline/config.yaml)
    /// 3. User config (~/.promptline/config.yaml)
    /// 4. Default configuration
    pub fn load() -> Result<Self> {
        // Try project config
        let project_config = PathBuf::from("./.promptline/config.yaml");
        if project_config.exists() {
            return Self::load_from_file(&project_config);
        }

        // Try user config
        if let Some(mut user_config) = dirs::config_dir() {
            user_config.push("promptline");
            user_config.push("config.yaml");
            if user_config.exists() {
                return Self::load_from_file(&user_config);
            }
        }

        // Use defaults
        Ok(Self::default())
    }

    /// Expand environment variables in configuration
    fn expand_env_vars(&mut self) -> Result<()> {
        for provider in self.models.providers.values_mut() {
            if let Some(ref mut key) = provider.api_key {
                if key.starts_with("${") && key.ends_with("}") {
                    let var_name = &key[2..key.len() - 1];
                    *key = std::env::var(var_name).map_err(|_| {
                        ConfigError::Invalid(format!("Environment variable not found: {}", var_name))
                    })?;
                }
            }
        }
        Ok(())
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            models: ModelConfig::default(),
            tools: ToolPermissions::default(),
            safety: SafetyConfig::default(),
            agent: AgentConfig::default(),
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            default: default_model(),
            providers: HashMap::new(),
        }
    }
}

impl Default for ModelParams {
    fn default() -> Self {
        Self {
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
        }
    }
}

impl Default for ToolPermissions {
    fn default() -> Self {
        Self {
            file_read: PermissionLevel::Allow,
            file_write: PermissionLevel::Ask,
            file_delete: PermissionLevel::Deny,
            shell_execute: PermissionLevel::Ask,
            git_status: PermissionLevel::Ask,
            git_diff: PermissionLevel::Ask,
            git_commit: PermissionLevel::Ask,
            web_get: PermissionLevel::Ask,
            codebase_search: PermissionLevel::Ask,
        }
    }
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            require_approval: true,
            require_diff_preview: true,
            max_iterations: default_max_iterations(),
            dangerous_commands: default_dangerous_commands(),
            allowed_commands: None,
            denied_commands: None,
            protected_patterns: default_protected_patterns(),
            enable_backups: true,
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            default_mode: default_mode(),
            use_chain_of_thought: true,
            explain_before_action: true,
            default_system_prompt_template: None,
        }
    }
}

// Default value functions
fn default_model() -> String {
    // Default model depends on provider
    // For Gemini: "gemini-pro" is most stable
    // For OpenAI: "gpt-3.5-turbo" or "gpt-4"
    // For Ollama: "cogito-2.1:671b" is available on the user's account
    std::env::var("PROMPTLINE_PROVIDER")
        .ok()
        .and_then(|p| match p.as_str() {
            "gemini" => Some("gemini-pro".to_string()),
            "openai" => Some("gpt-3.5-turbo".to_string()),
            _ => None,
        })
        .unwrap_or_else(|| "llama3".to_string())
}

fn default_temperature() -> f32 {
    0.2
}

fn default_max_tokens() -> usize {
    4096
}

fn default_permission() -> PermissionLevel {
    PermissionLevel::Ask
}

fn default_true() -> bool {
    true
}

fn default_max_iterations() -> usize {
    20
}

fn default_mode() -> String {
    "plan".to_string()
}

fn default_dangerous_commands() -> Vec<String> {
    vec![
        "rm -rf /".to_string(),
        "rm -rf *".to_string(),
        "mkfs".to_string(),
        "dd if=".to_string(),
        "format".to_string(),
    ]
}

fn default_protected_patterns() -> Vec<String> {
    vec![
        "**/.env".to_string(),
        "**/.env.*".to_string(),
        "**/*secret*".to_string(),
        "**/*password*".to_string(),
        "**/*.pem".to_string(),
        "**/*.key".to_string(),
        "**/id_rsa".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.models.default, "llama3");
        assert_eq!(config.safety.max_iterations, 20);
        assert_eq!(config.tools.file_read, PermissionLevel::Allow);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: Config = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(config.models.default, deserialized.models.default);
    }
}
