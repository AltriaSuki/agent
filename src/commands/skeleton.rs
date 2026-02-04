use anyhow::{Result, Context, bail};
use colored::Colorize;
use process_core::{state::ProcessState, phase::Phase};
use process_config::config::Config;
use process_ai::provider::CompletionRequest;
use std::fs;
use std::path::Path;
use crate::utils::{strip_markdown_code_block, get_ai_provider};

pub async fn execute() -> Result<()> {
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
    let prompt = format!(r#"You are a software architect. Please parse the Seed and Rules, then generate a comprehensive project directory structure (skeleton).

--- SEED ---
{}
--- END SEED ---

--- RULES ---
{}
--- END RULES ---

Requirements:
1. Generate a complete file tree for a robust basic implementation.
2. Include ALL necessary config files (e.g., Cargo.toml, package.json, Dockerfile, etc.).
3. Follow the architectural decision from RULES (e.g., Monorepo vs Microservices).
4. For each file, provide a 1-sentence description of its purpose.

Please output ONLY valid YAML format without code block markers:

files:
  - path: "README.md"
    description: "Project documentation"
  - path: "Cargo.toml"
    description: "Workspace configuration"
  - path: "src/main.rs"
    description: "Entry point"
  # ... detailed tree
"#, seed_content, rules_content);

    // 4. Call AI
    println!("Calling AI to generate skeleton...");
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
    let output_path = Path::new(".process/skeleton.yaml");
    fs::write(output_path, cleaned_content).context("Failed to write skeleton.yaml")?;
    println!("{} Output saved to {}", "✔".green(), output_path.display());

    // 7. Update State
    state.set_phase(Phase::Skeleton);
    state.save()?;
    println!("{} State updated to Skeleton", "✔".green());

    println!("\nNext: Run {} to validate skeleton.", "process skeleton-validate".bold());
    
    Ok(())
}
