use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::Input;
use process_ai::provider::CompletionRequest;
use process_config::config::Config;
use process_core::{phase::Phase, state::ProcessState};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::decision_log::DecisionsLog;
use crate::utils::{get_ai_provider, strip_markdown_code_block};
use crate::prompts::PromptEngine;

#[derive(Debug, Serialize, Deserialize)]
struct DecisionReview {
    decision: String,
    phase: String,
    outcome: String,
    lesson: String,
}

pub async fn execute() -> Result<()> {
    println!("{}", "Phase 6: Postmortem — AI Retrospective".bold().blue());

    let mut state = ProcessState::load()?;
    state.check_phase(Phase::Stabilize)?;

    // Gather inputs
    let learnings = read_optional(".process/learnings.yaml");
    let friction = read_optional(".process/friction.yaml");
    let rules = read_optional(".process/converge_summary.yaml");

    let config = Config::load()?;
    let engine = PromptEngine::new(&config.ai.provider);
    let mut ctx = tera::Context::new();
    ctx.insert("learnings", &learnings);
    ctx.insert("friction", &friction);
    ctx.insert("rules", &rules);
    let prompt = engine.render("postmortem", &ctx)?;

    println!("Calling AI for retrospective analysis...");
    let provider = get_ai_provider(&config).await?;
    println!("Using Provider: {}", provider.name().cyan());

    let response = provider.complete(&CompletionRequest {
        prompt,
        max_tokens: Some(4096),
        model: None,
    }).await?;

    let cleaned = strip_markdown_code_block(&response.content);

    let output_path = Path::new(".process/postmortem.yaml");
    fs::write(output_path, cleaned)
        .context("Failed to write postmortem.yaml")?;
    println!("{} Postmortem saved to {}", "✔".green(), output_path.display());

    // MS1.5d: Interactive decision quality review
    let reviews = review_decisions()?;
    if !reviews.is_empty() {
        let reviews_yaml = serde_yaml::to_string(&reviews)
            .context("Failed to serialize decision reviews")?;
        let mut full = cleaned.to_string();
        full.push_str("\n\ndecision_quality_review:\n");
        full.push_str(&reviews_yaml);
        fs::write(output_path, full)
            .context("Failed to update postmortem with decision reviews")?;
        println!("{} Decision quality review appended to postmortem", "✔".green());
    }

    state.set_phase(Phase::Postmortem);
    state.save()?;
    println!("{} State updated to Postmortem", "✔".green());

    println!("\nNext: {}", "process done".bold());

    Ok(())
}

fn read_optional(path: &str) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|_| format!("(no {} found)", path))
}

fn review_decisions() -> Result<Vec<DecisionReview>> {
    let log_path = Path::new(".process/decisions_log.yaml");
    if !log_path.exists() {
        println!("{} No decisions_log.yaml found, skipping decision review", "⚠".yellow());
        return Ok(vec![]);
    }

    let content = fs::read_to_string(log_path)
        .context("Failed to read decisions_log.yaml")?;
    let log: DecisionsLog = serde_yaml::from_str(&content)
        .context("Failed to parse decisions_log.yaml")?;

    if log.decisions.is_empty() {
        println!("{} No decisions recorded, skipping review", "⚠".yellow());
        return Ok(vec![]);
    }

    println!("\n{}", "━━━ 决策质量回顾 ━━━".bold().cyan());
    println!("Review each decision you made during the project.\n");

    let mut reviews = Vec::new();

    for entry in &log.decisions {
        println!("{}", format!("━━━ {} ━━━", entry.phase_transition).bold().cyan());
        println!("  Decision: {}", entry.decision);
        println!("  Reasoning: {}", entry.reasoning);
        println!("  Confidence: {}", entry.confidence);

        let outcome: String = Input::new()
            .with_prompt("结果如何？ [correct/wrong/pending]")
            .default("pending".to_string())
            .interact_text()
            .context("Failed to read outcome")?;

        let lesson: String = Input::new()
            .with_prompt("学到了什么？")
            .default("N/A".to_string())
            .interact_text()
            .context("Failed to read lesson")?;

        reviews.push(DecisionReview {
            decision: entry.decision.clone(),
            phase: entry.phase_transition.clone(),
            outcome,
            lesson,
        });

        println!();
    }

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    Ok(reviews)
}
