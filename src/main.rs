mod cli;
mod commands;
mod decision_log;
mod prompts;
mod utils;

use clap::{CommandFactory, Parser};
use clap_complete::{generate, shells};
use cli::{AdoptCommands, BranchCommands, Cli, Commands, ShellType};
use colored::Colorize;

#[tokio::main]
async fn main() {
    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .terminal_links(true)
                .unicode(true)
                .context_lines(2)
                .build(),
        )
    }))
    .ok();

    if let Err(err) = run().await {
        eprintln!("{} {:#}", "error:".red().bold(), err);
        std::process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { force } => commands::init::execute(force).await?,
        Commands::Status => commands::status::execute().await?,
        Commands::AiConfig(cmd) => commands::ai_config::execute(&cmd).await?,
        Commands::SeedValidate => commands::seed_validate::execute()?,
        Commands::Diverge { skip_decision } => {
            commands::diverge::execute(skip_decision).await?
        }
        Commands::DivergeValidate => commands::diverge_validate::execute()?,
        Commands::DivergeChallenge => commands::diverge_challenge::execute()?,
        Commands::Converge { skip_decision } => {
            commands::converge::execute(skip_decision).await?
        }
        Commands::ConvergeValidate => commands::converge_validate::execute()?,
        Commands::ConvergeChallenge => commands::converge_challenge::execute()?,
        Commands::Skeleton { skip_decision } => {
            commands::skeleton::execute(skip_decision).await?
        }
        Commands::SkeletonValidate => commands::skeleton_validate::execute()?,
        Commands::Branch(cmd) => match cmd {
            BranchCommands::New { name } => commands::branch_new::execute(&name)?,
            BranchCommands::Start { name } => commands::branch_start::execute(&name)?,
            BranchCommands::Review { name, role } => commands::branch_review::execute(&name, role.as_deref()).await?,
            BranchCommands::Abuse { name } => commands::branch_abuse::execute(&name).await?,
            BranchCommands::Gate { name } => commands::branch_gate::execute(&name)?,
            BranchCommands::Merge { name } => commands::branch_merge::execute(&name)?,
        },
        Commands::Adopt(cmd) => match cmd {
            AdoptCommands::ScanStructure => commands::adopt_scan_structure::execute()?,
            AdoptCommands::ScanDependencies => commands::adopt_scan_dependencies::execute()?,
            AdoptCommands::InferConventions => {
                commands::adopt_infer_conventions::execute().await?
            }
            AdoptCommands::ScanGitHistory { max_commits } => {
                commands::adopt_scan_git_history::execute(max_commits).await?
            }
            AdoptCommands::GapAnalysis => commands::adopt_gap_analysis::execute().await?,
            AdoptCommands::All { max_commits } => {
                commands::adopt_all::execute(max_commits).await?
            }
        },
        Commands::Learn { lesson, category } => {
            commands::learn::execute(&lesson, &category)?
        }
        Commands::Friction { branch, description, severity } => {
            commands::friction::execute(&branch, &description, &severity)?
        }
        Commands::Stabilize { skip_decision } => {
            commands::stabilize::execute(skip_decision)?
        }
        Commands::Postmortem => commands::postmortem::execute().await?,
        Commands::Done => commands::done::execute()?,
        Commands::Generate(cmd) => commands::generate::execute(&cmd)?,
        Commands::Check(cmd) => commands::check::execute(&cmd)?,
        Commands::Pass(cmd) => commands::pass::execute(&cmd)?,
        Commands::Guide => commands::help::execute(),
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            match shell {
                ShellType::Bash => generate(shells::Bash, &mut cmd, &name, &mut std::io::stdout()),
                ShellType::Zsh => generate(shells::Zsh, &mut cmd, &name, &mut std::io::stdout()),
                ShellType::Fish => generate(shells::Fish, &mut cmd, &name, &mut std::io::stdout()),
            }
        }
    }

    Ok(())
}
