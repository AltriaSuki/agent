use anyhow::{Result, Context, bail};
use colored::Colorize;
use process_core::{state::ProcessState, phase::Phase};
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Typed structure for skeleton validation
#[derive(Deserialize)]
struct SkeletonOutput {
    files: Vec<SkeletonFile>,
}

#[derive(Deserialize)]
struct SkeletonFile {
    path: String,
    #[serde(default)]
    _description: Option<String>,
}

pub fn execute() -> Result<()> {
    println!("{}", "Validating Skeleton Output (.process/skeleton.yaml)".bold().blue());

    // 1. Check file exists
    let skeleton_path = Path::new(".process/skeleton.yaml");
    if !skeleton_path.exists() {
        bail!("Skeleton file not found at .process/skeleton.yaml. Run 'process skeleton' first.");
    }

    let content = fs::read_to_string(skeleton_path).context("Failed to read skeleton.yaml")?;

    // 2. Parse and validate structure
    let output: SkeletonOutput = serde_yaml::from_str(&content)
        .context("Invalid YAML format in skeleton.yaml")?;

    // 3. Validate files
    if output.files.is_empty() {
        bail!("Skeleton must contain at least one file");
    }

    let mut has_readme = false;
    let mut _has_gitignore = false;

    for file in &output.files {
        if file.path.is_empty() {
            bail!("File path cannot be empty");
        }
        if file.path.starts_with('/') || file.path.contains("..") {
             bail!("Invalid file path '{}': Must be relative and safe", file.path);
        }
        
        let path_lower = file.path.to_lowercase();
        if path_lower == "readme.md" {
            has_readme = true;
        }
        if path_lower == ".gitignore" {
            _has_gitignore = true;
        }
    }

    if !has_readme {
        println!("{}", "Warning: No README.md found in skeleton".yellow());
    }

    // 4. Update state (if not already verified, but skeleton-validate IS the verification step)
    // Actually, skeleton command sets phase to Skeleton. Validate confirms it.
    // We can stick to current phase or strictly check we are in Skeleton.
    let state = ProcessState::load()?;
    state.check_phase(Phase::Skeleton)?;
    // No phase change needed here, just confirmation.
    state.save()?; // Update timestamp

    println!("{} Skeleton validated ({} files)", "✔".green(), output.files.len());
    println!("\nSkeleton is ready. Next steps would be to apply this plan (future feature).");

    Ok(())
}
