use anyhow::{bail, Context, Result};
use colored::Colorize;
use process_ai::provider::CompletionRequest;
use process_config::config::Config;
use process_core::{phase::Phase, state::ProcessState};
use std::fs;
use std::path::Path;

use crate::prompts::PromptEngine;
use crate::utils::{get_branch_ai_provider, strip_markdown_code_block};

pub async fn execute(name: &str) -> Result<()> {
    println!(
        "{}",
        "Branch Implement — AI-Assisted Implementation".bold().blue()
    );

    let state = ProcessState::load()?;
    state.check_phase(Phase::Branching)?;

    // Check branch file exists and is in implementing status
    let branch_path = Path::new(".process/branches").join(format!("{}.yaml", name));
    if !branch_path.exists() {
        bail!(
            "Branch '{}' not found. Run 'process branch new {}' first.",
            name,
            name
        );
    }

    let branch_content =
        fs::read_to_string(&branch_path).context("Failed to read branch file")?;

    if !branch_content.contains("status: \"implementing\"") {
        bail!(
            "Branch '{}' is not in 'implementing' status. Run 'process branch start {}' first.",
            name,
            name
        );
    }

    // Read project context
    let seed = read_optional(".process/seed.yaml");
    let rules = read_optional(".process/converge_summary.yaml");
    let skeleton = read_optional(".process/skeleton.yaml");

    // Load AI provider (branch-level override or global)
    let config = Config::load()?;
    let engine = PromptEngine::new(&config.ai.provider);
    let (provider, provider_name) =
        get_branch_ai_provider(&config, &branch_content).await?;

    println!("Using Provider: {}", provider_name.cyan());

    // Build prompt
    let mut ctx = tera::Context::new();
    ctx.insert("seed", &seed);
    ctx.insert("rules", &rules);
    ctx.insert("skeleton", &skeleton);
    ctx.insert("branch", &branch_content);

    let prompt = engine.render("branch.implement", &ctx)?;

    println!("  {} Generating implementation plan...", "→".cyan());

    let response = provider
        .complete(&CompletionRequest {
            prompt,
            max_tokens: Some(4096),
            model: None,
        })
        .await?;

    let cleaned = strip_markdown_code_block(&response.content);

    // Save implementation plan
    let plan_path = Path::new(".process/branches").join(format!("{}-implement.yaml", name));
    fs::write(&plan_path, cleaned).context("Failed to write implementation plan")?;

    println!(
        "{} Implementation plan saved to {}",
        "✔".green(),
        plan_path.display()
    );
    println!("\nNext steps:");
    println!("  1. Review the plan in {}", plan_path.display());
    println!("  2. Implement the changes (AI or manual)");
    println!(
        "  3. When done: {}",
        format!("process branch review {}", name).bold()
    );

    Ok(())
}

fn read_optional(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| format!("(no {} found)", path))
}
