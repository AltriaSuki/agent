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
    println!("{}", "Phase 3: Skeleton — Generating Project Structure".bold().blue());

    // 1. Check State
    let mut state = ProcessState::load()?;
    state.check_phase(Phase::Converge)?;

    // 2. Check Input Files
    let seed_path = Path::new(".process/seed.yaml");
    let rules_path = Path::new(".process/rules.yaml");
    
    if !seed_path.exists() {
        bail!("Seed file not found. Run 'process init' first.");
    }
    if !rules_path.exists() {
        bail!("Rules file not found. Run 'process converge' first.");
    }

    let seed_content = fs::read_to_string(seed_path).context("Failed to read seed.yaml")?;
    let rules_content = fs::read_to_string(rules_path).context("Failed to read rules.yaml")?;

    // 3. Prepare Prompt
    let config = Config::load()?;
    let engine = PromptEngine::new(&config.ai.provider);
    let mut ctx = tera::Context::new();
    ctx.insert("seed", &seed_content);
    ctx.insert("rules", &rules_content);
    let prompt = engine.render("skeleton", &ctx)?;

    // 4. Call AI
    println!("Calling AI to generate skeleton...");
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
    let output_path = Path::new(".process/skeleton.yaml");
    fs::write(output_path, cleaned_content).context("Failed to write skeleton.yaml")?;
    println!("{} Output saved to {}", "✔".green(), output_path.display());

    // 7. Decision recording
    decision_log::prompt_decision("converge → skeleton", skip_decision)?;

    // 8. Update State
    state.set_phase(Phase::Skeleton);
    state.save()?;
    println!("{} State updated to Skeleton", "✔".green());

    println!("\nNext: Run {} to validate skeleton.", "process skeleton-validate".bold());
    
    Ok(())
}
