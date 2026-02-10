use std::sync::Arc;
use anyhow::Result;
use process_config::config::Config;
use process_ai::{
    registry::AiRegistry,
    provider::AiProvider,
    providers::claude::ClaudeProvider,
    providers::openai::OpenAiProvider,
    providers::ollama::OllamaProvider,
    providers::claude_cli::ClaudeCliProvider,
    providers::manual::ManualProvider,
};

/// Strip markdown code block markers from AI responses
pub fn strip_markdown_code_block(content: &str) -> &str {
    let content = content.trim();
    
    // Determine the prefix to strip
    let stripped = if content.starts_with("```yaml") {
        content.strip_prefix("```yaml").unwrap_or(content)
    } else if content.starts_with("```json") {
        content.strip_prefix("```json").unwrap_or(content)
    } else if content.starts_with("```") {
        content.strip_prefix("```").unwrap_or(content)
    } else {
        return content;
    };
    
    // Find the closing ``` and strip everything after it
    if let Some(end_idx) = stripped.rfind("```") {
        stripped[..end_idx].trim()
    } else {
        stripped.trim()
    }
}

/// Initialize AI registry with configured providers
pub fn create_ai_registry(config: &Config) -> AiRegistry {
    let mut registry = AiRegistry::new();
    
    // Claude API provider (priority 90 when key available)
    registry.register(ClaudeProvider::new(config.ai.claude.clone()));

    // OpenAI provider (priority 80 when key available)
    registry.register(OpenAiProvider::new(config.ai.openai.clone()));

    // Ollama local provider (priority 30, always registered)
    registry.register(OllamaProvider::new(config.ai.ollama.clone()));

    // Claude CLI provider (priority 95 when binary found)
    registry.register(ClaudeCliProvider::new());

    // Manual provider (priority 1, always available on TTY)
    registry.register(ManualProvider::new());

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
    fn test_strip_json_block() {
        let input = "```json\n{\"key\": \"value\"}\n```";
        assert_eq!(strip_markdown_code_block(input), "{\"key\": \"value\"}");
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

    #[test]
    fn test_trailing_text_after_block() {
        let input = "```yaml\nkey: value\n```\n\nSome AI explanation text";
        assert_eq!(strip_markdown_code_block(input), "key: value");
    }
}
