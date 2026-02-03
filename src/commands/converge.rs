use anyhow::{Result, Context, bail};
use colored::Colorize;
use process_core::{state::ProcessState, phase::Phase};
use process_config::config::Config;
use process_ai::provider::CompletionRequest;
use std::fs;
use std::path::Path;
use crate::utils::{strip_markdown_code_block, get_ai_provider};

pub async fn execute() -> Result<()> {
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
    let prompt = format!(r#"You are a software architect. Please read the seed and divergent proposals, then complete the following tasks:

--- SEED ---
{}
--- END SEED ---

--- DIVERGE SUMMARY ---
{}
--- END DIVERGE ---

Tasks:
1. Decision: Select/combine proposals, list reasons for eliminated proposals
2. Extract rules from the selected approach

Please output ONLY valid YAML format without code block markers:

invariants:
  - id: "INV-001"
    rule: "Rule description"
    rationale: "Why this rule"
    added_in_phase: 2
    frozen: false

conventions:
  - id: "CONV-001"
    rule: "Convention description"
    rationale: "Why this convention"

conflict_resolution:
  policy: "human_final_say"

rejected_approaches:
  - name: "Rejected proposal name"
    reason: "Reason for rejection"

selected_approach:
  name: "Selected proposal name"
  rationale: "Reason for selection"
"#, seed_content, diverge_content);

    // 4. Call AI
    println!("Calling AI to converge proposals...");
    let config = Config::load()?;
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

    // 7. Update State
    state.set_phase(Phase::Converge);
    state.save()?;
    println!("{} State updated to Converge", "✔".green());

    println!("\nNext: Run {} to validate rules.", "process converge-validate".bold());
    
    Ok(())
}
