//! Slash command handling
//!
//! Provides commands for configuration and control

use crate::config::Config;
use crate::permissions::PermissionManager;
use anyhow::Result;

/// Slash command types
#[derive(Debug, Clone, PartialEq)]
pub enum SlashCommand {
    Help,
    Settings,
    Clear,
    Status,
    Model,
    Permissions,
    Quit,
    Version,
}

/// Command handler
pub struct CommandHandler {
    config: Config,
    permissions: PermissionManager,
}

impl CommandHandler {
    /// Create a new command handler
    pub fn new(config: Config, permissions: PermissionManager) -> Self {
        Self {
            config,
            permissions,
        }
    }

    /// Parse a slash command from input
    pub fn parse(input: &str) -> Option<SlashCommand> {
        let trimmed = input.trim();
        if !trimmed.starts_with('/') {
            return None;
        }

        match trimmed.to_lowercase().as_str() {
            "/help" | "/h" => Some(SlashCommand::Help),
            "/settings" | "/config" => Some(SlashCommand::Settings),
            "/clear" | "/new" => Some(SlashCommand::Clear),
            "/status" => Some(SlashCommand::Status),
            "/model" => Some(SlashCommand::Model),
            "/permissions" | "/perms" => Some(SlashCommand::Permissions),
            "/quit" | "/exit" | "/q" => Some(SlashCommand::Quit),
            "/version" | "/v" => Some(SlashCommand::Version),
            _ => None,
        }
    }

    /// Execute a slash command
    pub fn execute(&self, command: SlashCommand) -> Result<String> {
        match command {
            SlashCommand::Help => Ok(self.help()),
            SlashCommand::Settings => Ok(self.settings()),
            SlashCommand::Clear => Ok("Session cleared.".to_string()),
            SlashCommand::Status => Ok(self.status()),
            SlashCommand::Model => Ok(self.model_info()),
            SlashCommand::Permissions => Ok(self.permissions_info()),
            SlashCommand::Quit => Ok("Goodbye! 👋".to_string()),
            SlashCommand::Version => Ok(format!("PromptLine v{}", crate::VERSION)),
        }
    }

    /// Show help message
    fn help(&self) -> String {
        r#"
⚙️  PromptLine Commands

Available slash commands:
  /help         Show this help message
  /settings     Configure permissions and preferences
  /clear        Start new session (clear history)
  /status       Show current configuration
  /model        Show model information
  /permissions  Manage tool permissions
  /quit         Exit PromptLine
  /version      Show version info

Aliases:
  /h → /help
  /q → /quit
  /v → /version
  /perms → /permissions
"#.to_string()
    }

    /// Show settings
    fn settings(&self) -> String {
        let perms = self.permissions.get_all_permissions();
        let mut output = String::from("\n⚙️  PromptLine Settings\n\nPermissions:\n");

        if perms.is_empty() {
            output.push_str("  (No custom permissions set)\n");
        } else {
            for (tool, level) in perms {
                output.push_str(&format!("  • {}: {:?}\n", tool, level));
            }
        }

        output.push_str(&format!("\nProvider: {}\n", self.config.models.default));
        output.push_str("\nType /help for available commands\n");

        output
    }

    /// Show status
    fn status(&self) -> String {
        format!(
            "\n⚙️  Status\n\nProvider: {}\nVersion: {}\n",
            self.config.models.default,
            crate::VERSION
        )
    }

    /// Show model info
    fn model_info(&self) -> String {
        format!(
            "\n🤖 Model Information\n\nProvider: {}\nDefault Model: {}\n",
            "Ollama", // TODO: Get from config
            self.config.models.default
        )
    }

    /// Show permissions info
    fn permissions_info(&self) -> String {
        let perms = self.permissions.get_all_permissions();
        let mut output = String::from("\n🔐 Tool Permissions\n\n");

        if perms.is_empty() {
            output.push_str("No custom permissions set. All tools will prompt for permission.\n");
        } else {
            for (tool, level) in perms {
                let icon = match level {
                    crate::permissions::PermissionLevel::Always => "✓",
                    crate::permissions::PermissionLevel::Never => "✗",
                    _ => "?",
                };
                output.push_str(&format!("  {} {}: {:?}\n", icon, tool, level));
            }
        }

        output.push_str("\nUse /settings to configure permissions\n");

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_commands() {
        assert_eq!(CommandHandler::parse("/help"), Some(SlashCommand::Help));
        assert_eq!(CommandHandler::parse("/quit"), Some(SlashCommand::Quit));
        assert_eq!(CommandHandler::parse("/h"), Some(SlashCommand::Help));
        assert_eq!(CommandHandler::parse("not a command"), None);
    }
}
