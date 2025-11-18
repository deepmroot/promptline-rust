//! CLI interface

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "promptline")]
#[command(version, about = "An Agentic AI-Powered CLI for Intelligent Code Assistance", long_about = None)]
pub struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Model provider to use (openai, anthropic, local)
    #[arg(short = 'p', long)]
    pub provider: Option<String>,

    /// Specific model to use (e.g., gpt-4, gpt-3.5-turbo)
    #[arg(short = 'm', long)]
    pub model: Option<String>,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Auto-approve all actions (use with caution!)
    #[arg(long)]
    pub auto_approve: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Direct task to execute (if no subcommand provided)
    #[arg(value_name = "TASK")]
    pub task: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run agent in plan mode (read-only analysis)
    Plan {
        /// Task to plan
        task: String,
    },

    /// Run agent in execution mode
    Agent {
        /// Task to execute
        task: String,
    },

    /// Start interactive chat mode
    Chat,

    /// Edit a file with AI assistance
    Edit {
        /// File to edit
        file: PathBuf,

        /// Description of changes
        instruction: String,
    },

    /// Initialize configuration
    Init,

    /// Check installation and configuration
    Doctor,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
