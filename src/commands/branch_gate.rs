use anyhow::{bail, Context, Result};
use colored::Colorize;
use process_core::{phase::Phase, state::ProcessState};
use std::fs;
use std::path::Path;

pub fn execute(name: &str) -> Result<()> {
    println!("{}", "Branch Gate — Merge Checklist".bold().blue());

    let state = ProcessState::load()?;
    state.check_phase(Phase::Branching)?;

    let branch_path = Path::new(".process/branches")
        .join(format!("{}.yaml", name));
    if !branch_path.exists() {
        bail!("Branch '{}' not found at {}", name, branch_path.display());
    }

    let branch_content = fs::read_to_string(&branch_path)
        .context("Failed to read branch file")?;

    let mut all_passed = true;

    // Check 1: Review completed
    let review_path = Path::new(".process/branches")
        .join(format!("{}-review.yaml", name));
    if review_path.exists() {
        println!("  {} Review process completed", "✓".green());
    } else {
        println!("  {} Review not found — run 'process branch review {}'",
            "✗".red(), name);
        all_passed = false;
    }

    // Check 2: Abuse tests
    let abuse_path = Path::new(".process/branches")
        .join(format!("{}-abuse.yaml", name));
    if abuse_path.exists() {
        let abuse_content = fs::read_to_string(&abuse_path)
            .context("Failed to read abuse file")?;
        if abuse_content.contains("high_severity_issues: 0") {
            println!("  {} Abuse tests passed (no high severity)", "✓".green());
        } else {
            println!("  {} Abuse tests have high severity issues", "⚠".yellow());
            all_passed = false;
        }
    } else {
        println!("  {} Abuse tests not found — run 'process branch abuse {}'",
            "✗".red(), name);
        all_passed = false;
    }

    // Check 3: Scope creep
    check_scope_creep(&branch_content, name)?;

    // Summary
    println!();
    if all_passed {
        println!("{} All automated checks passed.", "✔".green());
        println!("\nManual checks (please confirm before merging):");
        println!("  ? All tests passing?");
        println!("  ? All invariants verified?");
        println!("  ? Rollback steps tested?");
        println!("\nWhen ready: {}",
            format!("process branch merge {}", name).bold());
    } else {
        println!("{} Some checks failed. Fix issues and re-run.", "✗".red());
    }

    Ok(())
}

fn check_scope_creep(branch_content: &str, name: &str) -> Result<()> {
    // Parse files_not_to_touch from branch definition
    // Simple heuristic: look for files_not_to_touch entries
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", "HEAD~1"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let changed = String::from_utf8_lossy(&out.stdout);
            if changed.trim().is_empty() {
                println!("  {} No file changes detected", "✓".green());
            } else {
                // Check against files_not_to_touch if defined
                if branch_content.contains("files_not_to_touch: []") {
                    println!("  {} No scope restrictions defined", "✓".green());
                } else {
                    println!("  {} Changed files (verify no scope creep):", "⚠".yellow());
                    for file in changed.lines().take(5) {
                        println!("      {}", file);
                    }
                }
            }
        }
        _ => {
            println!("  {} Could not check scope (git diff failed)", "⚠".yellow());
        }
    }

    let _ = name; // used in caller context
    Ok(())
}
