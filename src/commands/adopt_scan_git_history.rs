use anyhow::{Context, Result};
use colored::Colorize;
use process_ai::provider::CompletionRequest;
use process_config::config::Config;
use std::fs;
use std::path::Path;
use std::process::Command;

use super::adopt_utils::ensure_process_dir;
use crate::utils::{get_ai_provider, strip_markdown_code_block};
use crate::prompts::PromptEngine;

pub async fn execute(max_commits: usize) -> Result<()> {
    println!(
        "{}",
        "Adopt: Scan Git History — AI-assisted git archaeology"
            .bold()
            .blue()
    );

    ensure_process_dir()?;

    // 1. Check if git repo
    let is_git = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !is_git {
        anyhow::bail!("Not a git repository. Cannot scan git history.");
    }

    // 2. Gather git log
    let regular_log = run_git_log(max_commits)?;
    let merge_log = run_git_merge_log(50)?;

    if regular_log.is_empty() {
        anyhow::bail!("No git commits found.");
    }

    println!(
        "{} Collected git history ({} commit limit)",
        "✔".green(),
        max_commits.to_string().cyan()
    );

    // 3. Build prompt and call AI
    let config = Config::load()?;
    let engine = PromptEngine::new(&config.ai.provider);
    let mut ctx = tera::Context::new();
    ctx.insert("commit_log", &regular_log);
    let merge_log_opt = if merge_log.is_empty() { None } else { Some(&merge_log) };
    ctx.insert("merge_log", &merge_log_opt);
    let prompt = engine.render("adopt_scan_git_history", &ctx)?;

    println!("Calling AI to analyze git history...");
    let provider = get_ai_provider(&config).await?;
    println!("Using Provider: {}", provider.name().cyan());

    let response = provider
        .complete(&CompletionRequest {
            prompt,
            max_tokens: Some(4096),
            model: None,
        })
        .await?;

    // 4. Clean and save
    let cleaned = strip_markdown_code_block(&response.content);
    let output_path = Path::new(".process/decisions_log.yaml");
    fs::write(output_path, cleaned).context("Failed to write decisions_log.yaml")?;

    println!(
        "{} Output saved to {}",
        "✔".green(),
        output_path.display()
    );
    println!(
        "\nNext: Review {} for accuracy.",
        "decisions_log.yaml".bold()
    );

    Ok(())
}

fn run_git_log(max_commits: usize) -> Result<String> {
    let output = Command::new("git")
        .args([
            "log",
            "--oneline",
            "--no-merges",
            "-n",
            &max_commits.to_string(),
        ])
        .output()
        .context("Failed to run git log")?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn run_git_merge_log(max: usize) -> Result<String> {
    let output = Command::new("git")
        .args([
            "log",
            "--merges",
            "--oneline",
            "-n",
            &max.to_string(),
        ])
        .output()
        .context("Failed to run git log --merges")?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

