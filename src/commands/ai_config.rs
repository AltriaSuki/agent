use anyhow::{Result, anyhow};
use crate::cli::AiConfigCommands;
use crate::utils;
use process_config::config::Config;
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
            if let Some(openai) = &config.ai.openai {
                println!("\n[OpenAI]");
                if let Some(model) = &openai.model { println!("  Model: {}", model); }
                if let Some(tokens) = &openai.max_tokens { println!("  Max Tokens: {}", tokens); }
                if let Some(url) = &openai.base_url { println!("  Base URL: {}", url); }
            }
            if let Some(ollama) = &config.ai.ollama {
                println!("\n[Ollama]");
                if let Some(model) = &ollama.model { println!("  Model: {}", model); }
                if let Some(url) = &ollama.base_url { println!("  Base URL: {}", url); }
            }

            // Show available providers
            let registry = utils::create_ai_registry(&config);
            println!("\n{}", "Available Providers:".bold());
            for name in ["claude", "openai", "ollama", "claude-cli", "manual"] {
                let status = if registry.provider_exists(name) {
                    "✔".green().to_string()
                } else {
                    "✘".red().to_string()
                };
                println!("  {} {}", status, name);
            }
        },
        AiConfigCommands::Test => {
            println!("Testing AI Connection...");
            let config = Config::load()?;

            let provider = utils::get_ai_provider(&config).await?;

            println!("Selected Provider: {}", provider.name().cyan());
            
            let response = provider.complete(&CompletionRequest {
                prompt: "Hello, just say 'Connected'.".to_string(),
                max_tokens: Some(10),
                model: None,
            }).await?;
            
            println!("Response: {}", response.content.green());
            println!("{}", "Connection Successful! ✔".green().bold());
        },
        AiConfigCommands::SetProvider { name } => {
            let valid = ["auto", "claude", "openai", "ollama", "claude-cli", "manual"];
            if !valid.contains(&name.as_str()) {
                return Err(anyhow!(
                    "Unknown provider '{}'. Valid options: {}",
                    name, valid.join(", ")
                ));
            }

            let config_path = std::path::Path::new(".process/config.yaml");
            if !config_path.exists() {
                return Err(anyhow!("No .process/config.yaml found. Run 'process init' first."));
            }

            let content = std::fs::read_to_string(config_path)?;
            let mut doc: serde_yaml::Value = serde_yaml::from_str(&content)?;
            
            if let Some(ai) = doc.get_mut("ai") {
                if let Some(mapping) = ai.as_mapping_mut() {
                    mapping.insert(
                        serde_yaml::Value::String("provider".to_string()),
                        serde_yaml::Value::String(name.clone()),
                    );
                }
            }

            let updated = serde_yaml::to_string(&doc)?;
            std::fs::write(config_path, updated)?;

            println!("Default provider set to: {}", name.green().bold());
        },
    }
    Ok(())
}
