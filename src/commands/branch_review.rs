use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::Input;
use process_ai::provider::CompletionRequest;
use process_config::config::Config;
use process_core::{phase::Phase, state::ProcessState};
use std::fs;
use std::path::Path;

use crate::utils::{get_ai_provider, strip_markdown_code_block};
use crate::prompts::PromptEngine;

pub async fn execute(name: &str) -> Result<()> {
    println!("{}", "Branch Review — Multi-Role AI Review".bold().blue());

    let state = ProcessState::load()?;
    state.check_phase(Phase::Branching)?;

    // Check branch file exists
    let branch_path = Path::new(".process/branches").join(format!("{}.yaml", name));
    if !branch_path.exists() {
        bail!("Branch '{}' not found at {}", name, branch_path.display());
    }

    let branch_content = fs::read_to_string(&branch_path)
        .context("Failed to read branch file")?;

    // Read rules
    let rules_path = Path::new(".process/converge_summary.yaml");
    let rules_content = if rules_path.exists() {
        fs::read_to_string(rules_path).context("Failed to read rules")?
    } else {
        String::from("(no rules file found)")
    };

    // Build prompt
    let config = Config::load()?;
    let engine = PromptEngine::new(&config.ai.provider);
    let mut ctx = tera::Context::new();
    ctx.insert("rules", &rules_content);
    ctx.insert("branch", &branch_content);
    let prompt = engine.render("branch_review", &ctx)?;

    // Call AI
    println!("Calling AI for multi-role review...");
    let provider = get_ai_provider(&config).await?;
    println!("Using Provider: {}", provider.name().cyan());

    let response = provider.complete(&CompletionRequest {
        prompt,
        max_tokens: Some(4096),
        model: None,
    }).await?;

    let cleaned = strip_markdown_code_block(&response.content);

    // Save review
    let review_path = Path::new(".process/branches")
        .join(format!("{}-review.yaml", name));
    fs::write(&review_path, cleaned)
        .context("Failed to write review file")?;
    println!("{} Review saved to {}", "✔".green(), review_path.display());

    // Conflict detection and human ruling
    prompt_conflict_ruling(&review_path)?;

    // Update branch status: implementing → reviewing
    let updated = branch_content.replace(
        "status: \"implementing\"",
        "status: \"reviewing\"",
    );
    fs::write(&branch_path, &updated)
        .context("Failed to update branch status")?;

    println!("{} Branch '{}' status → reviewing", "✔".green(), name);
    println!("\nNext: Run {} for adversarial testing.",
        format!("process branch abuse {}", name).bold());

    Ok(())
}

fn prompt_conflict_ruling(review_path: &Path) -> Result<()> {
    let content = fs::read_to_string(review_path)
        .context("Failed to read review")?;

    // Detect if conflicts section is non-empty
    let has_conflicts = content.contains("conflicts:")
        && !content.contains("conflicts: []");

    if !has_conflicts {
        println!("{} No inter-role conflicts detected", "✔".green());
        return Ok(());
    }

    println!("\n{}", "━━━ 角色冲突裁决 ━━━".bold().cyan());
    println!("AI review 中检测到角色间分歧，请裁决：\n");

    let ruling: String = Input::new()
        .with_prompt("你的裁决是什么？")
        .interact_text()
        .context("Failed to read ruling")?;

    let reasoning: String = Input::new()
        .with_prompt("为什么这样裁决？")
        .interact_text()
        .context("Failed to read reasoning")?;

    let risk: String = Input::new()
        .with_prompt("接受了什么风险？")
        .default("N/A".to_string())
        .interact_text()
        .context("Failed to read risk")?;

    // Append ruling to review file
    let ruling_yaml = format!(
        "\n\nhuman_conflict_ruling:\n  ruling: \"{}\"\n  reasoning: \"{}\"\n  risk_accepted: \"{}\"\n",
        ruling, reasoning, risk
    );

    let mut full = content;
    full.push_str(&ruling_yaml);
    fs::write(review_path, full)
        .context("Failed to update review with ruling")?;

    println!("{} 冲突裁决已记录", "✔".green());
    Ok(())
}
