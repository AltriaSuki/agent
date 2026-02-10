use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

use super::adopt_utils::ensure_process_dir;

pub fn execute() -> Result<()> {
    println!(
        "{}",
        "Adopt: Scan Dependencies — Analyzing project dependencies"
            .bold()
            .blue()
    );

    ensure_process_dir()?;

    let mut language = String::new();
    let mut deps: Vec<(String, String)> = Vec::new();
    let mut dev_deps: Vec<(String, String)> = Vec::new();
    let mut constraints: Vec<String> = Vec::new();

    // Try Cargo.toml
    if Path::new("Cargo.toml").exists() {
        parse_cargo_toml(&mut language, &mut deps, &mut dev_deps, &mut constraints)?;
    }

    // Try package.json
    if Path::new("package.json").exists() {
        parse_package_json(&mut language, &mut deps, &mut dev_deps, &mut constraints)?;
    }

    // Try requirements.txt
    if Path::new("requirements.txt").exists() {
        parse_requirements_txt(&mut language, &mut deps, &mut constraints)?;
    }

    // Try go.mod
    if Path::new("go.mod").exists() {
        parse_go_mod(&mut language, &mut deps, &mut constraints)?;
    }

    // Try pyproject.toml
    if Path::new("pyproject.toml").exists() && language.is_empty() {
        parse_pyproject_toml(&mut language, &mut deps, &mut dev_deps, &mut constraints)?;
    }

    if language.is_empty() {
        language = "unknown".to_string();
        println!(
            "{} No recognized manifest found. Producing minimal seed.yaml.",
            "⚠".yellow()
        );
    }

    // Build seed.yaml
    let yaml = build_seed_yaml(&language, &deps, &dev_deps, &constraints);

    let output_path = Path::new(".process/seed.yaml");
    fs::write(output_path, &yaml).context("Failed to write seed.yaml")?;

    println!(
        "{} Detected language: {}",
        "✔".green(),
        language.cyan()
    );
    println!(
        "{} Found {} dependencies, {} dev-dependencies",
        "✔".green(),
        deps.len().to_string().cyan(),
        dev_deps.len().to_string().cyan()
    );
    println!(
        "{} Output saved to {}",
        "✔".green(),
        output_path.display()
    );
    println!(
        "\n{} Fill in [TODO] placeholders in seed.yaml to complete adoption.",
        "→".bold()
    );

    Ok(())
}

fn parse_cargo_toml(
    language: &mut String,
    deps: &mut Vec<(String, String)>,
    dev_deps: &mut Vec<(String, String)>,
    constraints: &mut Vec<String>,
) -> Result<()> {
    let content = fs::read_to_string("Cargo.toml").context("Failed to read Cargo.toml")?;
    let parsed: toml::Value = content.parse().context("Failed to parse Cargo.toml")?;

    *language = "rust".to_string();

    if let Some(edition) = parsed
        .get("package")
        .and_then(|p| p.get("edition"))
        .and_then(|e| e.as_str())
    {
        constraints.push(format!("Rust edition {}", edition));
    }

    if let Some(table) = parsed.get("dependencies").and_then(|d| d.as_table()) {
        for (name, val) in table {
            let version = extract_toml_version(val);
            deps.push((name.clone(), version));
        }
    }

    if let Some(table) = parsed.get("dev-dependencies").and_then(|d| d.as_table()) {
        for (name, val) in table {
            let version = extract_toml_version(val);
            dev_deps.push((name.clone(), version));
        }
    }

    Ok(())
}

fn extract_toml_version(val: &toml::Value) -> String {
    match val {
        toml::Value::String(s) => s.clone(),
        toml::Value::Table(t) => t
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("*")
            .to_string(),
        _ => "*".to_string(),
    }
}

fn parse_package_json(
    language: &mut String,
    deps: &mut Vec<(String, String)>,
    dev_deps: &mut Vec<(String, String)>,
    constraints: &mut Vec<String>,
) -> Result<()> {
    let content = fs::read_to_string("package.json").context("Failed to read package.json")?;
    let parsed: serde_json::Value =
        serde_json::from_str(&content).context("Failed to parse package.json")?;

    if language.is_empty() {
        // Check for TypeScript
        let has_ts = parsed
            .get("devDependencies")
            .and_then(|d| d.get("typescript"))
            .is_some();
        *language = if has_ts {
            "typescript".to_string()
        } else {
            "javascript".to_string()
        };
    }

    if let Some(engines) = parsed.get("engines").and_then(|e| e.as_object()) {
        for (engine, ver) in engines {
            if let Some(v) = ver.as_str() {
                constraints.push(format!("{} {}", engine, v));
            }
        }
    }

    if let Some(obj) = parsed.get("dependencies").and_then(|d| d.as_object()) {
        for (name, ver) in obj {
            let v = ver.as_str().unwrap_or("*").to_string();
            deps.push((name.clone(), v));
        }
    }

    if let Some(obj) = parsed.get("devDependencies").and_then(|d| d.as_object()) {
        for (name, ver) in obj {
            let v = ver.as_str().unwrap_or("*").to_string();
            dev_deps.push((name.clone(), v));
        }
    }

    Ok(())
}

