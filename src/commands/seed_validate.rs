use anyhow::{Result, Context, bail};
use colored::Colorize;
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Typed structure for seed validation
#[derive(Deserialize)]
struct SeedOutput {
    idea: String,
    target_user: String,
    constraints: Vec<String>,
    non_goals: Vec<String>,
    success_criteria: Vec<String>,
    reversibility_budget: String,
}

pub fn execute() -> Result<()> {
    println!("{}", "Validating Seed (.process/seed.yaml)".bold().blue());

    // 1. Check file exists
    let seed_path = Path::new(".process/seed.yaml");
    if !seed_path.exists() {
        bail!("Seed file not found at .process/seed.yaml. Run 'process init' first.");
    }

    let content = fs::read_to_string(seed_path).context("Failed to read seed.yaml")?;

    // 2. Parse and validate structure
    let seed: SeedOutput = serde_yaml::from_str(&content)
        .context("Invalid YAML format in seed.yaml. Required fields: idea, target_user, constraints, non_goals, success_criteria, reversibility_budget")?;

    // 3. Validate non-empty fields
    if seed.idea.is_empty() {
        bail!("'idea' cannot be empty");
    }
    if seed.target_user.is_empty() {
        bail!("'target_user' cannot be empty");
    }
    if seed.constraints.is_empty() {
        bail!("'constraints' must have at least one entry");
    }
    if seed.non_goals.is_empty() {
        bail!("'non_goals' must have at least one entry");
    }
    if seed.success_criteria.is_empty() {
        bail!("'success_criteria' must have at least one entry");
    }

    // 4. Validate reversibility_budget
    let valid_budgets = ["high", "medium", "low"];
    if !valid_budgets.contains(&seed.reversibility_budget.as_str()) {
        bail!(
            "Invalid reversibility_budget '{}'. Valid: {:?}",
            seed.reversibility_budget,
            valid_budgets
        );
    }

    println!("{} Seed validated successfully", "✔".green());
    println!("  - Idea: {}", seed.idea.cyan());
    println!("  - {} constraints, {} non-goals, {} success criteria",
        seed.constraints.len(), seed.non_goals.len(), seed.success_criteria.len());
    println!("  - Reversibility: {}", seed.reversibility_budget);

    println!("\nNext: Run {} to generate divergent proposals.", "process diverge".bold());

    Ok(())
}
