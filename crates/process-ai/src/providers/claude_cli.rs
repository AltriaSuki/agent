use crate::provider::{AiProvider, CompletionRequest, CompletionResponse};
use anyhow::{Result, Context, anyhow};
use async_trait::async_trait;
use std::process::Command;

/// Claude CLI Provider — calls the `claude` command-line tool directly.
/// This is the highest-priority provider when available because it uses
/// the user's authenticated CLI session (no API key management needed).
pub struct ClaudeCliProvider;

impl ClaudeCliProvider {
    pub fn new() -> Self {
        Self
    }

    fn find_claude_binary() -> Option<String> {
        // Check if `claude` is in PATH
        Command::new("which")
            .arg("claude")
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    }
}

#[async_trait]
impl AiProvider for ClaudeCliProvider {
    fn name(&self) -> &'static str {
        "claude-cli"
    }

    fn priority(&self) -> u8 {
        // Highest priority — uses authenticated CLI session
        if Self::find_claude_binary().is_some() { 95 } else { 0 }
    }

    async fn is_available(&self) -> bool {
        Self::find_claude_binary().is_some()
    }

    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        let binary = Self::find_claude_binary()
            .ok_or_else(|| anyhow!("Claude CLI binary not found in PATH"))?;

        let output = Command::new(&binary)
            .arg("--print")
            .arg("--output-format")
            .arg("text")
            .arg(&request.prompt)
            .output()
            .context("Failed to execute claude CLI")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Claude CLI error: {}", stderr));
        }

        let content = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if content.is_empty() {
            return Err(anyhow!("Claude CLI returned empty response"));
        }

        Ok(CompletionResponse {
            content,
            usage: None,
        })
    }
}
