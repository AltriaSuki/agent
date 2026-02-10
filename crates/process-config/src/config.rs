use serde::{Deserialize, Serialize};
use config::{Config as ConfigLoader, File, Environment};
use anyhow::{Result, Context};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ai: AiConfig,
    pub settings: SettingsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: String,
    pub claude: Option<ProviderConfig>,
    pub openai: Option<ProviderConfig>,
    pub ollama: Option<ProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub max_tokens: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsConfig {
    pub auto_save: bool,
    pub timeout_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ai: AiConfig {
                provider: "auto".to_string(),
                claude: None,
                openai: None,
                ollama: None,
            },
            settings: SettingsConfig {
                auto_save: true,
                timeout_secs: 120,
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let builder = ConfigLoader::builder();

        // 1. Start with default values
        // Note: The `config` crate doesn't easily support struct-based defaults as a "layer" directly 
        // without manual boilerplate, but we can rely on `serde` defaults or manually merging.
        // A common pattern is to use a "clean" builder layered on top.
        // Or simpler: Load defaults into a struct, serialise to Value, add as source? 
        // Actually `config` 0.13 supports `add_source(Config::try_from(&default_struct)?)`.
        
        let defaults = Config::default();
        
        let mut builder = builder
            .set_default("ai.provider", defaults.ai.provider)?
            .set_default("settings.auto_save", defaults.settings.auto_save)?
            .set_default("settings.timeout_secs", defaults.settings.timeout_secs)?;

        // 2. Global Config: ~/.config/process-cli/config.yaml
        if let Some(home_dir) = dirs::home_dir() {
            let global_path = home_dir.join(".config").join("process-cli").join("config.yaml");
            if global_path.exists() {
                builder = builder.add_source(File::from(global_path).required(false));
            }
        }

        // 3. Project Config: .process/config.yaml
        let project_path = PathBuf::from(".process/config.yaml");
        if project_path.exists() {
             builder = builder.add_source(File::from(project_path).required(false));
        }

        // 4. Environment Variables: PROCESS_CLI_AI_PROVIDER, etc.
        // Maps PROCESS_CLI_AI__PROVIDER to ai.provider
        builder = builder.add_source(
            Environment::with_prefix("PROCESS_CLI")
                .separator("__") 
        );

        let config = builder
            .build()
            .context("Failed to build configuration")?;

        let parsed: Config = config.try_deserialize()
            .context("Failed to deserialize configuration")?;

        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.ai.provider, "auto");
        assert!(config.ai.claude.is_none());
        assert!(config.ai.openai.is_none());
        assert!(config.ai.ollama.is_none());
        assert!(config.settings.auto_save);
        assert_eq!(config.settings.timeout_secs, 120);
    }

    #[test]
    fn test_load_returns_defaults_when_no_files() {
        // When no config files exist, should use defaults
        let config = Config::load().unwrap();
        assert_eq!(config.ai.provider, "auto");
        assert!(config.settings.auto_save);
    }

    #[test]
    fn test_provider_config_fields() {
        let pc = ProviderConfig {
            api_key: Some("test-key".to_string()),
            model: Some("gpt-4o".to_string()),
            base_url: Some("https://api.example.com".to_string()),
            max_tokens: Some(8192),
        };
        assert_eq!(pc.api_key.unwrap(), "test-key");
        assert_eq!(pc.max_tokens.unwrap(), 8192);
    }
}

