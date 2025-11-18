//! Prompt template management

use crate::error::{Result, PromptLineError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

const TEMPLATES_DIR_NAME: &str = "templates";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub name: String,
    pub description: String,
    pub template: String,
    pub variables: HashMap<String, String>,
    pub few_shot_examples: Option<Vec<crate::model::Message>>,
}

pub struct TemplateManager {
    templates_dir: PathBuf,
    templates: HashMap<String, PromptTemplate>,
}

impl TemplateManager {
    pub async fn new() -> Result<Self> {
        let templates_dir = if let Some(mut dir) = dirs::config_dir() {
            dir.push("promptline");
            dir.push(TEMPLATES_DIR_NAME);
            dir
        } else {
            PathBuf::from(".promptline").join(TEMPLATES_DIR_NAME)
        };

        fs::create_dir_all(&templates_dir).await?;

        let mut manager = Self {
            templates_dir,
            templates: HashMap::new(),
        };

        manager.load_templates().await?;

        Ok(manager)
    }

    async fn load_templates(&mut self) -> Result<()> {
        self.templates.clear();
        let mut entries = fs::read_dir(&self.templates_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
                let content = fs::read_to_string(&path).await?;
                let template: PromptTemplate = serde_yaml::from_str(&content)
                    .map_err(|e| PromptLineError::Config(crate::error::ConfigError::Invalid(format!("Failed to parse template {}: {}", path.display(), e))))?;
                self.templates.insert(template.name.clone(), template);
            }
        }
        Ok(())
    }

    pub fn get_template(&self, name: &str) -> Option<&PromptTemplate> {
        self.templates.get(name)
    }

    pub fn list_templates(&self) -> Vec<&PromptTemplate> {
        self.templates.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_template_manager() {
        let temp_dir = tempdir().unwrap();
        let templates_path = temp_dir.path().join(TEMPLATES_DIR_NAME);
        fs::create_dir_all(&templates_path).await.unwrap();

        // Create a dummy template file
        let template_content = r#"
name: "test_template"
description: "A test template"
template: "Hello, {{name}}!"
variables:
  name: "world"
        "#;
        fs::write(templates_path.join("test_template.yaml"), template_content).await.unwrap();

        let manager = TemplateManager::new().await.unwrap();
        // Override templates_dir for test
        let mut manager = TemplateManager {
            templates_dir: templates_path,
            templates: HashMap::new(),
        };
        manager.load_templates().await.unwrap();


        let template = manager.get_template("test_template").unwrap();
        assert_eq!(template.name, "test_template");
        assert_eq!(template.template, "Hello, {{name}}!");
    }
}
