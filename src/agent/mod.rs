//! Agent orchestration and ReACT loop

use crate::config::Config;
use crate::error::{AgentError, Result};
use crate::model::{LanguageModel, Message};
use crate::tools::{ToolContext, ToolRegistry};
use crate::prompt::templates::TemplateManager;

use serde::{Deserialize, Serialize};

use crate::safety::SafetyValidator;
use crate::permissions::PermissionManager;
use crate::formatter::ResponseFormatter;
use crate::loading::LoadingIndicator;
use std::sync::{Arc, Mutex};

/// Agent for orchestrating LLM interactions and tool execution
pub struct Agent {
    model: Box<dyn LanguageModel>,
    tools: ToolRegistry,
    config: Config,
    safety_validator: SafetyValidator,
    permission_manager: Arc<Mutex<PermissionManager>>,
    template_manager: TemplateManager,
    formatter: ResponseFormatter,
    iteration_count: usize,
    pub conversation_history: Vec<Message>,
}

/// Agent execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub success: bool,
    pub output: String,
    pub iterations: usize,
    pub tool_calls: Vec<String>,
}

impl Agent {
    /// Create a new agent
    pub async fn new(
        model: Box<dyn LanguageModel>,
        tools: ToolRegistry,
        config: Config,
        conversation_history: Vec<Message>,
        permission_manager: Arc<Mutex<PermissionManager>>,
    ) -> Result<Self> {
        let safety_validator = SafetyValidator::new(config.clone())?;
        let template_manager = TemplateManager::new().await?;
        let formatter = ResponseFormatter::new();
        Ok(Self {
            model,
            tools,
            config,
            safety_validator,
            permission_manager,
            template_manager,
            formatter,
            iteration_count: 0,
            conversation_history,
        })
    }

    /// Run the agent on a task
    pub async fn run(&mut self, task: &str) -> Result<AgentResult> {
        tracing::info!("Starting agent run for task: {}", task);

        self.iteration_count = 0;

        // Add system prompt
        let system_prompt = self.build_system_prompt().await;
        self.conversation_history
            .push(Message::system(system_prompt));

        // Add user task
        self.conversation_history
            .push(Message::user(task));

        let mut tool_calls = Vec::new();

        // ReACT loop
        loop {
            self.iteration_count += 1;

            if self.iteration_count > self.config.safety.max_iterations {
                return Err(AgentError::MaxIterationsExceeded.into());
            }

            tracing::debug!("Agent iteration: {}", self.iteration_count);

            // REASON: Get model response with loading indicator
            let mut loading = LoadingIndicator::new();
            loading.start();
            let response = self.model.chat(&self.conversation_history).await?;
            loading.stop().await;

            // Inject file content if mentioned in response
            // DISABLED: This was causing loops where the agent would mention files
            // and then try to auto-read them, leading to infinite loops
            // The agent should explicitly use file_read tool instead
            // self.inject_file_content(&response.content).await?;

            // Check if task is complete
            tracing::info!("Response content: {:?}", response.content);
            if self.is_complete(&response.content) {
                tracing::info!("Task complete detected!");
                return Ok(AgentResult {
                    success: true,
                    output: response.content,
                    iterations: self.iteration_count,
                    tool_calls,
                });
            } else {
                tracing::info!("Task not complete, continuing...");
            }

            // ACT: Parse and execute tool calls
            if let Some(tool_call) = self.parse_tool_call(&response.content) {
                // Clone tool call for later use (displaying output)
                let tool_call_clone = ParsedToolCall {
                    name: tool_call.name.clone(),
                    args: tool_call.args.clone(),
                };

                let result = self.execute_tool_call(tool_call, &mut tool_calls).await?;
                
                // If this was a file write, show the content that was written
                if tool_call_clone.name == "file_write" && result.success {
                    if let Some(content) = tool_call_clone.args.get("content").and_then(|c| c.as_str()) {
                        let path = tool_call_clone.args.get("path").and_then(|p| p.as_str()).unwrap_or("unknown");
                        let ext = std::path::Path::new(path)
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("txt");
                        
                        println!("\n\x1b[1;32mWritten to {}:\x1b[0m", path);
                        println!("```{}", ext);
                        println!("{}", content);
                        println!("```\n");
                    }
                }

                if !result.success {
                    return Ok(result);
                }
            } else {
                // No tool call found, add response to history
                self.conversation_history
                    .push(Message::assistant(response.content));
            }
        }
    }

