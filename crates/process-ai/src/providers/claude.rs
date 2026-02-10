use crate::provider::{AiProvider, CompletionRequest, CompletionResponse, TokenUsage};
use anyhow::{Result, Context, anyhow};
use async_trait::async_trait;
use process_config::config::ProviderConfig;
use reqwest::Client;
use serde_json::{json, Value};
use std::env;

pub struct ClaudeProvider {
    client: Client,
    config: ProviderConfig,
}

impl ClaudeProvider {
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
        // Priority: Config > Env Var (config key and base_url are paired)
        self.config.api_key.clone()
            .or_else(|| env::var("ANTHROPIC_API_KEY").ok())
            .ok_or_else(|| anyhow!("Missing ANTHROPIC_API_KEY"))
    }

    fn get_model(&self) -> String {
        self.config.model.clone()
            .or_else(|| env::var("ANTHROPIC_MODEL").ok())
            .unwrap_or_else(|| "claude-sonnet-4-5-20250929".to_string())
    }

    fn get_base_url(&self) -> String {
        self.config.base_url.clone()
            .or_else(|| env::var("ANTHROPIC_BASE_URL").ok())
            .unwrap_or_else(|| "https://api.anthropic.com".to_string())
            .trim_end_matches('/')
            .to_string()
    }
}

#[async_trait]
impl AiProvider for ClaudeProvider {
    fn name(&self) -> &'static str {
        "claude"
    }

    fn priority(&self) -> u8 {
        // High priority if configured
        if self.get_api_key().is_ok() { 90 } else { 0 }
    }

    async fn is_available(&self) -> bool {
        self.get_api_key().is_ok()
    }

    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        let api_key = self.get_api_key()?;
        let model = request.model.clone().unwrap_or_else(|| self.get_model());
        let base_url = self.get_base_url();
        let max_tokens = request.max_tokens.or(self.config.max_tokens).unwrap_or(4096);

        let url = format!("{}/v1/messages", base_url);

        let payload = json!({
            "model": model,
            "max_tokens": max_tokens,
            "messages": [
                {"role": "user", "content": request.prompt}
            ]
        });

        let is_custom_endpoint = self.config.base_url.is_some()
            || env::var("ANTHROPIC_BASE_URL").is_ok();

        let mut req = self.client.post(&url)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json");

        if is_custom_endpoint {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        } else {
            req = req.header("x-api-key", &api_key);
        }

        let response = req
            .json(&payload)
            .send()
            .await
            .context("Failed to send request to Claude API")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Claude API Error: {}", error_text));
        }

        let body: Value = response.json().await.context("Failed to parse JSON response")?;
        
        // Extract content
        let content = body["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid response format: missing content[0].text"))?
            .to_string();

        // Extract usage if available
        let usage = if let Some(u) = body.get("usage") {
            let prompt_tokens = u["input_tokens"].as_u64().unwrap_or(0) as usize;
            let completion_tokens = u["output_tokens"].as_u64().unwrap_or(0) as usize;
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
