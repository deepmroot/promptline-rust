//! OpenAI API provider implementation

use super::{LanguageModel, Message, ModelInfo, ModelResponse, ToolDefinition, TokenUsage};
use crate::error::{ModelError, Result};
use async_trait::async_trait;

pub struct OpenAIProvider {
    client: async_openai::Client<async_openai::config::OpenAIConfig>,
    model: String,
    temperature: f32,
    max_tokens: usize,
}

impl OpenAIProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        let config = async_openai::config::OpenAIConfig::new().with_api_key(api_key);
        let client = async_openai::Client::with_config(config);

        Self {
            client,
            model: model.unwrap_or_else(|| "gpt-4".to_string()),
            temperature: 0.2,
            max_tokens: 4096,
        }
    }

    pub fn with_params(mut self, temperature: f32, max_tokens: usize) -> Self {
        self.temperature = temperature;
        self.max_tokens = max_tokens;
        self
    }

    fn convert_message(&self, msg: &Message) -> async_openai::types::ChatCompletionRequestMessage {
        use async_openai::types::*;

        match msg.role.as_str() {
            "system" => ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessage {
                    content: msg.content.clone(),
                    role: Role::System,
                    name: None,
                },
            ),
            "user" => ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(msg.content.clone()),
                    role: Role::User,
                    name: None,
                },
            ),
            "assistant" => ChatCompletionRequestMessage::Assistant(
                ChatCompletionRequestAssistantMessage {
                    content: Some(msg.content.clone()),
                    name: None,
                    role: Role::Assistant,
                    #[allow(deprecated)]
                    function_call: None,
                    tool_calls: None,
                },
            ),
            _ => ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(msg.content.clone()),
                    role: Role::User,
                    name: None,
                },
            ),
        }
    }
}

#[async_trait]
impl LanguageModel for OpenAIProvider {
    async fn complete(&self, prompt: &str, system_prompt: Option<&str>) -> Result<ModelResponse> {
        let mut messages = Vec::new();

        if let Some(sys) = system_prompt {
            messages.push(Message::system(sys));
        }

        messages.push(Message::user(prompt));

        self.chat(&messages).await
    }

    async fn chat(&self, messages: &[Message]) -> Result<ModelResponse> {
        use async_openai::types::*;

        let openai_messages: Vec<_> = messages.iter().map(|m| self.convert_message(m)).collect();

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(openai_messages)
            .temperature(self.temperature)
            .max_tokens(self.max_tokens as u16)
            .build()
            .map_err(|e| ModelError::Api(format!("Failed to build request: {}", e)))?;

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| ModelError::Api(format!("API request failed: {}", e)))?;

        let choice = response
            .choices
            .first()
            .ok_or_else(|| ModelError::InvalidResponse("No choices in response".to_string()))?;

        let content = choice
            .message
            .content
            .clone()
            .unwrap_or_default();

        let usage = if let Some(usage) = response.usage {
            TokenUsage {
                prompt_tokens: usage.prompt_tokens as usize,
                completion_tokens: usage.completion_tokens as usize,
                total_tokens: usage.total_tokens as usize,
            }
        } else {
            TokenUsage::default()
        };

        Ok(ModelResponse {
            content,
            model: response.model,
            usage,
            tool_calls: None,
            finish_reason: choice.finish_reason.as_ref().map(|r| format!("{:?}", r)),
        })
    }

    async fn chat_with_tools(
        &self,
        messages: &[Message],
        _tools: &[ToolDefinition],
    ) -> Result<ModelResponse> {
        // For MVP, we'll use regular chat
        // Phase 2 will add proper function calling support
        self.chat(messages).await
    }

    fn model_info(&self) -> ModelInfo {
        ModelInfo {
            provider: "openai".to_string(),
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            supports_tools: true,
            supports_streaming: true,
        }
    }

    fn supports_tools(&self) -> bool {
        self.model.starts_with("gpt-4") || self.model.starts_with("gpt-3.5")
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_provider_creation() {
        let provider = OpenAIProvider::new("test-key".to_string(), Some("gpt-4".to_string()));
        let info = provider.model_info();

        assert_eq!(info.provider, "openai");
        assert_eq!(info.model, "gpt-4");
        assert!(info.supports_tools);
    }

    #[test]
    fn test_message_conversion() {
        let provider = OpenAIProvider::new("test-key".to_string(), None);

        let msg = Message::user("Hello");
        let _converted = provider.convert_message(&msg);

        // Just testing that conversion doesn't panic
    }
}