    async fn execute_tool_call(&mut self, tool_call: ParsedToolCall, tool_calls: &mut Vec<String>) -> Result<AgentResult> {
        tracing::info!("Executing tool: {}", tool_call.name);

        // Check permission using the new permission manager
        use crate::permissions::PermissionLevel;
        
        // Lock the permission manager
        // We need to be careful about holding the lock across await points
        // prompt_for_permission is synchronous (uses dialoguer), so it's fine
        let permission_level = {
            let pm = self.permission_manager.lock().unwrap();
            pm.check_permission(&tool_call.name)
        };
        
        match permission_level {
            PermissionLevel::Never => {
                return Err(crate::error::ToolError::PermissionDenied(tool_call.name).into());
            }
            PermissionLevel::Ask => {
                // Prompt user for permission
                // We need to lock again for mutation
                let allowed = {
                    let mut pm = self.permission_manager.lock().unwrap();
                    pm.prompt_for_permission(&tool_call.name)
                        .map_err(|e| crate::error::PromptLineError::Other(e.to_string()))?
                };
                
                if !allowed {
                    return Ok(AgentResult {
                        success: false,
                        output: "Permission denied.".to_string(),
                        iterations: self.iteration_count,
                        tool_calls: tool_calls.clone(),
                    });
                }
            }
            PermissionLevel::Once | PermissionLevel::Always => {
                // Permission already granted
            }
        }

        // Validate command
        let command_str = format!("{} {}", tool_call.name, tool_call.args);
        match self.safety_validator.validate_command(&command_str) {
            crate::safety::ValidationResult::Denied(reason) => {
                return Err(crate::error::PromptLineError::Safety(reason));
            }
            crate::safety::ValidationResult::RequiresApproval => {
                // Already handled by permission check
            }
            crate::safety::ValidationResult::Allowed => {
                tracing::debug!("Command is allowed by safety validator");
            }
        }

        tool_calls.push(tool_call.name.clone());

        let mut ctx = ToolContext::default();
        if let Ok(output) = tokio::process::Command::new("git")
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("HEAD")
            .output()
            .await
        {
            if output.status.success() {
                ctx.git_branch = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }
        // Execute the tool
        let result = self
            .tools
            .execute(&tool_call.name, tool_call.args, &ctx, &self.config)
            .await?;

        // Show formatted result to user
        let result_text = if result.success {
            &result.output
        } else {
            result.error.as_ref().unwrap_or(&result.output)
        };
        
        let formatted_output = self.formatter.format_tool_result(&tool_call.name, result_text);
        print!("{}", formatted_output);
        use std::io::Write;
        std::io::stdout().flush().ok();

        // OBSERVE: Add result to conversation (for the model)
        let observation = format!(
            "Tool '{}' result: {}",
            tool_call.name,
            result_text
        );

        self.conversation_history
            .push(Message::assistant(observation));

        Ok(AgentResult {
            success: true,
            output: "".to_string(),
            iterations: self.iteration_count,
            tool_calls: tool_calls.clone(),
        })
    }

    async fn build_system_prompt(&self) -> String {
        let tool_descriptions: Vec<String> = self
            .tools
            .definitions()
            .iter()
            .map(|def| {
                format!(
                    "- {}: {}",
                    def["name"].as_str().unwrap_or("unknown"),
                    def["description"].as_str().unwrap_or("")
                )
            })
            .collect();

        let current_dir = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        let git_branch = if let Ok(output) = std::process::Command::new("git")
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("HEAD")
            .output()
        {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        } else {
            None
        };

        let git_info = if let Some(branch) = git_branch {
            format!("You are currently on git branch: {}", branch)
        } else {
            "You are not in a git repository or branch could not be determined.".to_string()
        };

        let base_prompt = if let Some(template_name) = &self.config.agent.default_system_prompt_template {
            if let Some(template) = self.template_manager.get_template(template_name) {
                let mut prompt = template.template.clone();
                if let Some(examples) = &template.few_shot_examples {
                    for example in examples {
                        prompt.push_str(&format!("\n\n{}: {}", example.role, example.content));
                    }
                }
                prompt
            } else {
                tracing::warn!("System prompt template '{}' not found. Using default prompt.", template_name);
                self.default_system_prompt()
            }
        } else {
            self.default_system_prompt()
        };

        let project_context = match crate::context::ContextManager::new().await {
            Ok(context_manager) => context_manager.load_project_context().await.ok().flatten(),
            Err(e) => {
                tracing::warn!("Failed to load project context: {}", e);
                None
            }
        };

        let project_type = match crate::context::ContextManager::new().await {
            Ok(context_manager) => context_manager.detect_project_type().await.unwrap_or_else(|e| {
                tracing::warn!("Failed to detect project type: {}", e);
                "Generic".to_string()
            }),
            Err(e) => {
                tracing::warn!("Failed to create context manager: {}", e);
                "Generic".to_string()
            }
        };

        let mut final_prompt = String::new();
        if let Some(context) = project_context {
            final_prompt.push_str(&format!("Project Context:\n```\n{}\n```\n\n", context));
        }
        final_prompt.push_str(&format!(
            r###"{}

Current working directory: {}
Current project type: {}
{}

IDENTITY & BRANDING:
You are PromptLine, an advanced AI-powered CLI agent.
- You are NOT "Cogito", "Claude", "GPT", or any other model.
- You are a helpful, professional, and witty engineering assistant.
- If asked about your identity, always reply that you are PromptLine.
- Do not apologize excessively. Be concise and action-oriented.

OUTPUT FORMAT:
- Use Markdown for all responses.
- Use emojis sparingly but effectively to convey status (e.g., 🔍 for search, 📝 for writing).
- Keep responses clean and structured.

You can use the following tools:
{}

To use a tool, output JSON in this format:
{{"tool": "tool_name", "args": {{"arg": "value"}}}}

When you've completed the task, respond with: FINISH

Always explain your reasoning before taking an action."###,
            base_prompt,
            current_dir,
            project_type,
            git_info,
            tool_descriptions.join("\n")
        ));
        final_prompt
    }

    fn default_system_prompt(&self) -> String {
        r###"You are PromptLine, an AI coding assistant built to help developers with their tasks.

IDENTITY:
- Your name is PromptLine (not Cogito, Claude, GPT, or any other model name)
- You are a professional, helpful coding assistant
- Never mention your underlying model or AI provider

IMPORTANT GUIDELINES:
- For simple greetings (hi, hello, hey) or casual conversation, just respond naturally WITHOUT using any tools, then say FINISH
- Only use tools when the user asks you to DO something specific (read a file, search code, list files, etc.)
- When you use a tool, explain what you're doing briefly
- ALWAYS end your response with "FINISH" on a new line when done
- Be concise and professional in your responses

AVAILABLE TOOLS:
- file_read: Read file contents
- file_write: Write to a file
- file_list: List directory contents
- shell_execute: Run shell commands (use this to run scripts, e.g., 'node app.js', 'cargo run')
- git_status: Check git status
- git_diff: Show git diff
- web_get: Fetch web content
- codebase_search: Search code

TOOL USAGE FORMAT:
When you need to use a tool, respond with JSON:
{"tool": "tool_name", "args": {"arg_name": "value"}}

Example for running a command:
{"tool": "shell_execute", "args": {"command": "node hello.js"}}

Remember:
1. If the user asks to "run" something, USE `shell_execute`. Do not just explain how to run it.
2. If you write a file that needs to be run, you can immediately follow up with `shell_execute` to run it.
3. **NEW PROJECT RULE**: If asked to create a new project, app, or website, **ALWAYS** create a new directory for it first using `shell_execute` (e.g., `mkdir my-app`). Then write files into that directory.
   - **EXCEPTION**: If the user explicitly asks to add to or modify the *current* project, or if you are already inside the project directory (e.g., you see `package.json` or `Cargo.toml`), do NOT create a new folder. Work in the current directory.
4. Don't use tools for simple conversation - just chat naturally!"###.to_string()
    }

    fn parse_tool_call(&self, content: &str) -> Option<ParsedToolCall> {
        // Try to find JSON tool call in content
        if let Some(start) = content.find('{') {
            if let Some(end) = content.rfind('}') {
                let json_str = &content[start..=end];
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let (Some(tool), Some(args)) = (value.get("tool").and_then(|v| v.as_str()), value.get("args")) {
                        return Some(ParsedToolCall {
                            name: tool.to_string(),
                            args: args.clone(),
                        });
                    }
                }
            }
        }
        None
    }

