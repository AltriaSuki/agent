use anyhow::{Result, Context, bail};
use colored::Colorize;
use process_core::{state::ProcessState, phase::Phase};
use process_config::config::Config;
use process_ai::provider::CompletionRequest;
use std::fs;
use std::path::Path;
use crate::utils::{strip_markdown_code_block, get_ai_provider};
use crate::decision_log;
use crate::prompts::PromptEngine;

pub async fn execute(skip_decision: bool) -> Result<()> {
    println!("{}", "Phase 1: Diverge — Generating Architectural Proposals".bold().blue());

    // 1. Check State
    let mut state = ProcessState::load()?;
    state.check_phase(Phase::Seed)?;

    // 2. Check Input Files
    let seed_path = Path::new(".process/seed.yaml");
    if !seed_path.exists() {
        bail!("Seed file not found at .process/seed.yaml. Run 'process init' first.");
    }
    let seed_content = fs::read_to_string(seed_path).context("Failed to read seed.yaml")?;

    // 3. Prepare Prompt
    let config = Config::load()?;
    let engine = PromptEngine::new(&config.ai.provider);
    let mut ctx = tera::Context::new();
    ctx.insert("seed", &seed_content);
    let prompt = engine.render("diverge", &ctx)?;

    // 4. Call AI
    println!("Calling AI to generate proposals...");
    let provider = get_ai_provider(&config).await?;
    println!("Using Provider: {}", provider.name().cyan());

    let response = provider.complete(&CompletionRequest {
        prompt,
        max_tokens: Some(4096),
        model: None,
    }).await?;

    // 5. Clean Output
    let cleaned_content = strip_markdown_code_block(&response.content);

    // 6. Save Output
    let output_path = Path::new(".process/diverge_summary.yaml");
    fs::write(output_path, cleaned_content).context("Failed to write diverge_summary.yaml")?;
    println!("{} Output saved to {}", "✔".green(), output_path.display());

    // 7. Decision recording
    decision_log::prompt_decision("seed → diverge", skip_decision)?;

    // 8. Update State
    state.set_phase(Phase::Diverge);
    state.save()?;
    println!("{} State updated to Diverge", "✔".green());

    println!("\nNext: Run {} to validate output.", "process diverge-validate".bold());
    
    Ok(())
}
