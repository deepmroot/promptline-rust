use crate::error::{Result, ModelError};
use crate::model::{LanguageModel, ModelResponse};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

pub struct OllamaProvider {
    client: Client,
    base_url: String,
    api_key: Option<String>,
    default_model: String,
}

impl OllamaProvider {
    pub fn new(base_url: Option<String>, api_key: Option<String>, default_model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            api_key,
            default_model: default_model.unwrap_or_else(|| "llama2".to_string()),
        }
    }
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: OllamaMessage,
    #[allow(dead_code)]
    done: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaMessage {
    content: String,
}

#[async_trait]
impl LanguageModel for OllamaProvider {
    async fn chat(&self, messages: &[crate::model::Message]) -> Result<ModelResponse> {
        let url = format!("{}/api/chat", self.base_url);
        
        // Debug logging
        if let Some(key) = &self.api_key {
            let masked_key = if key.len() > 4 {
                format!("{}...", &key[..4])
            } else {
                "***".to_string()
            };
            tracing::info!("Ollama Chat: URL={}, Key={}, Model={}", url, masked_key, self.default_model);
        } else {
            tracing::info!("Ollama Chat: URL={}, Key=None, Model={}", url, self.default_model);
        }

        let mut ollama_messages = Vec::new();
        for msg in messages {
            ollama_messages.push(json!({
                "role": msg.role,
                "content": msg.content
            }));
        }

        let mut request = self.client.post(&url)
            .json(&json!({
                "model": self.default_model,
                "messages": ollama_messages,
                "stream": false
            }));

        if let Some(key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        let response = request.send().await.map_err(ModelError::Request)?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ModelError::Api(format!("Ollama API error: {}", error_text)).into());
        }

        let ollama_resp: OllamaResponse = response.json().await.map_err(ModelError::Request)?;

        Ok(ModelResponse {
            content: ollama_resp.message.content,
            model: self.default_model.clone(),
            usage: crate::model::TokenUsage::default(), // Ollama usage not parsed yet
            tool_calls: None,
            finish_reason: Some("stop".to_string()),
        })
    }

    async fn complete(&self, prompt: &str, system_prompt: Option<&str>) -> Result<ModelResponse> {
        let mut messages = Vec::new();
        if let Some(sys) = system_prompt {
            messages.push(crate::model::Message::system(sys));
        }
        messages.push(crate::model::Message::user(prompt));
        self.chat(&messages).await
    }

    async fn chat_with_tools(
        &self,
        messages: &[crate::model::Message],
        _tools: &[crate::model::ToolDefinition],
    ) -> Result<ModelResponse> {
        // For now, just ignore tools and chat normally
        self.chat(messages).await
    }

    fn model_info(&self) -> crate::model::ModelInfo {
        crate::model::ModelInfo {
            provider: "ollama".to_string(),
            model: self.default_model.clone(),
            max_tokens: 4096, // Default assumption
            supports_tools: false,
            supports_streaming: false,
        }
    }
}
