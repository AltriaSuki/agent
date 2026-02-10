use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::Input;
use process_ai::provider::CompletionRequest;
use process_config::config::Config;
use process_core::{phase::Phase, state::ProcessState};
use process_reviews::template::ReviewRegistry;
use std::fs;
use std::path::Path;

use crate::prompts::PromptEngine;
use crate::utils::{get_ai_provider, strip_markdown_code_block};

pub async fn execute(name: &str, role_filter: Option<&str>) -> Result<()> {
    println!("{}", "Branch Review — Multi-Role AI Review".bold().blue());

    let state = ProcessState::load()?;
    state.check_phase(Phase::Branching)?;

    // Check branch file exists
    let branch_path = Path::new(".process/branches").join(format!("{}.yaml", name));
    if !branch_path.exists() {
        bail!("Branch '{}' not found at {}", name, branch_path.display());
    }

    let branch_content =
        fs::read_to_string(&branch_path).context("Failed to read branch file")?;

    // Read rules
    let rules_path = Path::new(".process/converge_summary.yaml");
    let rules_content = if rules_path.exists() {
        fs::read_to_string(rules_path).context("Failed to read rules")?
    } else {
        String::from("(no rules file found)")
    };

    let config = Config::load()?;
    let engine = PromptEngine::new(&config.ai.provider);
    let provider = get_ai_provider(&config).await?;
    println!("Using Provider: {}", provider.name().cyan());

    let registry = ReviewRegistry::default();

    // Determine which roles to run
    let templates: Vec<&dyn process_reviews::template::ReviewTemplate> = match role_filter {
        Some(role_name) => {
            let tmpl = registry.get(role_name).ok_or_else(|| {
                anyhow::anyhow!(
                    "Unknown review role '{}'. Available: {}",
                    role_name,
                    registry.names().join(", ")
                )
            })?;
            vec![tmpl]
        }
        None => registry.all().iter().map(|t| t.as_ref()).collect(),
    };

    println!(
        "Running {} review(s): {}\n",
        templates.len(),
        templates
            .iter()
            .map(|t| t.name())
            .collect::<Vec<_>>()
            .join(", ")
    );

    let mut all_reviews = String::from("reviews:\n");
    let mut has_any_fail = false;

    for tmpl in &templates {
        println!(
            "  {} Running {} review...",
            "→".cyan(),
            tmpl.role().bold()
        );

        // Build per-role prompt
        let mut ctx = tera::Context::new();
        ctx.insert("rules", &rules_content);
        ctx.insert("branch", &branch_content);
        let prompt = engine.render(tmpl.prompt_template_name(), &ctx)?;

        let response = provider
            .complete(&CompletionRequest {
                prompt,
                max_tokens: Some(2048),
                model: None,
            })
            .await?;

        let cleaned = strip_markdown_code_block(&response.content);

        // Detect verdict
        let verdict = if cleaned.contains("verdict: \"fail\"") {
            has_any_fail = true;
            "FAIL".red().bold()
        } else if cleaned.contains("verdict: \"conditional_pass\"") {
            "CONDITIONAL".yellow().bold()
        } else {
            "PASS".green().bold()
        };

        println!("    {} {} → {}", "✔".green(), tmpl.role(), verdict);

        // Accumulate into combined YAML
        all_reviews.push_str(&format!("  # --- {} ---\n", tmpl.role()));
        for line in cleaned.lines() {
            all_reviews.push_str(&format!("  {}\n", line));
        }
        all_reviews.push('\n');
    }

    // Add overall verdict
    let overall = if has_any_fail {
        "fail"
    } else {
        "pass"
    };
    all_reviews.push_str(&format!("overall_verdict: \"{}\"\n", overall));

    // Save combined review
    let review_path = Path::new(".process/branches").join(format!("{}-review.yaml", name));
    fs::write(&review_path, &all_reviews).context("Failed to write review file")?;
    println!(
        "\n{} Review saved to {}",
        "✔".green(),
        review_path.display()
    );

    // Conflict detection and human ruling
    prompt_conflict_ruling(&review_path)?;

    // Update branch status
    let updated = branch_content.replace("status: \"implementing\"", "status: \"reviewing\"");
    fs::write(&branch_path, &updated).context("Failed to update branch status")?;

    println!("{} Branch '{}' status → reviewing", "✔".green(), name);
    println!(
        "\nNext: Run {} for adversarial testing.",
        format!("process branch abuse {}", name).bold()
    );

    Ok(())
}

fn prompt_conflict_ruling(review_path: &Path) -> Result<()> {
    let content = fs::read_to_string(review_path)
        .context("Failed to read review")?;

    // Detect if any role gave a different verdict
    let verdicts: Vec<&str> = content
        .lines()
        .filter(|l| l.contains("verdict:"))
        .collect();

    let has_conflicts = verdicts.len() > 1
        && !verdicts.windows(2).all(|w| w[0] == w[1]);

    if !has_conflicts {
        println!("{} No inter-role conflicts detected", "✔".green());
        return Ok(());
    }

    println!("\n{}", "━━━ Role Conflict Resolution ━━━".bold().cyan());
    println!("Reviewers disagree. Please make a ruling:\n");

    let ruling: String = Input::new()
        .with_prompt("Your ruling")
        .interact_text()
        .context("Failed to read ruling")?;

    let reasoning: String = Input::new()
        .with_prompt("Why this ruling?")
        .interact_text()
        .context("Failed to read reasoning")?;

    let risk: String = Input::new()
        .with_prompt("Risk accepted?")
        .default("N/A".to_string())
        .interact_text()
        .context("Failed to read risk")?;

    let ruling_yaml = format!(
        "\nhuman_conflict_ruling:\n  ruling: \"{}\"\n  reasoning: \"{}\"\n  risk_accepted: \"{}\"\n",
        ruling, reasoning, risk
    );

    let mut full = content;
    full.push_str(&ruling_yaml);
    fs::write(review_path, full)
        .context("Failed to update review with ruling")?;

    println!("{} Conflict ruling recorded", "✔".green());
    Ok(())
}
