use anyhow::{Context, Result};
use chrono::Utc;
use colored::Colorize;
use dialoguer::Input;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct DecisionsLog {
    pub decisions: Vec<DecisionEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DecisionEntry {
    pub phase_transition: String,
    pub decision: String,
    pub reasoning: String,
    pub confidence: String,
    pub revisit_trigger: String,
    pub decided_by: String,
    pub timestamp: String,
}

/// Prompt the user for a decision record interactively.
/// Returns Ok(true) if recorded, Ok(false) if skipped.
pub fn prompt_decision(phase_transition: &str, skip: bool) -> Result<bool> {
    if skip {
        println!("{} Decision recording skipped (--skip-decision)", "⚠".yellow());
        return Ok(false);
    }

    println!();
    println!("{}", "━━━ 决策记录 (必填) ━━━".bold().cyan());

    let decision: String = Input::new()
        .with_prompt("你做了什么决定？")
        .interact_text()
        .context("Failed to read decision")?;

    let reasoning: String = Input::new()
        .with_prompt("为什么？")
        .interact_text()
        .context("Failed to read reasoning")?;

    let confidence: String = Input::new()
        .with_prompt("信心度 [high/medium/low]")
        .default("medium".to_string())
        .interact_text()
        .context("Failed to read confidence")?;

    let revisit_trigger: String = Input::new()
        .with_prompt("什么情况下需要重新评估？")
        .default("N/A".to_string())
        .interact_text()
        .context("Failed to read revisit trigger")?;

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━".cyan());

    let entry = DecisionEntry {
        phase_transition: phase_transition.to_string(),
        decision,
        reasoning,
        confidence,
        revisit_trigger,
        decided_by: "human".to_string(),
        timestamp: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    };

    append_decision(entry)?;
    println!("{} 决策已记录到 decisions_log.yaml", "✔".green());

    Ok(true)
}

fn append_decision(entry: DecisionEntry) -> Result<()> {
    let path = Path::new(".process/decisions_log.yaml");

    let mut log = if path.exists() {
        let content = fs::read_to_string(path)
            .context("Failed to read decisions_log.yaml")?;
        serde_yaml::from_str::<DecisionsLog>(&content)
            .unwrap_or(DecisionsLog { decisions: vec![] })
    } else {
        DecisionsLog { decisions: vec![] }
    };

    log.decisions.push(entry);

    let content = serde_yaml::to_string(&log)
        .context("Failed to serialize decisions log")?;
    fs::write(path, content)
        .context("Failed to write decisions_log.yaml")?;

    Ok(())
}
