use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use process_core::state::ProcessState;

pub const IGNORE_DIRS: &[&str] = &[
    ".git",
    ".process",
    "node_modules",
    "target",
    "__pycache__",
    ".venv",
    "vendor",
    "dist",
    "build",
    ".next",
    ".cache",
];

pub fn ensure_process_dir() -> Result<()> {
    let process_dir = Path::new(".process");

    if process_dir.exists() {
        return Ok(());
    }

    fs::create_dir_all(process_dir).context("Failed to create .process directory")?;
    println!("{} Created .process/", "✔".green());

    // Minimal config.yaml
    let config_path = process_dir.join("config.yaml");
    let config_content = r#"ai:
  provider: auto
settings:
  auto_save: true
"#;
    fs::write(&config_path, config_content).context("Failed to write config.yaml")?;
    println!("{} Created {}", "✔".green(), config_path.display());

    // Initialize state
    let state = ProcessState::default();
    state.save().context("Failed to save initial state")?;
    println!("{} Initialized state to Seed", "✔".green());

    Ok(())
}

pub fn detect_language(extensions: &HashMap<String, usize>) -> String {
    let lang_map: &[(&[&str], &str)] = &[
        (&["rs"], "rust"),
        (&["ts", "tsx"], "typescript"),
        (&["js", "jsx", "mjs", "cjs"], "javascript"),
        (&["py", "pyi"], "python"),
        (&["go"], "go"),
        (&["java"], "java"),
        (&["rb"], "ruby"),
        (&["cpp", "cc", "cxx", "hpp", "h"], "cpp"),
        (&["c"], "c"),
        (&["cs"], "csharp"),
        (&["swift"], "swift"),
        (&["kt", "kts"], "kotlin"),
        (&["php"], "php"),
    ];

    let mut scores: HashMap<&str, usize> = HashMap::new();
    for (exts, lang) in lang_map {
        let total: usize = exts
            .iter()
            .filter_map(|e| extensions.get(*e))
            .sum();
        if total > 0 {
            *scores.entry(lang).or_default() += total;
        }
    }

    scores
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(lang, _)| lang.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn detect_frameworks(root_files: &[String]) -> Vec<String> {
    let mut frameworks = Vec::new();
    let checks: &[(&[&str], &str)] = &[
        (&["Cargo.toml"], "cargo"),
        (&["package.json"], "npm/node"),
        (&["go.mod"], "go-modules"),
        (&["requirements.txt", "pyproject.toml", "setup.py"], "python-packaging"),
        (&["Dockerfile", "docker-compose.yml", "docker-compose.yaml"], "docker"),
        (&["next.config.js", "next.config.mjs", "next.config.ts"], "nextjs"),
        (&["vite.config.ts", "vite.config.js"], "vite"),
        (&["tailwind.config.js", "tailwind.config.ts"], "tailwind"),
        (&["tsconfig.json"], "typescript"),
        (&[".eslintrc", ".eslintrc.js", ".eslintrc.json", ".eslintrc.yaml"], "eslint"),
        (&["Makefile"], "make"),
        (&["CMakeLists.txt"], "cmake"),
    ];

    for (files, framework) in checks {
        if files.iter().any(|f| root_files.contains(&f.to_string())) {
            frameworks.push(framework.to_string());
        }
    }

    frameworks
}

pub fn classify_file(path: &str) -> &'static str {
    let lower = path.to_lowercase();

    if lower.contains("test") || lower.contains("spec") {
        return "test";
    }
    if lower.contains("bench") {
        return "benchmark";
    }
    if lower == "src/main.rs"
        || lower == "src/index.ts"
        || lower == "src/index.js"
        || lower == "main.go"
        || lower == "main.py"
        || lower == "src/lib.rs"
    {
        return "entry-point";
    }

    let filename = Path::new(path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();

    match filename.as_ref() {
        "Cargo.toml" | "package.json" | "go.mod" | "pyproject.toml"
        | "requirements.txt" | "Makefile" | "CMakeLists.txt" => "config",
        "Dockerfile" | "docker-compose.yml" | "docker-compose.yaml" => "infra",
        ".gitignore" | ".editorconfig" | ".prettierrc" | ".eslintrc"
        | "rustfmt.toml" | "clippy.toml" => "tooling",
        "README.md" | "CHANGELOG.md" | "LICENSE" | "CONTRIBUTING.md" => "docs",
        _ => "source",
    }
}
