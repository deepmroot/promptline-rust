// PromptLine - An Agentic AI-Powered CLI for Intelligent Code Assistance
//
// Library entry point exposing public API

pub mod agent;
pub mod config;
pub mod context;
pub mod error;
pub mod model;
pub mod prompt;
pub mod safety;
pub mod tools;
pub mod util;

// Re-export commonly used types
pub use agent::Agent;
pub use config::Config;
pub use error::{PromptLineError, Result};
pub use model::{LanguageModel, ModelResponse};
pub use tools::{Tool, ToolResult};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::agent::Agent;
    pub use crate::config::Config;
    pub use crate::error::{PromptLineError, Result};
    pub use crate::model::{LanguageModel, ModelResponse};
    pub use crate::tools::{Tool, ToolResult};
}
