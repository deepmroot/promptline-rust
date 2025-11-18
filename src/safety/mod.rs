//! Safety validation and approval system

use crate::config::Config;
use crate::error::{Result, ToolError};
use dialoguer::Confirm;
use regex::Regex;

pub struct SafetyValidator {
    config: Config,
    dangerous_patterns: Vec<Regex>,
}

impl SafetyValidator {
    pub fn new(config: Config) -> Result<Self> {
        let dangerous_patterns = config
            .safety
            .dangerous_commands
            .iter()
            .map(|pattern| Regex::new(pattern))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| ToolError::ExecutionFailed(format!("Invalid regex pattern: {}", e)))?;

        Ok(Self {
            config,
            dangerous_patterns,
        })
    }

    /// Validate a command before execution
    pub fn validate_command(&self, command: &str) -> ValidationResult {
        // Check denied commands
        if let Some(denied_commands) = &self.config.safety.denied_commands {
            if denied_commands.iter().any(|c| command.starts_with(c)) {
                return ValidationResult::Denied(format!("Command is in the denied list: {}", command));
            }
        }

        // Check allowed commands
        if let Some(allowed_commands) = &self.config.safety.allowed_commands {
            if !allowed_commands.iter().any(|c| command.starts_with(c)) {
                return ValidationResult::Denied(format!("Command is not in the allowed list: {}", command));
            }
        }

        // Check dangerous patterns
        for pattern in &self.dangerous_patterns {
            if pattern.is_match(command) {
                return ValidationResult::Denied(format!(
                    "Command matches dangerous pattern: {}",
                    pattern.as_str()
                ));
            }
        }

        ValidationResult::RequiresApproval
    }

    /// Request approval from user
    pub fn request_approval(&self, action: &str, details: &str) -> Result<bool> {
        if !self.config.safety.require_approval {
            return Ok(true);
        }

        println!("\n{}", "=".repeat(60));
        println!("⚠️  Approval Required");
        println!("{}", "=".repeat(60));
        println!("Action: {}", action);
        println!("Details:\n{}", details);
        println!("{}", "=".repeat(60));

        let approved = Confirm::new()
            .with_prompt("Approve this action?")
            .default(false)
            .interact()
            .unwrap_or(false);

        Ok(approved)
    }

    /// Check if file is protected
    pub fn is_protected_file(&self, path: &str) -> bool {
        for pattern in &self.config.safety.protected_patterns {
            if let Ok(pattern) = glob::Pattern::new(pattern) {
                if pattern.matches(path) {
                    return true;
                }
            }
        }
        false
    }
}

#[derive(Debug, Clone)]
pub enum ValidationResult {
    Allowed,
    RequiresApproval,
    Denied(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dangerous_command_detection() {
        let mut config = Config::default();
        config.safety.dangerous_commands = vec!["rm -rf".to_string(), "format".to_string()];

        let validator = SafetyValidator::new(config).unwrap();

        assert!(matches!(
            validator.validate_command("rm -rf /tmp/test"),
            ValidationResult::Denied(_)
        ));

        assert!(matches!(
            validator.validate_command("format C:"),
            ValidationResult::Denied(_)
        ));

        assert!(matches!(
            validator.validate_command("ls -la"),
            ValidationResult::RequiresApproval
        ));
    }

    #[test]
    fn test_protected_file_detection() {
        let config = Config::default();
        let validator = SafetyValidator::new(config).unwrap();

        assert!(validator.is_protected_file(".env"));
        assert!(validator.is_protected_file("config/.env"));
        assert!(validator.is_protected_file("secrets.yaml"));
        assert!(validator.is_protected_file("id_rsa"));

        assert!(!validator.is_protected_file("README.md"));
        assert!(!validator.is_protected_file("src/main.rs"));
    }
}
