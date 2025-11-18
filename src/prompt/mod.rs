//! Prompt engineering and context management

pub mod templates;

// Placeholder for Phase 1 MVP
// Will be expanded in Phase 2 with context providers

pub fn build_system_prompt() -> String {
    r#"You are PromptLine, an AI assistant for coding tasks.

Follow these guidelines:
- Think step-by-step before taking actions
- Use available tools when needed
- Ask for clarification if the task is unclear
- Be concise but thorough in your responses"#
        .to_string()
}
