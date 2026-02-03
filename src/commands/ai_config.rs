use anyhow::Result;
use crate::cli::AiConfigCommands;
use process_config::config::Config;
use process_ai::registry::AiRegistry;
use process_ai::providers::claude::ClaudeProvider;
use process_ai::provider::CompletionRequest;
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
                if let Some(url) = &claude.base_url { println!("  Base URL: {}", url); }
            }
        },
        AiConfigCommands::Test => {
            println!("Testing AI Connection...");
            let config = Config::load()?;
            
            // Initialize Registry
            let mut registry = AiRegistry::new();
            registry.register(ClaudeProvider::new(config.ai.claude.clone()));
            // registry.register(OpenAIProvider::new(...));
            
            let provider_name = &config.ai.provider;
            let provider = registry.get_provider(provider_name).await?;
            
            println!("Selected Provider: {}", provider.name().cyan());
            
            let response = provider.complete(&CompletionRequest {
                prompt: "Hello, just say 'Connected'.".to_string(),
                max_tokens: Some(10),
                model: None,
            }).await?;
            
            println!("Response: {}", response.content.green());
            println!("{}", "Connection Successful! ✔".green().bold());
        },
    }
    Ok(())
}
