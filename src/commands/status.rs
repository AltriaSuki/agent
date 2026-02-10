use anyhow::Result;
use colored::Colorize;
use process_core::{phase::Phase, state::ProcessState};
use std::fs;
use std::path::Path;

pub async fn execute() -> Result<()> {
    let state = ProcessState::load()?;

    println!("━━━ Process Status ━━━");
    println!("{}: {}", "Current Phase".bold(), state.current_phase);
    println!("{}: {}", "Last Updated".bold(), state.last_updated);

    // Progress bar
    let phase_num = state.current_phase as u8;
    let total = Phase::Done as u8;
    let pct = (phase_num as f32 / total as f32 * 100.0) as u8;
    let filled = (phase_num as usize * 20) / total as usize;
    let bar: String = "█".repeat(filled) + &"░".repeat(20 - filled);
    println!("{}: [{}] {}%", "Progress".bold(), bar, pct);

    // Artifact checklist
    println!("\n{}", "Artifacts:".bold());
    check_file("  seed.yaml", ".process/seed.yaml");
    check_file("  diverge_summary.yaml", ".process/diverge_summary.yaml");
    check_file("  converge_summary.yaml", ".process/converge_summary.yaml");
    check_file("  skeleton.yaml", ".process/skeleton.yaml");
    check_file("  learnings.yaml", ".process/learnings.yaml");
    check_file("  friction.yaml", ".process/friction.yaml");
    check_file("  postmortem.yaml", ".process/postmortem.yaml");

    // Branch status
    let branches_dir = Path::new(".process/branches");
    if branches_dir.exists() {
        print_branch_status(branches_dir)?;
    }

    Ok(())
}

fn check_file(label: &str, path: &str) {
    if Path::new(path).exists() {
        println!("{} {}", "✓".green(), label);
    } else {
        println!("{} {}", "·".dimmed(), label.dimmed());
    }
}

fn print_branch_status(dir: &Path) -> Result<()> {
    let mut branches: Vec<(String, String)> = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "yaml") {
            let name = path.file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if name.ends_with("-review") || name.ends_with("-abuse") {
                continue;
            }
            let content = fs::read_to_string(&path).unwrap_or_default();
            let status = extract_status(&content);
            branches.push((name, status));
        }
    }

    if !branches.is_empty() {
        println!("\n{}", "Branches:".bold());
        for (name, status) in &branches {
            let icon = match status.as_str() {
                "merged" => "✓".green(),
                "rejected" => "✗".red(),
                "implementing" => "▶".yellow(),
                _ => "·".normal(),
            };
            println!("  {} {} ({})", icon, name, status);
        }
    }

    Ok(())
}

fn extract_status(content: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("status:") {
            return trimmed
                .trim_start_matches("status:")
                .trim()
                .trim_matches('"')
                .to_string();
        }
    }
    "unknown".to_string()
}
