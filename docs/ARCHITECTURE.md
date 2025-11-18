# PromptLine Architecture

This document describes the system architecture, module organization, and key design decisions for PromptLine.

## Table of Contents

- [Overview](#overview)
- [Design Principles](#design-principles)
- [Module Structure](#module-structure)
- [Core Components](#core-components)
- [Data Flow](#data-flow)
- [Trait System](#trait-system)
- [Error Handling](#error-handling)
- [Concurrency Model](#concurrency-model)

## Overview

PromptLine follows a modular, layered architecture with clear separation of concerns:

```
┌─────────────────────────────────────┐
│         CLI Interface (Clap)        │
├─────────────────────────────────────┤
│         Agent Loop (ReACT)          │
│  ┌───────────┬──────────┬────────┐  │
│  │  Planner  │ Executor │ Memory │  │
│  └───────────┴──────────┴────────┘  │
├─────────────────────────────────────┤
│       Tools Registry & Dispatch      │
│  ┌──────┬───────┬──────┬─────────┐  │
│  │Shell │ File  │ Git  │   Web   │  │
│  └──────┴───────┴──────┴─────────┘  │
├─────────────────────────────────────┤
│       Model Provider Interface       │
│  ┌────────┬────────────┬──────────┐ │
│  │OpenAI  │ Anthropic  │  Local   │ │
│  └────────┴────────────┴──────────┘ │
├─────────────────────────────────────┤
│     Safety & Validation Layer        │
└─────────────────────────────────────┘
```

## Design Principles

### 1. **Modularity**
Each component has a single responsibility and clear interfaces. Tools, models, and agent logic are independently testable.

### 2. **Safety by Default**
All potentially destructive operations require explicit user approval. The safety layer is non-bypassable.

### 3. **Provider Agnostic**
Model providers are abstracted behind traits, allowing seamless switching between OpenAI, Anthropic, or local models.

### 4. **Extensibility**
New tools can be added by implementing the `Tool` trait. Future plugin system will allow dynamic loading.

### 5. **Async First**
Built on Tokio for concurrent operations (API calls, tool execution, streaming).

### 6. **Type Safety**
Leverage Rust's type system to catch errors at compile time. Extensive use of `Result<T, E>` for error handling.

## Module Structure

### Directory Layout

```
src/
├── main.rs                 # Application entry point
├── cli.rs                  # CLI argument parsing and dispatch
├── config.rs               # Configuration management
├── error.rs                # Global error types
│
├── agent/                  # Agent orchestration
│   ├── mod.rs              # Agent struct and main loop
│   ├── planner.rs          # Task planning logic
│   ├── executor.rs         # Action execution
│   └── memory.rs           # Context and conversation state
│
├── model/                  # LLM provider integrations
│   ├── mod.rs              # LanguageModel trait
│   ├── openai.rs           # OpenAI implementation
│   ├── anthropic.rs        # Anthropic implementation
│   └── local_llm.rs        # Local model integration
│
├── tools/                  # Tool implementations
│   ├── mod.rs              # Tool trait and registry
│   ├── shell.rs            # Shell command execution
│   ├── file_edit.rs        # File read/write operations
│   ├── git.rs              # Git operations
│   └── web.rs              # Web requests
│
├── prompt/                 # Prompt engineering
│   ├── mod.rs              # Prompt assembly
│   ├── system.rs           # System prompt templates
│   └── context.rs          # Context injection
│
├── safety/                 # Safety and validation
│   ├── mod.rs              # Safety coordinator
│   ├── validators.rs       # Command/action validators
│   └── approval.rs         # User approval prompts
│
└── util/                   # Shared utilities
    ├── mod.rs
    ├── diff.rs             # Diff generation
    └── formatting.rs       # Output formatting
```

## Core Components

### 1. Agent Loop (ReACT Pattern)

The agent implements a **Reason → Act → Observe** loop:

```rust
pub struct Agent {
    model: Box<dyn LanguageModel>,
    tools: ToolRegistry,
    memory: ConversationMemory,
    safety: SafetyValidator,
    config: Config,
}

impl Agent {
    pub async fn run(&mut self, task: &str) -> Result<AgentResult> {
        loop {
            // REASON: Generate plan/next action
            let reasoning = self.think(task).await?;
            
            // Check termination
            if reasoning.is_complete() {
                break;
            }
            
            // ACT: Execute tool if needed
            if let Some(action) = reasoning.action {
                let result = self.act(action).await?;
                
                // OBSERVE: Update memory with result
                self.observe(result).await?;
            }
        }
        
        Ok(self.memory.summary())
    }
}
```

**Key Responsibilities:**
- Coordinate between model, tools, and memory
- Enforce maximum iteration limits
- Manage conversation state
- Handle errors and retries

### 2. Model Provider Interface

Abstract interface for LLM interactions:

```rust
#[async_trait]
pub trait LanguageModel: Send + Sync {
    async fn complete(
        &self,
        prompt: &str,
        system: Option<&str>
    ) -> Result<ModelResponse>;
    
    async fn chat(
        &self,
        messages: &[Message],
        tools: Option<&[ToolDefinition]>
    ) -> Result<ModelResponse>;
    
    async fn stream_chat(
        &self,
        messages: &[Message]
    ) -> Result<impl Stream<Item = Result<String>>>;
    
    fn supports_tools(&self) -> bool;
    fn estimate_tokens(&self, text: &str) -> usize;
}
```

**Implementations:**
- `OpenAIProvider`: GPT-4, GPT-3.5-turbo
- `AnthropicProvider`: Claude 3 family
- `LocalLLMProvider`: llama.cpp, Ollama

### 3. Tool System

Tools are actions the agent can perform:

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> serde_json::Value;  // JSON Schema
    fn is_read_only(&self) -> bool;
    
    async fn execute(
        &self,
        args: serde_json::Value,
        ctx: &ToolContext
    ) -> Result<ToolResult>;
    
    fn requires_approval(&self) -> ApprovalLevel {
        ApprovalLevel::Ask  // Default: always ask
    }
}
```

**Tool Registry:**
```rust
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        self.tools.insert(tool.name().to_string(), Box::new(tool));
    }
    
    pub async fn execute(
        &self,
        name: &str,
        args: serde_json::Value,
        ctx: &ToolContext
    ) -> Result<ToolResult> {
        let tool = self.tools.get(name)
            .ok_or(ToolError::NotFound)?;
        tool.execute(args, ctx).await
    }
}
```

### 4. Memory & Context Management

Manages conversation history and context:

```rust
pub struct ConversationMemory {
    messages: Vec<Message>,
    context: HashMap<String, String>,
    max_tokens: usize,
}

impl ConversationMemory {
    pub fn add_user_message(&mut self, content: String);
    pub fn add_assistant_message(&mut self, content: String);
    pub fn add_tool_result(&mut self, tool: &str, result: &ToolResult);
    
    pub fn get_context(&self) -> Vec<Message>;
    pub fn summarize_if_needed(&mut self) -> Result<()>;
    
    // Context injection
    pub fn add_file_context(&mut self, path: &str, content: &str);
    pub fn add_env_context(&mut self, key: &str, value: &str);
}
```

### 5. Safety Layer

Multi-layered safety enforcement:

```rust
pub struct SafetyValidator {
    dangerous_patterns: Vec<Regex>,
    allow_list: HashSet<String>,
    deny_list: HashSet<String>,
    require_approval: bool,
}

impl SafetyValidator {
    pub fn validate_command(&self, cmd: &str) -> ValidationResult {
        // Check deny list
        if self.is_dangerous(cmd) {
            return ValidationResult::Denied(reason);
        }
        
        // Check allow list
        if self.is_allowed(cmd) {
            return ValidationResult::Allowed;
        }
        
        // Default: require approval
        ValidationResult::RequiresApproval
    }
    
    pub async fn request_approval(
        &self,
        action: &Action
    ) -> Result<bool> {
        // Display action details
        // Show diff for file edits
        // Prompt user [Y/n/Edit]
    }
}
```

## Data Flow

### Single Command Execution

```
User Input
    ↓
[CLI Parser] → Parse args, load config
    ↓
[Agent.run(task)]
    ↓
[Agent.think()] → Build prompt with context
    ↓
[Model.chat()] → Send to LLM, get response
    ↓
[Parse Response] → Extract action/reasoning
    ↓
[Safety.validate()] → Check if action is safe
    ↓
[Safety.request_approval()] → Ask user if needed
    ↓         ↓ (denied)
    ↓      [Abort]
    ↓ (approved)
[Tool.execute()] → Run the action
    ↓
[Memory.observe()] → Record result
    ↓
[Agent.think()] → Next iteration or complete
    ↓
[Output Result]
```

### Interactive Chat Flow

```
User → "Start chat"
    ↓
[REPL Loop]
    ↓
User Input → Message
    ↓
[Agent.run(message)]
    ↓
... (same as above)
    ↓
Display Response
    ↓
[Wait for next input] → Loop
```

## Trait System

### Key Traits

1. **LanguageModel** - LLM provider abstraction
2. **Tool** - Action execution interface
3. **SafetyValidator** - Validation logic
4. **Config** - Configuration management

### Trait Benefits

- **Testability**: Mock implementations for tests
- **Extensibility**: New providers/tools by implementing traits
- **Compile-time guarantees**: Type safety for all components
- **Dynamic dispatch**: Runtime provider/tool selection

## Error Handling

### Error Types Hierarchy

```rust
#[derive(Debug, thiserror::Error)]
pub enum PromptLineError {
    #[error("Model error: {0}")]
    Model(#[from] ModelError),
    
    #[error("Tool error: {0}")]
    Tool(#[from] ToolError),
    
    #[error("Safety violation: {0}")]
    Safety(String),
    
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, PromptLineError>;
```

### Error Handling Strategy

- Use `Result<T, E>` everywhere
- Convert errors with `?` operator and `From` trait
- Provide context with `anyhow::Context`
- Log errors before propagating
- Display user-friendly messages at CLI level

## Concurrency Model

### Async Runtime: Tokio

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Tokio runtime handles all async operations
}
```

### Concurrent Operations

1. **Parallel Tool Execution**: Future enhancement for independent tools
2. **Streaming Responses**: Display model output as it arrives
3. **Background Tasks**: Log flushing, telemetry (opt-in)

### Thread Safety

- Use `Arc<Mutex<T>>` for shared mutable state
- Prefer message passing with `tokio::sync::mpsc`
- All trait objects are `Send + Sync`

## Configuration

### Config File Structure

```yaml
models:
  default: "gpt-4"
  providers:
    openai:
      api_key: "${OPENAI_API_KEY}"
      
tools:
  file_read: allow
  file_write: ask
  shell_execute: ask
  
safety:
  require_approval: true
  dangerous_commands:
    - "rm -rf"
    - "format"
```

### Config Loading Priority

1. Command-line flags (highest)
2. Environment variables
3. Project config (`./.promptline/config.yaml`)
4. User config (`~/.promptline/config.yaml`)
5. Default values (lowest)

## Testing Strategy

### Unit Tests
- Mock `LanguageModel` for agent tests
- Test each tool in isolation
- Validate safety checks

### Integration Tests
- End-to-end CLI command tests
- Real API calls (gated by feature flag)
- File system operations in temp directories

### Property Tests
- Fuzz command parsing
- Validate safety patterns

## Future Enhancements

1. **Plugin System**: Dynamic loading of external tools
2. **Multi-Agent**: Coordinate multiple specialized agents
3. **Streaming UI**: Real-time progress display
4. **Distributed Execution**: Run tools on remote machines
5. **Semantic Caching**: Cache model responses for common queries

---

**See Also:**
- [Roadmap](ROADMAP.md) for implementation timeline
- [Safety](SAFETY.md) for security details
- [Contributing](CONTRIBUTING.md) for development guidelines
