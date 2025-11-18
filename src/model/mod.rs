//! Language model provider interface

use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod gemini;
pub mod openai;

/// Message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

/// Tool definition for function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Model response
#[derive(Debug, Clone)]
pub struct ModelResponse {
    pub content: String,
    pub model: String,
    pub usage: TokenUsage,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub finish_reason: Option<String>,
}

/// Token usage information
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

/// Tool call from model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Language model provider trait
#[async_trait]
pub trait LanguageModel: Send + Sync {
    /// Generate a completion for a prompt
    async fn complete(
        &self,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> Result<ModelResponse>;

    /// Generate a chat completion
    async fn chat(&self, messages: &[Message]) -> Result<ModelResponse>;

    /// Generate a chat completion with tool support
    async fn chat_with_tools(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Result<ModelResponse>;

    /// Get model information
    fn model_info(&self) -> ModelInfo;

    /// Estimate token count for text
    fn estimate_tokens(&self, text: &str) -> usize {
        // Rough estimate: 1 token ≈ 4 characters
        (text.len() + 3) / 4
    }

    /// Check if model supports tool calling
    fn supports_tools(&self) -> bool {
        false
    }

    /// Check if model supports streaming
    fn supports_streaming(&self) -> bool {
        false
    }
}

/// Model information
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub provider: String,
    pub model: String,
    pub max_tokens: usize,
    pub supports_tools: bool,
    pub supports_streaming: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "Hello");

        let sys = Message::system("System prompt");
        assert_eq!(sys.role, "system");
    }

    #[test]
    fn test_token_estimation() {
        struct MockModel;

        #[async_trait]
        impl LanguageModel for MockModel {
            async fn complete(&self, _: &str, _: Option<&str>) -> Result<ModelResponse> {
                unimplemented!()
            }

            async fn chat(&self, _: &[Message]) -> Result<ModelResponse> {
                unimplemented!()
            }

            async fn chat_with_tools(
                &self,
                _: &[Message],
                _: &[ToolDefinition],
            ) -> Result<ModelResponse> {
                unimplemented!()
            }

            fn model_info(&self) -> ModelInfo {
                ModelInfo {
                    provider: "mock".to_string(),
                    model: "test".to_string(),
                    max_tokens: 4096,
                    supports_tools: false,
                    supports_streaming: false,
                }
            }
        }

        let model = MockModel;
        let tokens = model.estimate_tokens("Hello world!");
        assert!(tokens > 0);
        assert!(tokens < 10);
    }
}
