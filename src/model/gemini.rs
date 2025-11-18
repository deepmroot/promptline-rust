//! Google Gemini API provider implementation

use super::{LanguageModel, Message, ModelInfo, ModelResponse, ToolDefinition, TokenUsage};
use crate::error::{ModelError, Result};
use async_trait::async_trait;
use serde_json::json;

pub struct GeminiProvider {
    api_key: String,
    model: String,
    temperature: f32,
    max_tokens: usize,
    client: reqwest::Client,
}

impl GeminiProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            model: model.unwrap_or_else(|| "gemini-pro".to_string()),
            temperature: 0.2,
            max_tokens: 4096,
            client: reqwest::Client::new(),
        }
    }

    pub fn with_params(mut self, temperature: f32, max_tokens: usize) -> Self {
        self.temperature = temperature;
        self.max_tokens = max_tokens;
        self
    }

    fn convert_messages(&self, messages: &[Message]) -> Vec<serde_json::Value> {
        let mut parts = Vec::new();
        
        for msg in messages {
            let role = match msg.role.as_str() {
                "system" => "user", // Gemini doesn't have system role, merge into user
                "assistant" => "model",
                _ => "user",
            };
            
            parts.push(json!({
                "role": role,
                "parts": [{
                    "text": msg.content
                }]
            }));
        }
        
        parts
    }
}

#[async_trait]
impl LanguageModel for GeminiProvider {
    async fn complete(&self, prompt: &str, system_prompt: Option<&str>) -> Result<ModelResponse> {
        let mut messages = Vec::new();
        
        if let Some(sys) = system_prompt {
            messages.push(Message::system(sys));
        }
        
        messages.push(Message::user(prompt));
        
        self.chat(&messages).await
    }

    async fn chat(&self, messages: &[Message]) -> Result<ModelResponse> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let contents = self.convert_messages(messages);
        
        let request_body = json!({
            "contents": contents,
            "generationConfig": {
                "temperature": self.temperature,
                "maxOutputTokens": self.max_tokens,
            }
        });

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ModelError::Api(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ModelError::Api(format!("API error: {}", error_text)).into());
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ModelError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        // Extract content from Gemini response
        let content = response_json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // Extract token usage if available
        let usage = if let Some(usage_metadata) = response_json.get("usageMetadata") {
            TokenUsage {
                prompt_tokens: usage_metadata["promptTokenCount"].as_u64().unwrap_or(0) as usize,
                completion_tokens: usage_metadata["candidatesTokenCount"].as_u64().unwrap_or(0) as usize,
                total_tokens: usage_metadata["totalTokenCount"].as_u64().unwrap_or(0) as usize,
            }
        } else {
            TokenUsage::default()
        };

        Ok(ModelResponse {
            content,
            model: self.model.clone(),
            usage,
            tool_calls: None,
            finish_reason: response_json["candidates"][0]["finishReason"]
                .as_str()
                .map(|s| s.to_string()),
        })
    }

    async fn chat_with_tools(
        &self,
        messages: &[Message],
        _tools: &[ToolDefinition],
    ) -> Result<ModelResponse> {
        // For MVP, use regular chat
        // Gemini supports function calling but we'll implement it in Phase 2
        self.chat(messages).await
    }

    fn model_info(&self) -> ModelInfo {
        ModelInfo {
            provider: "gemini".to_string(),
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            supports_tools: true,
            supports_streaming: false,
        }
    }

    fn supports_tools(&self) -> bool {
        true
    }

    fn supports_streaming(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_provider_creation() {
        let provider = GeminiProvider::new("test-key".to_string(), Some("gemini-pro".to_string()));
        let info = provider.model_info();

        assert_eq!(info.provider, "gemini");
        assert_eq!(info.model, "gemini-pro");
    }

    #[test]
    fn test_message_conversion() {
        let provider = GeminiProvider::new("test-key".to_string(), None);

        let messages = vec![
            Message::system("You are helpful"),
            Message::user("Hello"),
            Message::assistant("Hi there"),
        ];

        let converted = provider.convert_messages(&messages);
        assert_eq!(converted.len(), 3);
    }
}
