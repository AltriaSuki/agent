use anyhow::{Result, Context, bail};
use colored::Colorize;
use process_core::{state::ProcessState, phase::Phase};
use process_config::config::Config;
use process_ai::{registry::AiRegistry, providers::claude::ClaudeProvider, provider::CompletionRequest};
use std::fs;
use std::path::Path;

pub async fn execute() -> Result<()> {
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
    let prompt = format!(r#"You are a software architect. Please read the following project seed and generate ≥2 independent technical proposals.

--- SEED ---
{}
--- END SEED ---

Requirements:
1. Each proposal must include: Architecture sketch (text description), Trade-offs, and Major Risks.
2. Check alignment with each Constraint.
3. Proposals must be substantially different (different architecture/tech stack/trade-offs).
4. Finally, generate a comparison table.

Please output ONLY valid YAML format without code block markers or extra explanation:

proposals:
  - name: "Proposal A"
    summary: "One sentence summary"
    architecture: |
      Multi-line description
    tradeoffs:
      - "Tradeoff 1"
    risks:
      - "Risk 1"
    constraint_alignment:
      Constraint1: "pass | partial | fail"
  - name: "Proposal B"
    ...
comparison_dimensions:
  - dimension: "Dimension Name"
    ranking: ["A", "B"]
    notes: "Explanation"
"#, seed_content);

    // 4. Call AI
    println!("Calling AI to generate proposals...");
    let config = Config::load()?;
    
    // Setup Registry
    let mut registry = AiRegistry::new();
    if let Some(claude_config) = config.ai.claude.clone() {
        registry.register(ClaudeProvider::new(Some(claude_config)));
    }
    // Add other providers here when implemented
    
    let provider = registry.get_provider(&config.ai.provider).await?;
    println!("Using Provider: {}", provider.name().cyan());

    let response = provider.complete(&CompletionRequest {
        prompt,
        max_tokens: Some(4096), // Sufficient for diverge output
        model: None, // Use default from provider config
    }).await?;

    let content = response.content.trim();
    
    // 5. Clean Output (Strip markdown blocks if any)
    let cleaned_content = if content.starts_with("```yaml") {
        content.strip_prefix("```yaml").unwrap_or(content)
            .strip_suffix("```").unwrap_or(content)
            .trim()
    } else if content.starts_with("```") {
         content.strip_prefix("```").unwrap_or(content)
            .strip_suffix("```").unwrap_or(content)
            .trim()
    } else {
        content
    };

    // 6. Save Output
    let output_path = Path::new(".process/diverge_summary.yaml");
    fs::write(output_path, cleaned_content).context("Failed to write diverge_summary.yaml")?;
    println!("{} Output saved to {}", "✔".green(), output_path.display());

    // 7. Update State
    state.set_phase(Phase::Diverge);
    state.save()?;
    println!("{} State updated to Diverge", "✔".green());

    println!("\nNext: Run {} to view detailed status.", "process status".bold());
    
    Ok(())
}
