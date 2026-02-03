use anyhow::Result;
use process_core::state::ProcessState;
use colored::Colorize;

pub async fn execute() -> Result<()> {
    let state = ProcessState::load()?;
    
    println!("━━━ Process Status ━━━");
    println!("{}: {}", "Current Phase".bold(), state.current_phase);
    println!("{}: {}", "Last Updated".bold(), state.last_updated);
    
    if !state.metadata.is_empty() {
        println!("\nMetadata:");
        for (k, v) in state.metadata {
            println!("  {}: {}", k, v);
        }
    }

    Ok(())
}
