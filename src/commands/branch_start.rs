use anyhow::{bail, Context, Result};
use colored::Colorize;
use process_core::{phase::Phase, state::ProcessState};
use std::fs;
use std::path::Path;

pub fn execute(name: &str) -> Result<()> {
    println!("{}", "Branch Start — Validate & Create Git Branch".bold().blue());

    let state = ProcessState::load()?;
    state.check_phase(Phase::Skeleton)?;

    // Check branch file exists
    let branch_path = Path::new(".process/branches").join(format!("{}.yaml", name));
    if !branch_path.exists() {
        bail!("Branch '{}' not found. Run 'process branch new {}' first.", name, name);
    }

    // Read and validate hypothesis is filled in
    let content = fs::read_to_string(&branch_path)
        .context("Failed to read branch file")?;

    if content.contains("hypothesis: \"\"") {
        bail!("Hypothesis is empty. Edit {} and fill in the hypothesis before starting.", branch_path.display());
    }

    // Update status: defined → implementing
    let updated = content.replace("status: \"defined\"", "status: \"implementing\"");
    fs::write(&branch_path, &updated)
        .context("Failed to update branch status")?;

    // Create git branch
    let git_branch = format!("feature/{}", name);
    let output = std::process::Command::new("git")
        .args(["checkout", "-b", &git_branch])
        .output()
        .context("Failed to run git checkout")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("{} Git branch creation failed: {}", "⚠".yellow(), stderr.trim());
        println!("  You may need to create the branch manually.");
    } else {
        println!("{} Created git branch: {}", "✔".green(), git_branch.cyan());
    }

    println!("{} Branch '{}' is now implementing", "✔".green(), name);
    println!("\nGuidance:");
    println!("  - Commit messages should reference invariant IDs:");
    println!("      feat: xxx [INV-001 verified]");
    println!("  - Record learnings: {}", "process learn \"lesson\"".bold());
    println!("  - When done: {}", format!("process branch review {}", name).bold());

    Ok(())
}
