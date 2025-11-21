//! Permission management system for tool execution
//!
//! Provides persistent permission storage with Once/Always/Never options

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Permission level for tool execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PermissionLevel {
    /// Allow once (this session only)
    Once,
    /// Always allow (persist to config)
    Always,
    /// Never allow (persist to config)
    Never,
    /// Ask every time
    Ask,
}

/// Manages tool execution permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionManager {
    /// Persistent permissions (saved to disk)
    permissions: HashMap<String, PermissionLevel>,
    /// Storage file path
    #[serde(skip)]
    storage_path: PathBuf,
    /// Session-only permissions (not saved)
    #[serde(skip)]
    session_permissions: HashMap<String, PermissionLevel>,
}

impl PermissionManager {
    /// Create a new permission manager
    pub fn new() -> Result<Self> {
        let storage_path = Self::get_storage_path()?;
        
        // Load existing permissions if file exists
        let permissions = if storage_path.exists() {
            let content = std::fs::read_to_string(&storage_path)?;
            serde_yaml::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Ok(Self {
            permissions,
            storage_path,
            session_permissions: HashMap::new(),
        })
    }

    /// Get the storage path for permissions
    fn get_storage_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        
        let config_dir = home.join(".promptline");
        std::fs::create_dir_all(&config_dir)?;
        
        Ok(config_dir.join("permissions.yaml"))
    }

    /// Check if a tool has permission to execute
    pub fn check_permission(&self, tool_name: &str) -> PermissionLevel {
        // Check session permissions first
        if let Some(level) = self.session_permissions.get(tool_name) {
            return level.clone();
        }

        // Check persistent permissions
        if let Some(level) = self.permissions.get(tool_name) {
            return level.clone();
        }

        // Default to Ask
        PermissionLevel::Ask
    }

    /// Set permission for a tool
    pub fn set_permission(&mut self, tool_name: String, level: PermissionLevel) -> Result<()> {
        match level {
            PermissionLevel::Once => {
                // Store in session only
                self.session_permissions.insert(tool_name, level);
            }
            PermissionLevel::Always | PermissionLevel::Never => {
                // Store persistently
                self.permissions.insert(tool_name, level);
                self.save()?;
            }
            PermissionLevel::Ask => {
                // Remove from both
                self.permissions.remove(&tool_name);
                self.session_permissions.remove(&tool_name);
                self.save()?;
            }
        }
        Ok(())
    }

    /// Save permissions to disk
    fn save(&self) -> Result<()> {
        let content = serde_yaml::to_string(&self.permissions)?;
        std::fs::write(&self.storage_path, content)?;
        Ok(())
    }

    /// Get all permissions for display
    pub fn get_all_permissions(&self) -> HashMap<String, PermissionLevel> {
        self.permissions.clone()
    }

    /// Prompt user for permission
    pub fn prompt_for_permission(&mut self, tool_name: &str) -> Result<bool> {
        use dialoguer::{theme::ColorfulTheme, Select};

        println!("\n⚠️  Permission Required: {}", tool_name);
        println!();

        let options = vec![
            "Once     - Allow this time only",
            "Always   - Always allow (recommended)",
            "Never    - Block this tool",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choice")
            .items(&options)
            .default(0)
            .interact()?;

        let level = match selection {
            0 => PermissionLevel::Once,
            1 => PermissionLevel::Always,
            2 => PermissionLevel::Never,
            _ => PermissionLevel::Ask,
        };

        self.set_permission(tool_name.to_string(), level.clone())?;

        if level != PermissionLevel::Never {
            println!("\n✓ Saved: {} = {:?}\n", tool_name, level);
        } else {
            println!("\n✗ Blocked: {}\n", tool_name);
        }

        Ok(level != PermissionLevel::Never)
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            permissions: HashMap::new(),
            storage_path: PathBuf::new(),
            session_permissions: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_levels() {
        let mut manager = PermissionManager::default();
        
        // Default should be Ask
        assert_eq!(manager.check_permission("test_tool"), PermissionLevel::Ask);
        
        // Set to Always
        manager.set_permission("test_tool".to_string(), PermissionLevel::Always).unwrap();
        assert_eq!(manager.check_permission("test_tool"), PermissionLevel::Always);
    }
}
