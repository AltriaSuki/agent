use anyhow::{Context, Result};
use chrono::Utc;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use process_core::state::ProcessState;

#[derive(Debug, Serialize, Deserialize)]
struct LearningsFile {
    learnings: Vec<Learning>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Learning {
    timestamp: String,
    category: String,
    lesson: String,
    phase: String,
}

pub fn execute(lesson: &str, category: &str) -> Result<()> {
    println!("{}", "Recording Learning".bold().blue());

    let state = ProcessState::load()?;

    let learnings_path = Path::new(".process/learnings.yaml");

    // Load existing or create new
    let mut file = if learnings_path.exists() {
        let content = fs::read_to_string(learnings_path)
            .context("Failed to read learnings.yaml")?;
        serde_yaml::from_str::<LearningsFile>(&content)
            .unwrap_or(LearningsFile { learnings: vec![] })
    } else {
        LearningsFile { learnings: vec![] }
    };

    // Append new learning
    let entry = Learning {
        timestamp: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        category: category.to_string(),
        lesson: lesson.to_string(),
        phase: format!("{}", state.current_phase),
    };

    file.learnings.push(entry);

    // Save
    let content = serde_yaml::to_string(&file)
        .context("Failed to serialize learnings")?;
    fs::write(learnings_path, content)
        .context("Failed to write learnings.yaml")?;

    println!("{} Learning recorded ({} total)", "✔".green(), file.learnings.len());
    println!("  Category: {}", category.cyan());
    println!("  Phase: {}", state.current_phase);

    Ok(())
}
