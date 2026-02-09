mod cli;
mod commands;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { force } => commands::init::execute(force).await?,
        Commands::Status => commands::status::execute().await?,
        Commands::AiConfig(cmd) => commands::ai_config::execute(&cmd).await?,
        Commands::SeedValidate => commands::seed_validate::execute()?,
        Commands::Diverge => commands::diverge::execute().await?,
        Commands::DivergeValidate => commands::diverge_validate::execute()?,
        Commands::Converge => commands::converge::execute().await?,
        Commands::ConvergeValidate => commands::converge_validate::execute()?,
        Commands::Skeleton => commands::skeleton::execute().await?,
        Commands::SkeletonValidate => commands::skeleton_validate::execute()?,
    }

    Ok(())
}
