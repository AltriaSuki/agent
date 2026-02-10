use anyhow::{Context, Result};
use colored::Colorize;
use process_ai::provider::CompletionRequest;
use process_config::config::Config;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use super::adopt_utils::{ensure_process_dir, IGNORE_DIRS};
use crate::utils::{get_ai_provider, strip_markdown_code_block};
use crate::prompts::PromptEngine;

const MAX_SAMPLE_FILES: usize = 10;
const MAX_LINES_PER_FILE: usize = 100;

pub async fn execute() -> Result<()> {
    println!(
        "{}",
        "Adopt: Infer Conventions — AI-assisted convention detection"
            .bold()
            .blue()
    );

    ensure_process_dir()?;

    // 1. Sample source files from different directories
    let samples = collect_source_samples()?;
    println!(
        "{} Sampled {} source files",
        "✔".green(),
        samples.len().to_string().cyan()
    );

    // 2. Read linter/formatter configs
    let linter_configs = read_linter_configs();

    // 3. Read skeleton.yaml if available
    let skeleton = read_optional_artifact(".process/skeleton.yaml");

    // 4. Build prompt
    let config = Config::load()?;
    let engine = PromptEngine::new(&config.ai.provider);
    let mut ctx = tera::Context::new();

    // Convert tuples to objects for Tera iteration
    let samples_json: Vec<serde_json::Value> = samples
        .iter()
        .map(|(path, content)| serde_json::json!({"path": path, "content": content}))
        .collect();
    ctx.insert("samples", &samples_json);

    let configs_json: Vec<serde_json::Value> = linter_configs
        .iter()
        .map(|(name, content)| serde_json::json!({"name": name, "content": content}))
        .collect();
    ctx.insert("linter_configs", &configs_json);

    // Truncate skeleton to 100 lines before inserting
    let skeleton_truncated = skeleton.as_ref().map(|s| {
        s.lines().take(100).collect::<Vec<_>>().join("\n")
    });
    ctx.insert("skeleton", &skeleton_truncated);

    let prompt = engine.render("adopt_infer_conventions", &ctx)?;

    // 5. Call AI
    println!("Calling AI to infer conventions...");
    let provider = get_ai_provider(&config).await?;
    println!("Using Provider: {}", provider.name().cyan());

    let response = provider
        .complete(&CompletionRequest {
            prompt,
            max_tokens: Some(4096),
            model: None,
        })
        .await?;

    // 6. Clean and save
    let cleaned = strip_markdown_code_block(&response.content);
    let output_path = Path::new(".process/rules.yaml");
    fs::write(output_path, cleaned).context("Failed to write rules.yaml")?;

    println!(
        "{} Output saved to {}",
        "✔".green(),
        output_path.display()
    );
    println!(
        "\nNext: Review {} and adjust inferred conventions.",
        "rules.yaml".bold()
    );

    Ok(())
}

fn collect_source_samples() -> Result<Vec<(String, String)>> {
    let source_exts = [
        "rs", "ts", "tsx", "js", "jsx", "py", "go", "java", "rb", "cpp", "c", "cs", "swift",
        "kt",
    ];
    let mut samples: Vec<(String, String)> = Vec::new();
    let mut seen_dirs: Vec<String> = Vec::new();

    let walker = WalkDir::new(".").into_iter().filter_entry(|e| {
        let name = e.file_name().to_string_lossy();
        !IGNORE_DIRS.contains(&name.as_ref())
    });

    for entry in walker.filter_map(|e| e.ok()) {
        if samples.len() >= MAX_SAMPLE_FILES {
            break;
        }

        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        if !source_exts.contains(&ext.as_str()) {
            continue;
        }

        // Prefer files from different directories
        let dir = path
            .parent()
            .unwrap_or(Path::new("."))
            .to_string_lossy()
            .to_string();

        if seen_dirs.len() < MAX_SAMPLE_FILES && seen_dirs.contains(&dir) {
            continue;
        }
        seen_dirs.push(dir);

        let rel = path
            .strip_prefix("./")
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        if let Ok(content) = fs::read_to_string(path) {
            let truncated: String = content
                .lines()
                .take(MAX_LINES_PER_FILE)
                .collect::<Vec<_>>()
                .join("\n");
            samples.push((rel, truncated));
        }
    }

    Ok(samples)
}

fn read_linter_configs() -> Vec<(String, String)> {
    let config_files = [
        ".editorconfig",
        "rustfmt.toml",
        ".rustfmt.toml",
        "clippy.toml",
        ".eslintrc",
        ".eslintrc.js",
        ".eslintrc.json",
        ".eslintrc.yaml",
        ".prettierrc",
        ".prettierrc.json",
        "biome.json",
    ];

    let mut configs = Vec::new();
    for name in &config_files {
        if let Ok(content) = fs::read_to_string(name) {
            configs.push((name.to_string(), content));
        }
    }
    configs
}

fn read_optional_artifact(path: &str) -> Option<String> {
    fs::read_to_string(path).ok()
}
