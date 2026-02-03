use std::sync::Arc;
use anyhow::Result;
use process_config::config::Config;
use process_ai::{registry::AiRegistry, providers::claude::ClaudeProvider, provider::AiProvider};

/// Strip markdown code block markers from AI responses
pub fn strip_markdown_code_block(content: &str) -> &str {
    let content = content.trim();
    
    // Check for ```yaml or ```json or plain ```
    if content.starts_with("```yaml") {
        content.strip_prefix("```yaml")
            .and_then(|s| s.strip_suffix("```"))
            .map(|s| s.trim())
            .unwrap_or(content)
    } else if content.starts_with("```json") {
        content.strip_prefix("```json")
            .and_then(|s| s.strip_suffix("```"))
            .map(|s| s.trim())
            .unwrap_or(content)
    } else if content.starts_with("```") {
        content.strip_prefix("```")
            .and_then(|s| s.strip_suffix("```"))
            .map(|s| s.trim())
            .unwrap_or(content)
    } else {
        content
    }
}

/// Initialize AI registry with configured providers
pub fn create_ai_registry(config: &Config) -> AiRegistry {
    let mut registry = AiRegistry::new();
    
    if let Some(claude_config) = config.ai.claude.clone() {
        registry.register(ClaudeProvider::new(Some(claude_config)));
    }
    
    // Add other providers here when implemented
    // if let Some(openai_config) = config.ai.openai.clone() {
    //     registry.register(OpenAiProvider::new(Some(openai_config)));
    // }
    
    registry
}

/// Get the configured AI provider
pub async fn get_ai_provider(config: &Config) -> Result<Arc<dyn AiProvider>> {
    let registry = create_ai_registry(config);
    registry.get_provider(&config.ai.provider).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_yaml_block() {
        let input = "```yaml\nkey: value\n```";
        assert_eq!(strip_markdown_code_block(input), "key: value");
    }

    #[test]
    fn test_strip_plain_block() {
        let input = "```\nsome content\n```";
        assert_eq!(strip_markdown_code_block(input), "some content");
    }

    #[test]
    fn test_no_block() {
        let input = "key: value";
        assert_eq!(strip_markdown_code_block(input), "key: value");
    }
}
