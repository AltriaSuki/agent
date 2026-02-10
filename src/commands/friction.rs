use anyhow::{bail, Context, Result};
use chrono::Utc;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use process_core::state::ProcessState;

#[derive(Debug, Serialize, Deserialize)]
struct FrictionFile {
    friction_points: Vec<FrictionPoint>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FrictionPoint {
    branch: String,
    timestamp: String,
    description: String,
    severity: String,
    action: String,
}

pub fn execute(branch: &str, description: &str, severity: &str) -> Result<()> {
    println!("{}", "Recording Friction Point".bold().blue());

    let _state = ProcessState::load()?;

    // Validate severity
    let valid = ["high", "medium", "low"];
    if !valid.contains(&severity) {
        bail!("Invalid severity '{}'. Valid: {:?}", severity, valid);
    }

    let path = Path::new(".process/friction.yaml");

    let mut file = if path.exists() {
        let content = fs::read_to_string(path)
            .context("Failed to read friction.yaml")?;
        serde_yaml::from_str::<FrictionFile>(&content)
            .unwrap_or(FrictionFile { friction_points: vec![] })
    } else {
        FrictionFile { friction_points: vec![] }
    };

    file.friction_points.push(FrictionPoint {
        branch: branch.to_string(),
        timestamp: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        description: description.to_string(),
        severity: severity.to_string(),
        action: "pending".to_string(),
    });

    let content = serde_yaml::to_string(&file)
        .context("Failed to serialize friction")?;
    fs::write(path, content)
        .context("Failed to write friction.yaml")?;

    println!("{} Friction point recorded ({} total)", "✔".green(), file.friction_points.len());
    println!("  Branch: {}", branch.cyan());
    println!("  Severity: {}", severity);

    Ok(())
}
