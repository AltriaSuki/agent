use anyhow::Result;
use colored::Colorize;
use process_core::{phase::Phase, state::ProcessState};

pub fn execute() -> Result<()> {
    println!("{}", "Phase 7: Done — Project Complete".bold().blue());

    let mut state = ProcessState::load()?;
    state.check_phase(Phase::Postmortem)?;

    state.set_phase(Phase::Done);
    state.save()?;

    println!("{} Project marked as complete!", "✔".green());
    println!();
    println!("Final status: {}", state.current_phase);
    println!();
    println!("Artifacts in .process/:");
    println!("  - seed.yaml");
    println!("  - diverge_summary.yaml");
    println!("  - converge_summary.yaml");
    println!("  - skeleton.yaml");
    println!("  - learnings.yaml");
    println!("  - friction.yaml");
    println!("  - postmortem.yaml");

    Ok(())
}
