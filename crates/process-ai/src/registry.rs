use crate::provider::AiProvider;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;

pub struct AiRegistry {
    providers: HashMap<String, Arc<dyn AiProvider>>,
}

impl Default for AiRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AiRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register<P: AiProvider + 'static>(&mut self, provider: P) {
        let name = provider.name().to_string();
        self.providers.insert(name, Arc::new(provider));
    }

    pub fn provider_exists(&self, name: &str) -> bool {
        self.providers.contains_key(name)
    }

    pub async fn get_provider(&self, name: &str) -> Result<Arc<dyn AiProvider>> {
        if name == "auto" {
            self.auto_detect().await
        } else {
            self.providers.get(name)
                .cloned()
                .ok_or_else(|| anyhow!("Provider '{}' not found", name))
        }
    }

    async fn auto_detect(&self) -> Result<Arc<dyn AiProvider>> {
        // Find provider with highest priority that is available
        let mut candidates = Vec::new();
        
        for provider in self.providers.values() {
            if provider.is_available().await {
                candidates.push(provider);
            }
        }
        
        candidates.sort_by_key(|b| std::cmp::Reverse(b.priority()));
        
        candidates.first()
            .map(|p| Arc::clone(p))
            .ok_or_else(|| anyhow!("No available AI providers found. Please configure API keys or check connections."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{CompletionRequest, CompletionResponse};
    use async_trait::async_trait;

    struct MockProvider {
        mock_name: &'static str,
        mock_priority: u8,
        mock_available: bool,
    }

    impl MockProvider {
        fn new(name: &'static str, priority: u8, available: bool) -> Self {
            Self {
                mock_name: name,
                mock_priority: priority,
                mock_available: available,
            }
        }
    }

    #[async_trait]
    impl AiProvider for MockProvider {
        fn name(&self) -> &'static str {
            self.mock_name
        }

        fn priority(&self) -> u8 {
            self.mock_priority
        }

        async fn is_available(&self) -> bool {
            self.mock_available
        }

        async fn complete(&self, _request: &CompletionRequest) -> Result<CompletionResponse> {
            Ok(CompletionResponse {
                content: format!("Response from {}", self.mock_name),
                usage: None,
            })
        }
    }

    #[test]
    fn test_new_registry_is_empty() {
        let registry = AiRegistry::new();
        assert!(registry.providers.is_empty());
    }

    #[test]
    fn test_register_provider() {
        let mut registry = AiRegistry::new();
        registry.register(MockProvider::new("test", 50, true));
        assert_eq!(registry.providers.len(), 1);
        assert!(registry.providers.contains_key("test"));
    }

    #[test]
    fn test_register_overwrites_same_name() {
        let mut registry = AiRegistry::new();
        registry.register(MockProvider::new("test", 50, true));
        registry.register(MockProvider::new("test", 90, true));
        assert_eq!(registry.providers.len(), 1);
        assert_eq!(registry.providers.get("test").unwrap().priority(), 90);
    }

    #[tokio::test]
    async fn test_get_provider_by_name() {
        let mut registry = AiRegistry::new();
        registry.register(MockProvider::new("mock_claude", 90, true));
        registry.register(MockProvider::new("mock_openai", 80, true));

        let provider = registry.get_provider("mock_claude").await.unwrap();
        assert_eq!(provider.name(), "mock_claude");
    }

    #[tokio::test]
    async fn test_get_unknown_provider_errors() {
        let registry = AiRegistry::new();
        let result = registry.get_provider("nonexistent").await;
        match result {
            Err(e) => assert!(e.to_string().contains("not found")),
            Ok(_) => panic!("Expected error"),
        }
    }

    #[tokio::test]
    async fn test_auto_detect_selects_highest_priority() {
        let mut registry = AiRegistry::new();
        registry.register(MockProvider::new("low", 10, true));
        registry.register(MockProvider::new("high", 90, true));
        registry.register(MockProvider::new("mid", 50, true));

        let provider = registry.get_provider("auto").await.unwrap();
        assert_eq!(provider.name(), "high");
    }

    #[tokio::test]
    async fn test_auto_detect_skips_unavailable() {
        let mut registry = AiRegistry::new();
        registry.register(MockProvider::new("unavailable", 100, false));
        registry.register(MockProvider::new("available", 50, true));

        let provider = registry.get_provider("auto").await.unwrap();
        assert_eq!(provider.name(), "available");
    }

    #[tokio::test]
    async fn test_auto_detect_empty_registry_errors() {
        let registry = AiRegistry::new();
        let result = registry.get_provider("auto").await;
        match result {
            Err(e) => assert!(e.to_string().contains("No available")),
            Ok(_) => panic!("Expected error"),
        }
    }

    #[tokio::test]
    async fn test_auto_detect_all_unavailable_errors() {
        let mut registry = AiRegistry::new();
        registry.register(MockProvider::new("a", 90, false));
        registry.register(MockProvider::new("b", 80, false));

        let result = registry.get_provider("auto").await;
        assert!(result.is_err());
    }
}

