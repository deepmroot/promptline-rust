//! Context management for PromptLine

use crate::error::Result;
use crate::model::Message;
use std::path::PathBuf;
use tokio::fs;

const HISTORY_FILE_NAME: &str = "history.json";

pub struct ContextManager {
    context_dir: PathBuf,
}

impl ContextManager {
    pub async fn new() -> Result<Self> {
        let context_dir = if let Some(mut dir) = dirs::config_dir() {
            dir.push("promptline");
            dir
        } else {
            PathBuf::from(".promptline")
        };

        // Ensure the directory exists
        fs::create_dir_all(&context_dir).await?;

        Ok(Self { context_dir })
    }

    fn history_file_path(&self) -> PathBuf {
        self.context_dir.join(HISTORY_FILE_NAME)
    }

    pub async fn load_history(&self) -> Result<Vec<Message>> {
        let path = self.history_file_path();
        if path.exists() {
            let content = fs::read_to_string(&path).await?;
            let history: Vec<Message> = serde_json::from_str(&content)?;
            Ok(history)
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn save_history(&self, history: &[Message]) -> Result<()> {
        let path = self.history_file_path();
        let content = serde_json::to_string_pretty(history)?;
        fs::write(&path, content).await?;
        Ok(())
    }

    pub async fn clear_history(&self) -> Result<()> {
        let path = self.history_file_path();
        if path.exists() {
            fs::remove_file(&path).await?;
        }
        Ok(())
    }

    pub async fn load_project_context(&self) -> Result<Option<String>> {
        let mut project_context_path = std::env::current_dir()?;
        project_context_path.push(".promptline");
        project_context_path.push("context.md");

        if project_context_path.exists() {
            let content = fs::read_to_string(&project_context_path).await?;
            Ok(Some(content))
        } else {
            Ok(None)
        }
    }

    pub async fn detect_project_type(&self) -> Result<String> {
        let current_dir = std::env::current_dir()?;

        if current_dir.join("Cargo.toml").exists() {
            Ok("Rust".to_string())
        } else if current_dir.join("package.json").exists() {
            Ok("Node.js".to_string())
        } else if current_dir.join("requirements.txt").exists() || current_dir.join("pyproject.toml").exists() {
            Ok("Python".to_string())
        } else if current_dir.join("pom.xml").exists() || current_dir.join("build.gradle").exists() {
            Ok("Java/Gradle".to_string())
        } else {
            Ok("Generic".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_history_persistence() {
        let temp_dir = tempdir().unwrap();
        let manager = ContextManager { context_dir: temp_dir.path().to_path_buf() };

        let messages = vec![
            Message::user("Hello"),
            Message::assistant("Hi there"),
        ];

        // Save history
        manager.save_history(&messages).await.unwrap();

        // Load history
        let loaded_messages = manager.load_history().await.unwrap();
        assert_eq!(messages.len(), loaded_messages.len());
        assert_eq!(messages[0].content, loaded_messages[0].content);
        assert_eq!(messages[1].content, loaded_messages[1].content);

        // Clear history
        manager.clear_history().await.unwrap();
        let cleared_messages = manager.load_history().await.unwrap();
        assert!(cleared_messages.is_empty());
    }
}
