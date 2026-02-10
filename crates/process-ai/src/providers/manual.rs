use crate::provider::{AiProvider, CompletionRequest, CompletionResponse};
use anyhow::{Result, Context, anyhow};
use async_trait::async_trait;
use std::io::{self, Write, BufRead};

/// Manual Provider — displays the prompt to the user and waits for
/// them to paste the AI response. Zero-dependency fallback that works
/// with any AI model via copy-paste.
pub struct ManualProvider;

impl ManualProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AiProvider for ManualProvider {
    fn name(&self) -> &'static str {
        "manual"
    }

    fn priority(&self) -> u8 {
        // Lowest priority — only used as last resort
        1
    }

    async fn is_available(&self) -> bool {
        // Always available — just requires a human
        atty::is(atty::Stream::Stdin)
    }

    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        
        writeln!(out, "\n{}", "=".repeat(60))?;
        writeln!(out, "📋 MANUAL AI MODE")?;
        writeln!(out, "{}", "=".repeat(60))?;
        writeln!(out, "\nCopy the following prompt to your preferred AI:\n")?;
        writeln!(out, "{}", "-".repeat(40))?;
        writeln!(out, "{}", request.prompt)?;
        writeln!(out, "{}", "-".repeat(40))?;
        writeln!(out, "\nPaste the AI response below.")?;
        writeln!(out, "End with an empty line followed by 'END' on its own line:\n")?;
        out.flush()?;

        let stdin = io::stdin();
        let mut response_lines = Vec::new();
        
        for line in stdin.lock().lines() {
            let line = line.context("Failed to read line")?;
            if line.trim() == "END" {
                break;
            }
            response_lines.push(line);
        }

        let content = response_lines.join("\n").trim().to_string();

        if content.is_empty() {
            return Err(anyhow!("Empty response provided"));
        }

        Ok(CompletionResponse {
            content,
            usage: None,
        })
    }
}
