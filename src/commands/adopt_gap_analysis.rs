use anyhow::{Context, Result};
use colored::Colorize;
use process_ai::provider::CompletionRequest;
use process_config::config::Config;
use std::fs;
use std::path::Path;

use super::adopt_utils::ensure_process_dir;
use crate::utils::{get_ai_provider, strip_markdown_code_block};
use crate::prompts::PromptEngine;

pub async fn execute() -> Result<()> {
    println!(
        "{}",
        "Adopt: Gap Analysis — AI-assisted gap identification"
            .bold()
            .blue()
    );

    ensure_process_dir()?;

    // 1. Read available artifacts
    let skeleton = read_artifact(".process/skeleton.yaml");
    let rules = read_artifact(".process/rules.yaml");
    let seed = read_artifact(".process/seed.yaml");
    let decisions = read_artifact(".process/decisions_log.yaml");

    let artifact_count = [&skeleton, &rules, &seed, &decisions]
        .iter()
        .filter(|a| a.is_some())
        .count();

    if artifact_count == 0 {
        anyhow::bail!(
            "No artifacts found. Run scan-structure, scan-dependencies, or infer-conventions first."
        );
    }

    println!(
        "{} Found {} existing artifacts",
        "✔".green(),
        artifact_count.to_string().cyan()
    );

    // 2. Build prompt
    let config = Config::load()?;
    let engine = PromptEngine::new(&config.ai.provider);
    let mut ctx = tera::Context::new();

    // Truncate skeleton to 200 lines before inserting
    let skeleton_truncated = skeleton.as_ref().map(|s| {
        s.lines().take(200).collect::<Vec<_>>().join("\n")
    });
    ctx.insert("skeleton", &skeleton_truncated);
    ctx.insert("rules", &rules);
    ctx.insert("seed", &seed);
    ctx.insert("decisions", &decisions);

    let prompt = engine.render("adopt_gap_analysis", &ctx)?;

    // 3. Call AI
    println!("Calling AI to analyze gaps...");
    let provider = get_ai_provider(&config).await?;
    println!("Using Provider: {}", provider.name().cyan());

    let response = provider
        .complete(&CompletionRequest {
            prompt,
            max_tokens: Some(4096),
            model: None,
        })
        .await?;

    // 4. Clean and save
    let cleaned = strip_markdown_code_block(&response.content);
    let output_path = Path::new(".process/gap-report.yaml");
    fs::write(output_path, cleaned).context("Failed to write gap-report.yaml")?;

    println!(
        "{} Output saved to {}",
        "✔".green(),
        output_path.display()
    );
    println!(
        "\nNext: Review {} and address identified gaps.",
        "gap-report.yaml".bold()
    );

    Ok(())
}

fn read_artifact(path: &str) -> Option<String> {
    fs::read_to_string(path).ok()
}
