use anyhow::{Result, Context, bail};
use colored::Colorize;
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Typed structure for rules validation
#[derive(Deserialize)]
struct RulesOutput {
    invariants: Vec<Invariant>,
    #[serde(default)]
    conventions: Vec<Convention>,
    conflict_resolution: ConflictResolution,
    #[serde(default)]
    rejected_approaches: Vec<RejectedApproach>,
    selected_approach: SelectedApproach,
}

#[derive(Deserialize)]
struct Invariant {
    id: String,
    #[allow(dead_code)]
    rule: String,
    #[allow(dead_code)]
    rationale: String,
}

#[derive(Deserialize)]
struct Convention {
    id: String,
    #[allow(dead_code)]
    rule: String,
}

#[derive(Deserialize)]
struct ConflictResolution {
    policy: String,
}

#[allow(dead_code)]  // Fields used for deserialization validation
#[derive(Deserialize)]
struct RejectedApproach {
    name: String,
    reason: String,
}

#[derive(Deserialize)]
struct SelectedApproach {
    name: String,
    #[allow(dead_code)]
    rationale: String,
}

pub fn execute() -> Result<()> {
    println!("{}", "Validating Converge Output (rules.yaml)".bold().blue());

    // 1. Check file exists
    let rules_path = Path::new(".process/rules.yaml");
    if !rules_path.exists() {
        bail!("Rules file not found at .process/rules.yaml. Run 'process converge' first.");
    }

    let content = fs::read_to_string(rules_path).context("Failed to read rules.yaml")?;

    // 2. Parse and validate structure using typed deserialization
    let rules: RulesOutput = serde_yaml::from_str(&content)
        .context("Invalid YAML format in rules.yaml")?;

    // 3. Validate invariants
    if rules.invariants.is_empty() {
        bail!("At least 1 invariant is required");
    }

    for inv in &rules.invariants {
        if !inv.id.starts_with("INV-") {
            bail!("Invariant ID '{}' must start with 'INV-'", inv.id);
        }
    }

    // 4. Validate conventions
    for conv in &rules.conventions {
        if !conv.id.starts_with("CONV-") {
            bail!("Convention ID '{}' must start with 'CONV-'", conv.id);
        }
    }

    // 5. Validate conflict resolution policy
    let valid_policies = ["human_final_say", "ai_decides", "majority_vote"];
    if !valid_policies.contains(&rules.conflict_resolution.policy.as_str()) {
        bail!(
            "Invalid conflict resolution policy '{}'. Valid: {:?}",
            rules.conflict_resolution.policy,
            valid_policies
        );
    }

    // 6. Validate selected approach exists
    if rules.selected_approach.name.is_empty() {
        bail!("Selected approach name cannot be empty");
    }

    println!("{} Rules validated successfully", "✔".green());
    println!("  - {} invariants", rules.invariants.len());
    println!("  - {} conventions", rules.conventions.len());
    println!("  - {} rejected approaches", rules.rejected_approaches.len());
    println!("  - Selected: {}", rules.selected_approach.name.cyan());

    println!("\nNext: Run {} to generate project skeleton.", "process skeleton".bold());

    Ok(())
}
