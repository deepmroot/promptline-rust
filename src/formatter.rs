//! Output formatting for clean, structured responses
//!
//! Provides formatting utilities for tool results and agent responses

use std::collections::HashMap;

/// Formats agent responses and tool outputs
pub struct ResponseFormatter {
    /// Tool icons for visual indicators
    tool_icons: HashMap<&'static str, &'static str>,
}

impl ResponseFormatter {
    /// Create a new formatter
    pub fn new() -> Self {
        let mut tool_icons = HashMap::new();
        tool_icons.insert("file_read", "📄");
        tool_icons.insert("file_write", "✏️");
        tool_icons.insert("file_list", "📁");
        tool_icons.insert("git_status", "📊");
        tool_icons.insert("git_diff", "🔍");
        tool_icons.insert("web_get", "🌐");
        tool_icons.insert("codebase_search", "🔍");

        Self { tool_icons }
    }

    /// Format a tool result with structured output
    pub fn format_tool_result(&self, tool_name: &str, result: &str) -> String {
        let icon = self.tool_icons.get(tool_name).unwrap_or(&"⚙️");
        
        match tool_name {
            "file_list" => self.format_file_list(result, icon),
            "file_read" => self.format_file_read(result, icon),
            "codebase_search" => self.format_search_result(result, icon),
            _ => format!("\n{} {}\n   ↳ {}\n", icon, tool_name.to_uppercase(), result),
        }
    }

    /// Format file list output
    fn format_file_list(&self, result: &str, icon: &str) -> String {
        // Parse the result and format nicely
        if result.contains("Found") {
            format!("\n{} DIRECTORY LISTING\n   ↳ {}\n", icon, result)
        } else {
            format!("\n{} {}\n", icon, result)
        }
    }

    /// Format file read output
    fn format_file_read(&self, result: &str, icon: &str) -> String {
        format!("\n{} FILE CONTENT\n\n{}\n", icon, result)
    }

    /// Format search results
    fn format_search_result(&self, result: &str, icon: &str) -> String {
        format!("\n{} SEARCH RESULTS\n   ↳ {}\n", icon, result)
    }

    /// Strip model self-identification from responses
    pub fn strip_model_identity(&self, content: &str) -> String {
        let patterns = vec![
            "I'm Cogito",
            "I am Cogito",
            "I'm an AI assistant created by Deep Cogito",
            "I am an AI assistant created by Deep Cogito",
            "I'm Claude",
            "I am Claude",
            "I'm GPT",
            "I am GPT",
            "I'm ChatGPT",
            "I am ChatGPT",
        ];

        let mut result = content.to_string();
        for pattern in patterns {
            result = result.replace(pattern, "I'm PromptLine");
        }

        result
    }

    /// Format a complete response with proper structure
    pub fn format_response(&self, content: &str) -> String {
        let cleaned = self.strip_model_identity(content);
        
        // Add proper spacing and structure
        cleaned
            .lines()
            .map(|line| {
                if line.trim().is_empty() {
                    String::new()
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format a simple greeting response
    pub fn format_greeting(&self, _input: &str) -> String {
        "Hey! 👋 Ready to help with your code. What are we building today?".to_string()
    }
}

impl Default for ResponseFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_identity() {
        let formatter = ResponseFormatter::new();
        let input = "Hello! I'm Cogito, an AI assistant. How can I help?";
        let output = formatter.strip_model_identity(input);
        assert!(output.contains("I'm PromptLine"));
        assert!(!output.contains("Cogito"));
    }
}
