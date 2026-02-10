use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::Input;
use process_core::{phase::Phase, state::ProcessState};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct ConvergeChallenges {
    selected_concerns: SelectedConcerns,
    rejected_regrets: Vec<RejectedRegret>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SelectedConcerns {
    biggest_worry: String,
    hidden_assumption: String,
    what_if_wrong: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RejectedRegret {
    approach: String,
    what_we_lose: String,
    conditions_to_reconsider: String,
}

pub fn execute() -> Result<()> {
    println!("{}", "Converge Challenge — Critique Your Selection".bold().blue());

    let state = ProcessState::load()?;
    state.check_phase(Phase::Converge)?;

    let rules_path = Path::new(".process/rules.yaml");
    if !rules_path.exists() {
        bail!("No converge output found. Run 'process converge' first.");
    }

    let content = fs::read_to_string(rules_path)
        .context("Failed to read rules.yaml")?;

    // Extract rejected approaches
    let rejected: Vec<String> = content
        .lines()
        .filter(|l| l.trim().starts_with("name:") || l.trim().starts_with("- name:"))
        .map(|l| {
            l.trim()
                .trim_start_matches("- ")
                .trim_start_matches("name:")
                .trim()
                .trim_matches('"')
                .to_string()
        })
        .collect();

    println!("{}", "━━━ 对选中方案的质疑 ━━━".bold().cyan());

    let worry: String = Input::new()
        .with_prompt("选中方案最大的隐患是什么？")
        .interact_text()
        .context("Failed to read input")?;

    let assumption: String = Input::new()
        .with_prompt("它依赖了什么隐含假设？")
        .interact_text()
        .context("Failed to read input")?;

    let what_if: String = Input::new()
        .with_prompt("如果这个选择是错的，后果是什么？")
        .interact_text()
        .context("Failed to read input")?;

    if worry.trim().is_empty() {
        bail!("Concern cannot be empty. Think harder.");
    }

    let selected_concerns = SelectedConcerns {
        biggest_worry: worry,
        hidden_assumption: assumption,
        what_if_wrong: what_if,
    };

    // Challenge rejected approaches
    println!("\n{}", "━━━ 对被拒方案的遗憾 ━━━".bold().cyan());

    let mut regrets = Vec::new();
    if rejected.len() > 1 {
        for name in rejected.iter().skip(1).take(3) {
            println!("被拒方案: {}", name.cyan());

            let lose: String = Input::new()
                .with_prompt("放弃它我们失去了什么？")
                .interact_text()
                .context("Failed to read input")?;

            let reconsider: String = Input::new()
                .with_prompt("什么条件下应该重新考虑？")
                .default("N/A".to_string())
                .interact_text()
                .context("Failed to read input")?;

            regrets.push(RejectedRegret {
                approach: name.clone(),
                what_we_lose: lose,
                conditions_to_reconsider: reconsider,
            });
            println!();
        }
    }

    let challenges = ConvergeChallenges {
        selected_concerns,
        rejected_regrets: regrets,
    };

    let output = serde_yaml::to_string(&challenges)
        .context("Failed to serialize")?;

    let out_path = Path::new(".process/converge_challenges.yaml");
    fs::write(out_path, output)
        .context("Failed to write challenges")?;

    println!("{} Challenges saved to {}", "✔".green(), out_path.display());
    println!("\nNext: Run {} to generate skeleton.", "process skeleton".bold());

    Ok(())
}
