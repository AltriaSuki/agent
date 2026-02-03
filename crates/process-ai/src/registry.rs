use crate::provider::AiProvider;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;

pub struct AiRegistry {
    providers: HashMap<String, Arc<dyn AiProvider>>,
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
        
        candidates.sort_by(|a, b| b.priority().cmp(&a.priority()));
        
        candidates.first()
            .cloned()
            .cloned() // Arc clone
            .ok_or_else(|| anyhow!("No available AI providers found. Please configure API keys or check connections."))
    }
}
