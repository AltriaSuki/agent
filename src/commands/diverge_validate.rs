use anyhow::{Result, Context, bail};
use colored::Colorize;
use process_core::{state::ProcessState, phase::Phase};
use std::fs;
use std::path::Path;

pub fn execute() -> Result<()> {
    println!("{}", "Validating Diverge Output".bold().blue());

    // 1. Check file exists
    let diverge_path = Path::new(".process/diverge_summary.yaml");
    if !diverge_path.exists() {
        bail!("Diverge file not found at .process/diverge_summary.yaml. Run 'process diverge' first.");
    }

    let content = fs::read_to_string(diverge_path).context("Failed to read diverge_summary.yaml")?;

    // 2. Validate YAML structure
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)
        .context("Invalid YAML format in diverge_summary.yaml")?;

    // 3. Check required fields
    let proposals = yaml.get("proposals")
        .ok_or_else(|| anyhow::anyhow!("Missing 'proposals' field in diverge_summary.yaml"))?;

    let proposals_arr = proposals.as_sequence()
        .ok_or_else(|| anyhow::anyhow!("'proposals' must be an array"))?;

    if proposals_arr.len() < 2 {
        bail!("At least 2 proposals required, found {}", proposals_arr.len());
    }

    // Validate each proposal has required fields
    for (i, proposal) in proposals_arr.iter().enumerate() {
        let proposal_map = proposal.as_mapping()
            .ok_or_else(|| anyhow::anyhow!("Proposal {} is not a valid object", i + 1))?;
        
        let name = proposal_map.get(&serde_yaml::Value::String("name".to_string()))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Proposal {} missing 'name' field", i + 1))?;
        
        if proposal_map.get(&serde_yaml::Value::String("architecture".to_string())).is_none() {
            bail!("Proposal '{}' missing 'architecture' field", name);
        }
        if proposal_map.get(&serde_yaml::Value::String("tradeoffs".to_string())).is_none() {
            bail!("Proposal '{}' missing 'tradeoffs' field", name);
        }
        if proposal_map.get(&serde_yaml::Value::String("risks".to_string())).is_none() {
            bail!("Proposal '{}' missing 'risks' field", name);
        }
    }

    // 4. Check comparison dimensions
    if yaml.get("comparison_dimensions").is_none() {
        bail!("Missing 'comparison_dimensions' field");
    }

    // 5. Update state
    let mut state = ProcessState::load()?;
    state.set_phase(Phase::Diverge);
    state.save()?;

    println!("{} Diverge output validated ({} proposals)", "✔".green(), proposals_arr.len());
    println!("\nNext: Run {} to converge on a single approach.", "process converge".bold());

    Ok(())
}
