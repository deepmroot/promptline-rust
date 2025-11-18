//! Error types for PromptLine

use thiserror::Error;

/// Main error type for PromptLine
#[derive(Debug, Error)]
pub enum PromptLineError {
    #[error("Model error: {0}")]
    Model(#[from] ModelError),

    #[error("Tool error: {0}")]
    Tool(#[from] ToolError),

    #[error("Safety violation: {0}")]
    Safety(String),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Dialoguer error: {0}")]
    Dialoguer(#[from] dialoguer::Error),

    #[error("{0}")]
    Other(String),
}

/// Model-related errors
#[derive(Debug, Error)]
pub enum ModelError {
    #[error("API error: {0}")]
    Api(String),

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Model not available: {0}")]
    NotAvailable(String),

    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),
}

/// Tool execution errors
#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    NotFound(String),

    #[error("Invalid arguments: {0}")]
    InvalidArgs(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Protected file: {0}")]
    ProtectedFile(String),

    #[error("Tool timeout")]
    Timeout,
}

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    NotFound(String),

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

/// Agent execution errors
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Maximum iterations exceeded")]
    MaxIterationsExceeded,

    #[error("User cancelled operation")]
    UserCancelled,

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, PromptLineError>;
