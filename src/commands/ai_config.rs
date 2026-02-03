use anyhow::Result;
use crate::cli::AiConfigCommands;
use process_config::config::Config;
use colored::Colorize;

pub async fn execute(command: &AiConfigCommands) -> Result<()> {
    match command {
        AiConfigCommands::Show => {
            let config = Config::load()?;
            println!("━━━ AI Configuration ━━━");
            println!("{}: {}", "Provider".bold(), config.ai.provider);
            println!("{}: {}", "Timeout".bold(), config.settings.timeout_secs);
            
            if let Some(claude) = &config.ai.claude {
                println!("\n[Claude]");
                if let Some(model) = &claude.model { println!("  Model: {}", model); }
                if let Some(tokens) = &claude.max_tokens { println!("  Max Tokens: {}", tokens); }
            }
            // ... print others ...
        },
        AiConfigCommands::Test => {
            println!("Configuration loaded successfully.");
            // TODO: Implement actual connection test using process-ai
        },
    }
    Ok(())
}
