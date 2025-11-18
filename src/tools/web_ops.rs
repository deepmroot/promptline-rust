//! Web operation tools

use super::{Tool, ToolContext, ToolResult};
use crate::error::{Result, ToolError};
use async_trait::async_trait;

/// Web Get tool
pub struct WebGetTool;

impl WebGetTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WebGetTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for WebGetTool {
    fn name(&self) -> &str {
        "web_get"
    }

    fn description(&self) -> &str {
        "Perform an HTTP GET request to a URL and return the response body. Use for fetching web content, API data, etc."
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to perform the GET request on"
                }
            },
            "required": ["url"]
        })
    }

    fn is_read_only(&self) -> bool {
        true
    }

    async fn execute(&self, args: serde_json::Value, _ctx: &ToolContext, _config: &crate::config::Config) -> Result<ToolResult> {
        let url = args["url"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgs("Missing URL".to_string()))?;

        tracing::info!("Performing web GET request to: {}", url);

        let client = reqwest::Client::new();
        let response = client.get(url).send().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to send request: {}", e)))?;

        let status = response.status();
        let body = response.text().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read response body: {}", e)))?;

        if status.is_success() {
            Ok(ToolResult::success(body)
                .with_metadata("status", serde_json::json!(status.as_u16())))
        } else {
            Err(ToolError::ExecutionFailed(format!("Request failed with status {}: {}", status, body)).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn test_web_get_success() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_string("Hello from mock!"))
            .mount(&mock_server)
            .await;

        let tool = WebGetTool::new();
        let ctx = ToolContext::default();
        let config = crate::config::Config::default();

        let url = format!("{}/test", mock_server.uri());
        let result = tool.execute(serde_json::json!({"url": url}), &ctx, &config).await.unwrap();

        assert!(result.success);
        assert_eq!(result.output, "Hello from mock!");
        assert_eq!(result.metadata["status"], 200);
    }

    #[tokio::test]
    async fn test_web_get_failure() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/error"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        let tool = WebGetTool::new();
        let ctx = ToolContext::default();
        let config = crate::config::Config::default();

        let url = format!("{}/error", mock_server.uri());
        let result = tool.execute(serde_json::json!({"url": url}), &ctx, &config).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Request failed with status 500"));
    }
}