    fn is_complete(&self, content: &str) -> bool {
        content.trim().ends_with("FINISH") || content.contains("task is complete")
    }

    /// Format a response using the formatter (strip model identity, clean up)
    pub fn format_response(&self, content: &str) -> String {
        self.formatter.format_response(content)
    }


}

#[derive(Debug)]
struct ParsedToolCall {
    name: String,
    args: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelInfo, ModelResponse, TokenUsage};
    use async_trait::async_trait;

    struct MockModel {
        responses: Vec<String>,
        call_count: std::sync::Arc<std::sync::Mutex<usize>>,
    }

    #[async_trait]
    impl LanguageModel for MockModel {
        async fn complete(&self, _: &str, _: Option<&str>) -> Result<ModelResponse> {
            unimplemented!()
        }

        async fn chat(&self, _: &[Message]) -> Result<ModelResponse> {
            let mut count = self.call_count.lock().unwrap();
            let response = self.responses[*count].clone();
            *count += 1;

            Ok(ModelResponse {
                content: response,
                model: "mock".to_string(),
                usage: TokenUsage::default(),
                tool_calls: None,
                finish_reason: Some("stop".to_string()),
            })
        }

        async fn chat_with_tools(
            &self,
            messages: &[crate::model::Message],
            _: &[crate::model::ToolDefinition],
        ) -> Result<ModelResponse> {
            self.chat(messages).await
        }

        fn model_info(&self) -> ModelInfo {
            ModelInfo {
                provider: "mock".to_string(),
                model: "test".to_string(),
                max_tokens: 4096,
                supports_tools: false,
                supports_streaming: false,
            }
        }
    }

    #[tokio::test]
    async fn test_agent_simple_task() {
        let model = Box::new(MockModel {
            responses: vec!["I will list the files. {\"tool\": \"file_list\", \"args\": {}}".to_string(), "FINISH".to_string()],
            call_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
        });

        let mut tools = ToolRegistry::new();
        tools.register(crate::tools::file_ops::FileListTool::new());

        let mut config = Config::default();
        config.safety.require_approval = false;
        let permission_manager = std::sync::Arc::new(std::sync::Mutex::new(crate::permissions::PermissionManager::new().unwrap()));
        let mut agent = Agent::new(model, tools, config, Vec::new(), permission_manager).await.unwrap();

        let result = agent.run("List the files").await.unwrap();

        assert!(result.success);
        assert_eq!(result.iterations, 2);
        assert_eq!(result.tool_calls.len(), 1);
    }
}