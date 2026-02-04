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

    /// Phase 1: Generate divergent architectural proposals
    Diverge,

    /// Validate diverge output
    DivergeValidate,

    /// Phase 2: Converge proposals into rules
    Converge,

    /// Validate converge output
    ConvergeValidate,

    /// Phase 3: Generate project skeleton
    Skeleton,

    /// Validate skeleton output
    SkeletonValidate,
}

#[derive(Subcommand)]
pub enum AiConfigCommands {
    /// Show current configuration
    Show,
    /// Test AI connection
    Test,
}
