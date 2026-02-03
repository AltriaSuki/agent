use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "process")]
#[command(about = "Process CLI - Intelligent Development Process Engine", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize the project and process
    Init {
        /// Force initialization even if already initialized
        #[arg(short, long)]
        force: bool,
    },

    /// Show current process status
    Status,

    /// AI Configuration
    #[command(subcommand)]
    AiConfig(AiConfigCommands),
}

#[derive(Subcommand)]
pub enum AiConfigCommands {
    /// Show current configuration
    Show,
    /// Test AI connection
    Test,
}
