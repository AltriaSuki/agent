use anyhow::Result;
use colored::Colorize;
use process_core::{phase::Phase, state::ProcessState};

use super::adopt_utils::ensure_process_dir;

pub async fn execute(max_commits: usize) -> Result<()> {
    println!("{}", "━━━ Adopt All — Full Project Adoption ━━━".bold().blue());
    println!();

    ensure_process_dir()?;

    // 1. Scan Structure
    println!("{}", "── Pass 1/5: Scan Structure ──".bold());
    match super::adopt_scan_structure::execute() {
        Ok(()) => println!("{} scan-structure complete\n", "✔".green()),
        Err(e) => println!("{} scan-structure failed: {}\n", "⚠".yellow(), e),
    }

    // 2. Scan Dependencies
    println!("{}", "── Pass 2/5: Scan Dependencies ──".bold());
    match super::adopt_scan_dependencies::execute() {
        Ok(()) => println!("{} scan-dependencies complete\n", "✔".green()),
        Err(e) => println!("{} scan-dependencies failed: {}\n", "⚠".yellow(), e),
    }

    // 3. Infer Conventions
    println!("{}", "── Pass 3/5: Infer Conventions ──".bold());
    match super::adopt_infer_conventions::execute().await {
        Ok(()) => println!("{} infer-conventions complete\n", "✔".green()),
        Err(e) => println!("{} infer-conventions failed: {}\n", "⚠".yellow(), e),
    }

    // 4. Scan Git History
    println!("{}", "── Pass 4/5: Scan Git History ──".bold());
    match super::adopt_scan_git_history::execute(max_commits).await {
        Ok(()) => println!("{} scan-git-history complete\n", "✔".green()),
        Err(e) => println!("{} scan-git-history failed: {}\n", "⚠".yellow(), e),
    }

    // 5. Gap Analysis
    println!("{}", "── Pass 5/5: Gap Analysis ──".bold());
    match super::adopt_gap_analysis::execute().await {
        Ok(()) => println!("{} gap-analysis complete\n", "✔".green()),
        Err(e) => println!("{} gap-analysis failed: {}\n", "⚠".yellow(), e),
    }

    // Set state to Skeleton so user can branch
    let mut state = ProcessState::load()?;
    state.set_phase(Phase::Skeleton);
    state.save()?;

    println!("{}", "━━━ Adoption Complete ━━━".bold().green());
    print_summary();

    Ok(())
}

fn print_summary() {
    println!();
    println!("{}", "Artifacts produced:".bold());
    check_artifact("skeleton.yaml", ".process/skeleton.yaml");
    check_artifact("seed.yaml", ".process/seed.yaml");
    check_artifact("rules.yaml", ".process/rules.yaml");
    check_artifact("decisions_log.yaml", ".process/decisions_log.yaml");
    check_artifact("gap-report.yaml", ".process/gap-report.yaml");
    println!();
    println!(
        "State set to {}. You can now run {}.",
        "Skeleton".cyan(),
        "process branch new <name>".bold()
    );
}

fn check_artifact(label: &str, path: &str) {
    if std::path::Path::new(path).exists() {
        println!("  {} {}", "✓".green(), label);
    } else {
        println!("  {} {} (not produced)", "✗".red(), label);
    }
}
