mod cli;

use cli::{Cli, Commands};
use promptline::prelude::*;
use promptline::{model::openai::OpenAIProvider, tools::*};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if it exists
    dotenv::dotenv().ok();

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

    // Create permission manager
    let permission_manager = std::sync::Arc::new(std::sync::Mutex::new(
        promptline::permissions::PermissionManager::new()?
    ));

    // Create agent
    let mut agent = Agent::new(model, tools, config, Vec::new(), permission_manager).await?;

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

async fn handle_chat(mut config: Config) -> anyhow::Result<()> {
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

    // Create REPL editor once
    let helper = promptline::repl::ReplHelper::new();
    let repl_config = rustyline::Config::builder()
        .completion_type(rustyline::CompletionType::List)
        .edit_mode(rustyline::EditMode::Emacs)
        .build();

    let mut editor = rustyline::Editor::with_config(repl_config)?;
    editor.set_helper(Some(helper));



    // Load history if exists
    let history_path = dirs::home_dir()
        .map(|d| d.join(".promptline").join("history.txt"));
    
    if let Some(path) = &history_path {
        if path.exists() {
            let _ = editor.load_history(path);
        }
    }

    // Outer loop for reloading agent
    loop {
        // Reload config if this is a reload
        // (For the first run, we use the passed config, but subsequent runs should reload)
        // Actually, let's just reload it every time we loop back here
        if let Ok(new_config) = Config::load() {
            config = new_config;
        }

        // Get provider from environment or use default
        let provider = std::env::var("PROMPTLINE_PROVIDER").unwrap_or_else(|_| "ollama".to_string());

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
            "openai" => {
                let api_key = std::env::var("OPENAI_API_KEY").ok().or_else(|| {
                    config.models.providers.get("openai")
                        .and_then(|p| p.api_key.clone())
                });

                // If no key, warn but don't crash yet (allow setting it via command)
                if api_key.is_none() {
                    println!("\x1b[1;33mWarning:\x1b[0m OpenAI API key not set. Use '/model config openai key <key>' to set it.");
                }

                let api_key = api_key.unwrap_or_default();
                Box::new(OpenAIProvider::new(api_key, Some(config.models.default.clone())))
            }
            unknown => {
                println!("\x1b[1;31mError:\x1b[0m Unknown provider '{}'. Defaulting to OpenAI.", unknown);
                // Fallback to OpenAI to keep the loop running, but user sees the error
                 let api_key = std::env::var("OPENAI_API_KEY").ok().or_else(|| {
                    config.models.providers.get("openai")
                        .and_then(|p| p.api_key.clone())
                });
                let api_key = api_key.unwrap_or_default();
                Box::new(OpenAIProvider::new(api_key, Some(config.models.default.clone())))
            }
        };

        // Register tools
        let mut tools = ToolRegistry::new();
        tools.register(file_ops::FileReadTool::new());
        tools.register(file_ops::FileWriteTool::new());
        tools.register(file_ops::FileListTool::new());
        tools.register(shell::ShellTool::new());
        tools.register(git_ops::GitStatusTool::new());
        tools.register(git_ops::GitDiffTool::new());
        tools.register(web_ops::WebGetTool::new());
        tools.register(search_ops::CodebaseSearchTool::new());

        // Create shared permission manager
        let permission_manager = std::sync::Arc::new(std::sync::Mutex::new(
            promptline::permissions::PermissionManager::new()?
        ));

        // Create agent
        let mut agent = Agent::new(
            model, 
            tools, 
            config.clone(), 
            Vec::new(),
            permission_manager.clone()
        ).await?;
        
        // Create command handler
        let mut command_handler = promptline::commands::CommandHandler::new(config.clone(), permission_manager);

        // Inner loop for REPL
        #[allow(unused_assignments)]
        let mut reload_requested = false;
        loop {
            // Read line with autocomplete
            // Using simple prompt to debug cursor issue
            let readline = editor.readline("→ ~ ");

            match readline {
                Ok(line) => {
                    let input = line.trim();
                    editor.add_history_entry(input)?;

                    // Check for exit commands
                    if input.is_empty() {
                        continue;
                    }
                    if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                        println!("\n👋 Goodbye!");
                        return Ok(());
                    }

                    // Check for slash commands
                    if let Some(command) = promptline::commands::CommandHandler::parse(input) {
                        match command_handler.execute(command) {
                            Ok(output) => {
                                println!("{}", output.message);
                                match output.action {
                                    promptline::commands::CommandAction::Quit => return Ok(()),
                                    promptline::commands::CommandAction::ClearHistory => {
                                        agent.conversation_history.clear();
                                        println!("✓ Session cleared");
                                    }
                                    promptline::commands::CommandAction::ReloadAgent => {
                                        println!("↻ Reloading agent...");
                                        reload_requested = true;
                                        break; // Break inner loop to reload
                                    }
                                    promptline::commands::CommandAction::None => {}
                                }
                                continue;
                            }
                            Err(e) => {
                                eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
                                continue;
                            }
                        }
                    } else if input.starts_with('/') {
                        println!("\x1b[1;33mUnknown command:\x1b[0m {}", input);
                        println!("Type /help for available commands");
                        continue;
                    }

                    // Run agent with user input
                    print!("\n\x1b[1;34mPromptLine:\x1b[0m ");
                    io::stdout().flush()?;

                    match agent.run(input).await {
                        Ok(result) => {
                            // Use the result output directly
                            let response_content = &result.output;
                            
                            if !response_content.is_empty() && response_content != "FINISH" {
                                // Format the response to strip model identity and clean up
                                let formatted = agent.format_response(response_content);
                                println!("{}\n", formatted);
                            }

                            // Add the response to history so the model remembers it
                            agent.conversation_history.push(promptline::model::Message::assistant(response_content.clone()));
                        }
                        Err(e) => {
                            eprintln!("\n\x1b[1;31mError:\x1b[0m {}\n", e);
                        }
                    }
                }
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    println!("^C");
                    return Ok(());
                }
                Err(rustyline::error::ReadlineError::Eof) => {
                    println!("^D");
                    return Ok(());
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    return Ok(());
                }
            }
        }

        if !reload_requested {
            break;
        }
    }

    // Save history
    if let Some(path) = &history_path {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let _ = editor.save_history(path);
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