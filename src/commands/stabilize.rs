use anyhow::{Context, Result};
use colored::Colorize;
use process_core::{phase::Phase, state::ProcessState};
use std::fs;
use std::path::Path;

use crate::decision_log;

pub fn execute(skip_decision: bool) -> Result<()> {
    println!("{}", "Phase 5: Stabilize — Freeze Invariants".bold().blue());

    let mut state = ProcessState::load()?;
    state.check_phase(Phase::Branching)?;

    // Check for unmerged branches
    let branches_dir = Path::new(".process/branches");
    if branches_dir.exists() {
        let mut unmerged = Vec::new();
        for entry in fs::read_dir(branches_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "yaml") {
                let name = path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                // Skip review/abuse files
                if name.ends_with("-review") || name.ends_with("-abuse") {
                    continue;
                }
                let content = fs::read_to_string(&path)
                    .context("Failed to read branch file")?;
                if !content.contains("status: \"merged\"")
                    && !content.contains("status: \"rejected\"")
                {
                    unmerged.push(name);
                }
            }
        }

        if !unmerged.is_empty() {
            println!("{} Unmerged branches found:", "⚠".yellow());
            for b in &unmerged {
                println!("    - {}", b);
            }
            println!("  Consider merging or rejecting before stabilizing.");
        }
    }

    // Check friction points
    let friction_path = Path::new(".process/friction.yaml");
    if friction_path.exists() {
        let content = fs::read_to_string(friction_path)?;
        let high_count = content.matches("severity: high").count()
            + content.matches("severity: \"high\"").count();
        if high_count > 0 {
            println!("{} {} high-severity friction points exist",
                "⚠".yellow(), high_count);
        }
    }

    // Decision recording
    decision_log::prompt_decision("branching → stabilize", skip_decision)?;

    // Update state
    state.set_phase(Phase::Stabilize);
    state.save()?;

    println!("{} State updated to Stabilize", "✔".green());
    println!("\nStabilization rules:");
    println!("  - No new invariants allowed");
    println!("  - Only bugfix branches permitted");
    println!("  - High severity friction points must be resolved");
    println!("\nNext: {}", "process postmortem".bold());

    Ok(())
}
