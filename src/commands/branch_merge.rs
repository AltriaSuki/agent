use anyhow::{bail, Context, Result};
use colored::Colorize;
use process_core::{phase::Phase, state::ProcessState};
use std::fs;
use std::path::Path;

pub fn execute(name: &str) -> Result<()> {
    println!("{}", "Branch Merge — Mark as Merged".bold().blue());

    let state = ProcessState::load()?;
    state.check_phase(Phase::Branching)?;

    let branch_path = Path::new(".process/branches")
        .join(format!("{}.yaml", name));
    if !branch_path.exists() {
        bail!("Branch '{}' not found at {}", name, branch_path.display());
    }

    let content = fs::read_to_string(&branch_path)
        .context("Failed to read branch file")?;

    // Update status to merged
    let updated = content
        .replace("status: \"abuse-testing\"", "status: \"merged\"")
        .replace("status: \"reviewing\"", "status: \"merged\"")
        .replace("status: \"implementing\"", "status: \"merged\"");

    fs::write(&branch_path, &updated)
        .context("Failed to update branch status")?;

    println!("{} Branch '{}' marked as merged", "✔".green(), name);
    println!("\nNext steps:");
    println!("  - Record friction: {}",
        format!("process friction {} \"description\"", name).bold());
    println!("  - New branch: {}", "process branch new <name>".bold());
    println!("  - Stabilize: {}", "process stabilize".bold());

    Ok(())
}
