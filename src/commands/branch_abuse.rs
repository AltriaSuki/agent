use anyhow::{bail, Context, Result};
use colored::Colorize;
use process_ai::provider::CompletionRequest;
use process_config::config::Config;
use process_core::{phase::Phase, state::ProcessState};
use std::fs;
use std::path::Path;

use crate::utils::{get_ai_provider, strip_markdown_code_block};
use crate::prompts::PromptEngine;

pub async fn execute(name: &str) -> Result<()> {
    println!("{}", "Branch Abuse — Adversarial Testing".bold().blue());

    let state = ProcessState::load()?;
    state.check_phase(Phase::Branching)?;

    let branch_path = Path::new(".process/branches")
        .join(format!("{}.yaml", name));
    if !branch_path.exists() {
        bail!("Branch '{}' not found at {}", name, branch_path.display());
    }

    let branch_content = fs::read_to_string(&branch_path)
        .context("Failed to read branch file")?;

    let config = Config::load()?;
    let engine = PromptEngine::new(&config.ai.provider);
    let mut ctx = tera::Context::new();
    ctx.insert("branch", &branch_content);
    let prompt = engine.render("branch_abuse", &ctx)?;

    println!("Calling AI for adversarial testing...");
    let provider = get_ai_provider(&config).await?;
    println!("Using Provider: {}", provider.name().cyan());

    let response = provider.complete(&CompletionRequest {
        prompt,
        max_tokens: Some(4096),
        model: None,
    }).await?;

    let cleaned = strip_markdown_code_block(&response.content);

    // Save abuse test results
    let abuse_path = Path::new(".process/branches")
        .join(format!("{}-abuse.yaml", name));
    fs::write(&abuse_path, cleaned)
        .context("Failed to write abuse test file")?;
    println!("{} Abuse tests saved to {}", "✔".green(), abuse_path.display());

    // Update status: reviewing → abuse-testing
    let updated = branch_content.replace(
        "status: \"reviewing\"",
        "status: \"abuse-testing\"",
    );
    fs::write(&branch_path, &updated)
        .context("Failed to update branch status")?;

    println!("{} Branch '{}' status → abuse-testing", "✔".green(), name);
    println!("\nNext: Run {} for merge gate checks.",
        format!("process branch gate {}", name).bold());

    Ok(())
}
