mod cli;

use cli::{Cli, Commands};
use promptline::prelude::*;
use promptline::{model::openai::OpenAIProvider, tools::*};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
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
            // Direct task execution
            if let Some(task) = cli.task {
                handle_agent(&task, config).await?;
            } else {
                // No command or task, show help
                println!("PromptLine v{}", promptline::VERSION);
                println!("\nUse --help for usage information");
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
        "openai" | _ => {
            let api_key = std::env::var("OPENAI_API_KEY")
                .map_err(|_| anyhow::anyhow!("OPENAI_API_KEY not set. Run 'promptline init' for setup."))?;
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

async fn handle_chat(_config: Config) -> anyhow::Result<()> {
    println!("💬 Interactive chat mode\n");
    println!("This is a placeholder. Phase 2 will implement REPL mode.");
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