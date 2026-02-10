use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::Input;
use process_core::{phase::Phase, state::ProcessState};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct ChallengesFile {
    challenges: Vec<ProposalChallenge>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProposalChallenge {
    proposal: String,
    weaknesses_i_see: String,
    what_could_go_wrong: String,
    question_for_ai: String,
}

pub fn execute() -> Result<()> {
    println!("{}", "Diverge Challenge — Critique AI Proposals".bold().blue());

    let state = ProcessState::load()?;
    state.check_phase(Phase::Diverge)?;

    // Check diverge output exists
    let diverge_path = Path::new(".process/diverge_summary.yaml");
    if !diverge_path.exists() {
        bail!("No diverge output found. Run 'process diverge' first.");
    }

    let content = fs::read_to_string(diverge_path)
        .context("Failed to read diverge_summary.yaml")?;

    // Extract proposal names (simple heuristic)
    let proposals: Vec<String> = content
        .lines()
        .filter(|l| l.trim().starts_with("name:"))
        .map(|l| {
            l.trim()
                .trim_start_matches("name:")
                .trim()
                .trim_matches('"')
                .to_string()
        })
        .collect();

    if proposals.is_empty() {
        bail!("No proposals found in diverge_summary.yaml");
    }

    println!("Found {} proposals. You must challenge each one.\n",
        proposals.len());

    let mut challenges = Vec::new();

    for name in &proposals {
        println!("{}", format!("━━━ Challenge: {} ━━━", name).bold().cyan());

        let weaknesses: String = Input::new()
            .with_prompt("你看到的弱点是什么？")
            .interact_text()
            .context("Failed to read input")?;

        let risk: String = Input::new()
            .with_prompt("最坏情况会怎样？")
            .interact_text()
            .context("Failed to read input")?;

        let question: String = Input::new()
            .with_prompt("想问 AI 什么？")
            .interact_text()
            .context("Failed to read input")?;

        // Validate non-empty
        if weaknesses.trim().is_empty() {
            bail!("Weakness cannot be empty. Think harder.");
        }

        challenges.push(ProposalChallenge {
            proposal: name.clone(),
            weaknesses_i_see: weaknesses,
            what_could_go_wrong: risk,
            question_for_ai: question,
        });

        println!();
    }

    let file = ChallengesFile { challenges };
    let output = serde_yaml::to_string(&file)
        .context("Failed to serialize challenges")?;

    let out_path = Path::new(".process/diverge_challenges.yaml");
    fs::write(out_path, output)
        .context("Failed to write challenges file")?;

    println!("{} Challenges saved to {}", "✔".green(), out_path.display());
    println!("\nNext: Run {} to converge.", "process converge".bold());

    Ok(())
}