fn parse_requirements_txt(
    language: &mut String,
    deps: &mut Vec<(String, String)>,
    constraints: &mut Vec<String>,
) -> Result<()> {
    let content =
        fs::read_to_string("requirements.txt").context("Failed to read requirements.txt")?;

    if language.is_empty() {
        *language = "python".to_string();
    }
    constraints.push("pip/requirements.txt".to_string());

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('-') {
            continue;
        }
        // Split on ==, >=, ~=, etc.
        let (name, version) = if let Some(idx) = line.find(['=', '>', '<', '~', '!']) {
            (line[..idx].trim().to_string(), line[idx..].trim().to_string())
        } else {
            (line.to_string(), "*".to_string())
        };
        deps.push((name, version));
    }

    Ok(())
}

fn parse_go_mod(
    language: &mut String,
    deps: &mut Vec<(String, String)>,
    constraints: &mut Vec<String>,
) -> Result<()> {
    let content = fs::read_to_string("go.mod").context("Failed to read go.mod")?;

    if language.is_empty() {
        *language = "go".to_string();
    }

    let mut in_require = false;
    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("go ") {
            constraints.push(format!("go {}", trimmed.trim_start_matches("go ").trim()));
        }

        if trimmed == "require (" {
            in_require = true;
            continue;
        }
        if trimmed == ")" {
            in_require = false;
            continue;
        }

        if in_require {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                deps.push((parts[0].to_string(), parts[1].to_string()));
            }
        }
    }

    Ok(())
}

fn parse_pyproject_toml(
    language: &mut String,
    deps: &mut Vec<(String, String)>,
    dev_deps: &mut Vec<(String, String)>,
    constraints: &mut Vec<String>,
) -> Result<()> {
    let content = fs::read_to_string("pyproject.toml").context("Failed to read pyproject.toml")?;
    let parsed: toml::Value = content.parse().context("Failed to parse pyproject.toml")?;

    *language = "python".to_string();

    if let Some(python_req) = parsed
        .get("project")
        .and_then(|p| p.get("requires-python"))
        .and_then(|r| r.as_str())
    {
        constraints.push(format!("python {}", python_req));
    }

    if let Some(dep_list) = parsed
        .get("project")
        .and_then(|p| p.get("dependencies"))
        .and_then(|d| d.as_array())
    {
        for dep in dep_list {
            if let Some(s) = dep.as_str() {
                deps.push((s.to_string(), "*".to_string()));
            }
        }
    }

    // optional-dependencies often contains dev/test groups
    if let Some(opt) = parsed
        .get("project")
        .and_then(|p| p.get("optional-dependencies"))
        .and_then(|o| o.as_table())
    {
        for (_group, group_deps) in opt {
            if let Some(arr) = group_deps.as_array() {
                for dep in arr {
                    if let Some(s) = dep.as_str() {
                        dev_deps.push((s.to_string(), "*".to_string()));
                    }
                }
            }
        }
    }

    Ok(())
}

fn build_seed_yaml(
    language: &str,
    deps: &[(String, String)],
    dev_deps: &[(String, String)],
    constraints: &[String],
) -> String {
    let mut yaml = String::new();
    yaml.push_str("# seed.yaml — generated by adopt scan-dependencies\n");
    yaml.push_str("# Fill in [TODO] fields to complete adoption.\n\n");
    yaml.push_str("idea: \"[TODO] Describe the core idea of this project\"\n");
    yaml.push_str("target_user: \"[TODO] Who uses this? What scenario?\"\n\n");

    yaml.push_str("constraints:\n");
    for c in constraints {
        yaml.push_str(&format!("  - \"{}\"\n", c));
    }
    yaml.push_str("  # [TODO] Add additional hard constraints\n\n");

    yaml.push_str("non_goals:\n");
    yaml.push_str("  - \"[TODO] What this project explicitly does NOT do\"\n\n");

    yaml.push_str("success_criteria:\n");
    yaml.push_str("  - \"[TODO] Verifiable success criterion\"\n\n");

    yaml.push_str("reversibility_budget: \"medium\"\n\n");

    yaml.push_str(&format!("language: \"{}\"\n\n", language));

    if !deps.is_empty() {
        yaml.push_str("dependencies:\n");
        for (name, ver) in deps {
            yaml.push_str(&format!("  - name: \"{}\"\n    version: \"{}\"\n", name, ver));
        }
        yaml.push('\n');
    }

    if !dev_deps.is_empty() {
        yaml.push_str("dev_dependencies:\n");
        for (name, ver) in dev_deps {
            yaml.push_str(&format!("  - name: \"{}\"\n    version: \"{}\"\n", name, ver));
        }
    }

    yaml
}
