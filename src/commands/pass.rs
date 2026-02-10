use anyhow::Result;
use crate::cli::PassCommands;
use colored::Colorize;
use process_core::pass_manager::PassManager;

pub fn execute(command: &PassCommands) -> Result<()> {
    let cwd = std::env::current_dir()?;

    // Build the pass manager with all registered passes
    let mut manager = build_pass_manager();

    match command {
        PassCommands::List => {
            println!("{}", "━━━ Registered Passes ━━━".bold());
            let passes = manager.list_passes();
            if passes.is_empty() {
                println!("  (no passes registered yet)");
            } else {
                for (name, desc) in &passes {
                    println!("  {} — {}", name.cyan(), desc);
                }
                println!("\n  {} pass(es) total", passes.len());
            }
        }
        PassCommands::Run { name } => {
            println!("{} Running pass: {}", "▶".cyan(), name.bold());
            manager.run_pass(name, &cwd)?;
            println!("{}", "✅ Pass complete".green().bold());
        }
        PassCommands::RunAll => {
            println!("{}", "Running all passes in dependency order...".bold());
            manager.run_all(&cwd)?;
            println!("{}", "✅ All passes complete".green().bold());
        }
    }

    Ok(())
}

/// Build PassManager with all built-in passes
fn build_pass_manager() -> PassManager {
    let manager = PassManager::new();
    
    // Built-in passes will be registered here as they are migrated
    // from existing commands into Pass implementations.
    //
    // Example (future):
    //   manager.register(SeedInitPass);
    //   manager.register(DivergeGeneratePass);
    //   manager.register(ConvergeAnalyzePass);
    //   manager.register(SkeletonGeneratePass);
    
    manager
}
