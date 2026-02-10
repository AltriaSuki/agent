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
    println!("{}", "Phase 2: Converge — Pruning & Rule Extraction".bold().blue());

    // 1. Check State
    let mut state = ProcessState::load()?;
    state.check_phase(Phase::Diverge)?;

    // 2. Check Input Files
    let diverge_path = Path::new(".process/diverge_summary.yaml");
    let seed_path = Path::new(".process/seed.yaml");
    
    if !diverge_path.exists() {
        bail!("Diverge file not found. Run 'process diverge' first.");
    }
    if !seed_path.exists() {
        bail!("Seed file not found.");
    }

    let diverge_content = fs::read_to_string(diverge_path).context("Failed to read diverge_summary.yaml")?;
    let seed_content = fs::read_to_string(seed_path).context("Failed to read seed.yaml")?;

    // 3. Prepare Prompt
    let config = Config::load()?;
    let engine = PromptEngine::new(&config.ai.provider);
    let mut ctx = tera::Context::new();
    ctx.insert("seed", &seed_content);
    ctx.insert("diverge_summary", &diverge_content);
    let prompt = engine.render("converge", &ctx)?;

    // 4. Call AI
    println!("Calling AI to converge proposals...");
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
    let output_path = Path::new(".process/rules.yaml");
    fs::write(output_path, cleaned_content).context("Failed to write rules.yaml")?;
    println!("{} Output saved to {}", "✔".green(), output_path.display());

    // 7. Decision recording
    decision_log::prompt_decision("diverge → converge", skip_decision)?;

    // 8. Update State
    state.set_phase(Phase::Converge);
    state.save()?;
    println!("{} State updated to Converge", "✔".green());

    println!("\nNext: Run {} to validate rules.", "process converge-validate".bold());
    
    Ok(())
}
