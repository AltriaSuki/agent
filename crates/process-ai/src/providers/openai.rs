use crate::provider::{AiProvider, CompletionRequest, CompletionResponse, TokenUsage};
use anyhow::{Result, Context, anyhow};
use async_trait::async_trait;
use process_config::config::ProviderConfig;
use reqwest::Client;
use serde_json::{json, Value};
use std::env;

pub struct OpenAiProvider {
    client: Client,
    config: ProviderConfig,
}

impl OpenAiProvider {
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

    fn get_api_key(&self) -> Result<String> {
        if let Ok(key) = env::var("OPENAI_API_KEY") {
            return Ok(key);
        }
        self.config.api_key.clone().ok_or_else(|| anyhow!("Missing OPENAI_API_KEY"))
    }

    fn get_model(&self) -> String {
        env::var("OPENAI_MODEL")
            .ok()
            .or(self.config.model.clone())
            .unwrap_or_else(|| "gpt-4o".to_string())
    }

    fn get_base_url(&self) -> String {
        env::var("OPENAI_BASE_URL")
            .ok()
            .or(self.config.base_url.clone())
            .unwrap_or_else(|| "https://api.openai.com".to_string())
            .trim_end_matches('/')
            .to_string()
    }
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    fn name(&self) -> &'static str {
        "openai"
    }

    fn priority(&self) -> u8 {
        if self.get_api_key().is_ok() { 80 } else { 0 }
    }

    async fn is_available(&self) -> bool {
        self.get_api_key().is_ok()
    }

    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        let api_key = self.get_api_key()?;
        let model = request.model.clone().unwrap_or_else(|| self.get_model());
        let base_url = self.get_base_url();
        let max_tokens = request.max_tokens.or(self.config.max_tokens).unwrap_or(4096);

        let url = format!("{}/v1/chat/completions", base_url);

        let payload = json!({
            "model": model,
            "max_tokens": max_tokens,
            "messages": [
                {"role": "user", "content": request.prompt}
            ]
        });

        let response = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("content-type", "application/json")
            .json(&payload)
            .send()
            .await
            .context("Failed to send request to OpenAI API")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI API Error: {}", error_text));
        }

        let body: Value = response.json().await.context("Failed to parse JSON response")?;
        
        let content = body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid response format: missing choices[0].message.content"))?
            .to_string();

        let usage = if let Some(u) = body.get("usage") {
            let prompt_tokens = u["prompt_tokens"].as_u64().unwrap_or(0) as usize;
            let completion_tokens = u["completion_tokens"].as_u64().unwrap_or(0) as usize;
            Some(TokenUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            })
        } else {
            None
        };

        Ok(CompletionResponse {
            content,
            usage,
        })
    }
}
