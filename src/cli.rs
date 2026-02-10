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

    /// Validate seed.yaml
    SeedValidate,

    /// Phase 1: Generate divergent architectural proposals
    Diverge {
        /// Skip interactive decision recording
        #[arg(long)]
        skip_decision: bool,
    },

    /// Validate diverge output
    DivergeValidate,

    /// Challenge diverge proposals (human critiques AI)
    DivergeChallenge,

    /// Phase 2: Converge proposals into rules
    Converge {
        /// Skip interactive decision recording
        #[arg(long)]
        skip_decision: bool,
    },

    /// Validate converge output
    ConvergeValidate,

    /// Challenge converge decision (human critiques selection)
    ConvergeChallenge,

    /// Phase 3: Generate project skeleton
    Skeleton {
        /// Skip interactive decision recording
        #[arg(long)]
        skip_decision: bool,
    },

    /// Validate skeleton output
    SkeletonValidate,

    /// Branch workflow commands
    #[command(subcommand)]
    Branch(BranchCommands),

    /// Adopt an existing project into the process workflow
    #[command(subcommand)]
    Adopt(AdoptCommands),

    /// Record a learning
    Learn {
        /// The lesson content
        lesson: String,

        /// Category (default: general)
        #[arg(short, long, default_value = "general")]
        category: String,
    },

    /// Record a friction point
    Friction {
        /// Related branch name
        branch: String,

        /// Description of the friction
        description: String,

        /// Severity: high, medium, low
        #[arg(short, long, default_value = "medium")]
        severity: String,
    },

    /// Phase 5: Freeze invariants and stabilize
    Stabilize {
        /// Skip interactive decision recording
        #[arg(long)]
        skip_decision: bool,
    },

    /// Phase 6: AI-generated retrospective
    Postmortem,

    /// Phase 7: Mark project as complete
    Done,
}

#[derive(Subcommand)]
pub enum AiConfigCommands {
    /// Show current configuration
    Show,
    /// Test AI connection
    Test,
}

#[derive(Subcommand)]
pub enum BranchCommands {
    /// Create a new branch hypothesis
    New {
        /// Branch name
        name: String,
    },
    /// Validate hypothesis and create git branch
    Start {
        /// Branch name
        name: String,
    },
    /// Multi-role AI review
    Review {
        /// Branch name
        name: String,
    },
    /// Adversarial abuse testing
    Abuse {
        /// Branch name
        name: String,
    },
    /// Merge gate checks
    Gate {
        /// Branch name
        name: String,
    },
    /// Mark branch as merged
    Merge {
        /// Branch name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum AdoptCommands {
    /// Scan project directory structure (local, no AI)
    ScanStructure,
    /// Scan and parse dependency manifests (local, no AI)
    ScanDependencies,
    /// Infer coding conventions from source samples (AI-assisted)
    InferConventions,
    /// Analyze git history for architectural decisions (AI-assisted)
    ScanGitHistory {
        /// Maximum number of commits to analyze
        #[arg(short, long, default_value = "200")]
        max_commits: usize,
    },
    /// Identify gaps and undocumented decisions (AI-assisted)
    GapAnalysis,
    /// Run all adopt passes sequentially
    All {
        /// Maximum number of commits for git history scan
        #[arg(short, long, default_value = "200")]
        max_commits: usize,
    },
}
