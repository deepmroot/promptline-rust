mod cli;

use cli::{Cli, Commands};
use promptline::prelude::*;
use promptline::{model::openai::OpenAIProvider, tools::*};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging - only show warnings and errors by default
    // Set RUST_LOG=info to see debug logs
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .init();

    // Parse CLI arguments
    let cli = Cli::parse_args();

    // Set verbose logging
    if cli.verbose {
        tracing::info!("Verbose mode enabled");
    }

    // Load configuration
    let mut config = if let Some(config_path) = &cli.config {
        Config::load_from_file(config_path)?
    } else {
        Config::load()?
    };

    // Apply CLI overrides
    if cli.auto_approve {
        config.safety.require_approval = false;
        tracing::warn!("Auto-approve enabled - all actions will execute without confirmation!");
    }

    // Handle subcommands
    match cli.command {
        Some(Commands::Init) => {
            handle_init()?;
        }
        Some(Commands::Doctor) => {
            handle_doctor(&config)?;
        }
        Some(Commands::Plan { task }) => {
            handle_plan(&task, config).await?;
        }
        Some(Commands::Agent { task }) => {
            handle_agent(&task, config).await?;
        }
        Some(Commands::Chat) => {
            handle_chat(config).await?;
        }
        Some(Commands::Edit { file, instruction }) => {
            handle_edit(&file, &instruction, config).await?;
        }
        None => {
            // Direct task execution or start chat mode
            if let Some(task) = cli.task {
                handle_agent(&task, config).await?;
            } else {
                // No command or task, start interactive chat by default
                handle_chat(config).await?;
            }
        }
    }

    Ok(())
}

fn handle_init() -> anyhow::Result<()> {
    println!("🚀 Initializing PromptLine...\n");

    // Check for API key
    let api_key = std::env::var("OPENAI_API_KEY")
        .unwrap_or_else(|_| {
            println!("⚠️  OPENAI_API_KEY environment variable not set");
            String::new()
        });

    if api_key.is_empty() {
        println!("To use OpenAI models, set your API key:");
        println!("  export OPENAI_API_KEY='your-api-key-here'");
    } else {
        println!("✓ OPENAI_API_KEY found");
    }

    // Create default config
    let config = Config::default();

    // Determine config path
    let config_path = if let Some(mut dir) = dirs::config_dir() {
        dir.push("promptline");
        std::fs::create_dir_all(&dir)?;
        dir.push("config.yaml");
        dir
    } else {
        std::path::PathBuf::from(".promptline/config.yaml")
    };

    // Save config
    config.save_to_file(&config_path)?;

    println!("\n✓ Configuration saved to: {}", config_path.display());
    println!("\nPromptLine is ready! Try:");
    println!("  promptline \"list all rust files\"");

    Ok(())
}

fn handle_doctor(config: &Config) -> anyhow::Result<()> {
    println!("🔍 PromptLine Health Check\n");

    println!("✓ Binary version: {}", promptline::VERSION);

    // Check API key
    match std::env::var("OPENAI_API_KEY") {
        Ok(key) if !key.is_empty() => {
            println!("✓ OpenAI API key configured");
        }
        _ => {
            println!("✗ OpenAI API key not found");
            println!("  Set OPENAI_API_KEY environment variable");
        }
    }

    // Check config
    println!("✓ Configuration loaded");
    println!("  Default model: {}", config.models.default);
    println!("  Max iterations: {}", config.safety.max_iterations);
    println!("  Approval required: {}", config.safety.require_approval);

    println!("\n✓ All checks passed!");

    Ok(())
}

async fn handle_plan(task: &str, _config: Config) -> anyhow::Result<()> {
    println!("🤔 Planning mode (read-only)\n");

    // For MVP, planning is just showing what would be done
    println!("Task: {}", task);
    println!("\nThis is a placeholder for plan mode.");
    println!("Phase 1 MVP will implement the agent loop.");

    Ok(())
}

async fn handle_agent(task: &str, config: Config) -> anyhow::Result<()> {
    println!("⚙️  Agent mode\n");

    // Determine provider from environment or config
    let provider = std::env::var("PROMPTLINE_PROVIDER")
        .unwrap_or_else(|_| "openai".to_string());

    // Create model provider based on type
    let model: Box<dyn promptline::model::LanguageModel> = match provider.as_str() {
        "ollama" => {
            let api_key = std::env::var("OLLAMA_API_KEY").ok().or_else(|| {
                config.models.providers.get("ollama")
                    .and_then(|p| p.api_key.clone())
            });
            
            let base_url = config.models.providers.get("ollama")
                .and_then(|p| p.base_url.clone());

            Box::new(promptline::model::ollama::OllamaProvider::new(
                base_url,
                api_key,
                Some(config.models.default.clone())
            ))
        }
        "openai" | _ => {
            // Try environment variable first
            let api_key = std::env::var("OPENAI_API_KEY").ok().or_else(|| {
                // Fallback to config
                config.models.providers.get("openai")
                    .and_then(|p| p.api_key.clone())
            });

            let api_key = api_key.ok_or_else(|| {
                anyhow::anyhow!("OPENAI_API_KEY not set. You can set it via:\n1. Environment variable: OPENAI_API_KEY\n2. Config file: ~/.promptline/config.yaml (under models.providers.openai.api_key)")
            })?;

            Box::new(OpenAIProvider::new(api_key, Some(config.models.default.clone())))
        }
    };

    // Create tool registry
    let mut tools = ToolRegistry::new();
    tools.register(file_ops::FileReadTool::new());
    tools.register(file_ops::FileWriteTool::new());
    tools.register(file_ops::FileListTool::new());
    tools.register(shell::ShellTool::new());
    tools.register(git_ops::GitStatusTool::new());
    tools.register(git_ops::GitDiffTool::new());
    tools.register(git_ops::GitCommitTool::new());
    tools.register(web_ops::WebGetTool::new());
    tools.register(search_ops::CodebaseSearchTool::new());

    // Create agent
    let mut agent = Agent::new(model, tools, config, Vec::new()).await?;

    // Run agent
    println!("Task: {}\n", task);
    let result = agent.run(task).await?;

    // Display result
    println!("\n{}", "=".repeat(60));
    if result.success {
        println!("✓ Task completed successfully");
    } else {
        println!("✗ Task failed");
    }
    println!("Iterations: {}", result.iterations);
    println!("Tools used: {}", result.tool_calls.join(", "));
    println!("{}", "=".repeat(60));
    println!("\nResult:\n{}", result.output);

    Ok(())
}

