use anyhow::{Result, Context, bail};
use colored::Colorize;
use process_core::{state::ProcessState, phase::Phase};
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Typed structure for diverge output validation
#[derive(Deserialize)]
struct DivergeOutput {
    proposals: Vec<Proposal>,
    #[serde(default)]
    comparison_dimensions: Vec<ComparisonDimension>,
}

#[derive(Deserialize)]
struct Proposal {
    name: String,
    #[serde(default)]
    #[allow(dead_code)]
    summary: Option<String>,
    architecture: String,
    tradeoffs: Vec<String>,
    risks: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    constraint_alignment: Option<serde_yaml::Value>,
}

#[allow(dead_code)]  // Fields used for deserialization validation
#[derive(Deserialize)]
struct ComparisonDimension {
    dimension: String,
    #[serde(default)]
    ranking: Vec<String>,
    #[serde(default)]
    notes: Option<String>,
}

pub fn execute() -> Result<()> {
    println!("{}", "Validating Diverge Output".bold().blue());

    // 1. Check file exists
    let diverge_path = Path::new(".process/diverge_summary.yaml");
    if !diverge_path.exists() {
        bail!("Diverge file not found at .process/diverge_summary.yaml. Run 'process diverge' first.");
    }

    let content = fs::read_to_string(diverge_path).context("Failed to read diverge_summary.yaml")?;

    // 2. Parse and validate structure using typed deserialization
    let output: DivergeOutput = serde_yaml::from_str(&content)
        .context("Invalid YAML format in diverge_summary.yaml")?;

    // 3. Validate proposals
    if output.proposals.len() < 2 {
        bail!("At least 2 proposals required, found {}", output.proposals.len());
    }

    for proposal in &output.proposals {
        if proposal.name.is_empty() {
            bail!("Proposal name cannot be empty");
        }
        if proposal.architecture.is_empty() {
            bail!("Proposal '{}' missing architecture description", proposal.name);
        }
        if proposal.tradeoffs.is_empty() {
            bail!("Proposal '{}' must have at least one tradeoff", proposal.name);
        }
        if proposal.risks.is_empty() {
            bail!("Proposal '{}' must have at least one risk", proposal.name);
        }
    }

    // 4. Update state
    let mut state = ProcessState::load()?;
    state.set_phase(Phase::Diverge);
    state.save()?;

    println!("{} Diverge output validated ({} proposals)", "✔".green(), output.proposals.len());
    if !output.comparison_dimensions.is_empty() {
        println!("  - {} comparison dimensions", output.comparison_dimensions.len());
    }

    println!("\nNext: Run {} to converge on a single approach.", "process converge".bold());

    Ok(())
}
