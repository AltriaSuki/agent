use crate::provider::{AiProvider, CompletionRequest, CompletionResponse};
use anyhow::{Result, Context, anyhow};
use async_trait::async_trait;
use process_config::config::ProviderConfig;
use reqwest::Client;
use serde_json::{json, Value};
use std::env;

pub struct OllamaProvider {
    client: Client,
    config: ProviderConfig,
}

impl OllamaProvider {
    pub fn new(config: Option<ProviderConfig>) -> Self {
        Self {
            client: Client::new(),
            config: config.unwrap_or(ProviderConfig {
                api_key: None,
                model: None,
                base_url: None,
                max_tokens: None,
            }),
        }
    }

    fn get_model(&self) -> String {
        env::var("OLLAMA_MODEL")
            .ok()
            .or(self.config.model.clone())
            .unwrap_or_else(|| "llama3.1".to_string())
    }

    fn get_base_url(&self) -> String {
        env::var("OLLAMA_BASE_URL")
            .ok()
            .or(self.config.base_url.clone())
            .unwrap_or_else(|| "http://localhost:11434".to_string())
            .trim_end_matches('/')
            .to_string()
    }
}

#[async_trait]
impl AiProvider for OllamaProvider {
    fn name(&self) -> &'static str {
        "ollama"
    }

    fn priority(&self) -> u8 {
        // Lower priority — local fallback
        30
    }

    async fn is_available(&self) -> bool {
        let url = format!("{}/api/tags", self.get_base_url());
        self.client.get(&url)
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        let model = request.model.clone().unwrap_or_else(|| self.get_model());
        let base_url = self.get_base_url();

        let url = format!("{}/api/generate", base_url);

        let payload = json!({
            "model": model,
            "prompt": request.prompt,
            "stream": false
        });

        let response = self.client.post(&url)
            .header("content-type", "application/json")
            .json(&payload)
            .send()
            .await
            .context("Failed to send request to Ollama")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Ollama API Error: {}", error_text));
        }

        let body: Value = response.json().await.context("Failed to parse JSON response")?;
        
        let content = body["response"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid response format: missing 'response' field"))?
            .to_string();

        Ok(CompletionResponse {
            content,
            usage: None, // Ollama doesn't provide token usage in the same way
        })
    }
}