async fn handle_chat(config: Config) -> anyhow::Result<()> {
    use std::io::{self, Write};
    
    // Clear screen and show banner
    print!("\x1b[2J\x1b[1;1H");
    
    // ASCII Art Banner
    println!("\x1b[1;34m");
    println!(r#"
    ██████╗ ██████╗  ██████╗ ███╗   ███╗██████╗ ████████╗██╗     ██╗███╗   ██╗███████╗
    ██╔══██╗██╔══██╗██╔═══██╗████╗ ████║██╔══██╗╚══██╔══╝██║     ██║████╗  ██║██╔════╝
    ██████╔╝██████╔╝██║   ██║██╔████╔██║██████╔╝   ██║   ██║     ██║██╔██╗ ██║█████╗  
    ██╔═══╝ ██╔══██╗██║   ██║██║╚██╔╝██║██╔═══╝    ██║   ██║     ██║██║╚██╗██║██╔══╝  
    ██║     ██║  ██║╚██████╔╝██║ ╚═╝ ██║██║        ██║   ███████╗██║██║ ╚████║███████╗
    ╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚═╝╚═╝        ╚═╝   ╚══════╝╚═╝╚═╝  ╚═══╝╚══════╝
    "#);
    println!("\x1b[0m");
    
    println!("\x1b[32m    PromptLine v{} (Rust) - Agentic AI CLI\x1b[0m", promptline::VERSION);
    println!("\x1b[90m    Type a command to see the agent in action (e.g., \"refactor main.rs\" or \"explain this code\")\x1b[0m");
    println!();

    // Get provider from environment or use default
    let provider = std::env::var("PROMPTLINE_PROVIDER").unwrap_or_else(|_| "openai".to_string());

    // Create model based on provider
    let model: Box<dyn promptline::model::LanguageModel> = match provider.as_str() {
        "ollama" => {
            let api_key = std::env::var("OLLAMA_API_KEY").ok().or_else(|| {
                config.models.providers.get("ollama")
                    .and_then(|p| p.api_key.clone())
            });
            
            let base_url = config.models.providers.get("ollama")
                .and_then(|p| p.base_url.clone());

            Box::new(promptline::model::ollama::OllamaProvider::new(
                base_url,
                api_key,
                Some(config.models.default.clone())
            ))
        }
        "openai" | _ => {
            let api_key = std::env::var("OPENAI_API_KEY").ok().or_else(|| {
                config.models.providers.get("openai")
                    .and_then(|p| p.api_key.clone())
            });

            let api_key = api_key.ok_or_else(|| {
                anyhow::anyhow!("OPENAI_API_KEY not set")
            })?;

            Box::new(OpenAIProvider::new(api_key, Some(config.models.default.clone())))
        }
    };

    // Register tools
    let mut tools = ToolRegistry::new();
    tools.register(file_ops::FileReadTool::new());
    tools.register(file_ops::FileWriteTool::new());
    tools.register(file_ops::FileListTool::new());
    tools.register(git_ops::GitStatusTool::new());
    tools.register(git_ops::GitDiffTool::new());
    tools.register(web_ops::WebGetTool::new());
    tools.register(search_ops::CodebaseSearchTool::new());

    // Create agent once
    let mut agent = Agent::new(model, tools, config, Vec::new()).await?;

    loop {
        // Print prompt with arrow like in the image
        print!("\n\x1b[32m→ ~ \x1b[0m");
        io::stdout().flush()?;

        // Read user input
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        // Check for exit commands
        if input.is_empty() {
            continue;
        }
        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
            println!("\n👋 Goodbye!");
            break;
        }

        // Run agent with user input
        print!("\n\x1b[1;34mPromptLine:\x1b[0m ");
        io::stdout().flush()?;

        match agent.run(input).await {
            Ok(result) => {
                // Find the last assistant message in the conversation history
                // This contains the actual response, not just "FINISH"
                let last_response = agent.conversation_history
                    .iter()
                    .rev()
                    .find(|msg| msg.role == "assistant")
                    .map(|msg| msg.content.as_str())
                    .unwrap_or(&result.output);
                
                if !last_response.is_empty() && last_response != "FINISH" {
                    // Format the response to strip model identity and clean up
                    let formatted = agent.format_response(last_response);
                    if !formatted.trim().starts_with("Tool '") {
                        // Don't print tool execution messages, only actual responses
                        println!("{}\n", formatted);
                    }
                }
            }
            Err(e) => {
                eprintln!("\n\x1b[1;31mError:\x1b[0m {}\n", e);
            }
        }
    }

    Ok(())
}

async fn handle_edit(
    _file: &std::path::Path,
    _instruction: &str,
    _config: Config,
) -> anyhow::Result<()> {
    println!("📝 Edit mode\n");
    println!("This is a placeholder. Phase 1 will implement file editing.");
    Ok(())
}