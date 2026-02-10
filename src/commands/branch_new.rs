use anyhow::{bail, Context, Result};
use colored::Colorize;
use process_core::{phase::Phase, state::ProcessState};
use std::fs;
use std::path::Path;

pub fn execute(name: &str) -> Result<()> {
    println!("{}", "Branch New — Creating Branch Hypothesis".bold().blue());

    let mut state = ProcessState::load()?;
    state.check_phase(Phase::Skeleton)?;

    // Ensure branches directory exists
    let branches_dir = Path::new(".process/branches");
    if !branches_dir.exists() {
        fs::create_dir_all(branches_dir)
            .context("Failed to create branches directory")?;
    }

    let branch_path = branches_dir.join(format!("{}.yaml", name));
    if branch_path.exists() {
        bail!("Branch '{}' already exists at {}", name, branch_path.display());
    }

    let template = format!(
r#"# Branch Hypothesis: {name}
hypothesis: ""
# "Adding X feature will make Y possible"

scope:
  files_to_touch: []
  files_not_to_touch: []

invariants_at_risk:
  - ""
# Which invariants might be affected

rollback_plan: |
  git revert to branch start point
  Verify: <specific command>
  Impact scope: <which modules>

dependencies:
  blocked_by: []
  blocks: []

priority: 5
# 1-10, lower number = higher priority

estimated_complexity: "medium"
# small | medium | large

status: "defined"
# defined → implementing → reviewing → abuse-testing → merged | rejected

# ai_config: (optional, override global AI config for this branch)
#   provider: "claude"
#   claude:
#     api_key: "sk-xxx"
#     model: "claude-opus-4-6"
#     base_url: "https://your-proxy.com"
"#);

    fs::write(&branch_path, &template)
        .context("Failed to write branch file")?;

    // Advance to Branching phase
    state.set_phase(Phase::Branching);
    state.save()?;

    println!("{} Branch hypothesis created: {}", "✔".green(), branch_path.display());
    println!("\nNext steps:");
    println!("  1. Edit {} and fill in the hypothesis", branch_path.display());
    println!("  2. Run {} to start implementing", format!("process branch start {}", name).bold());

    Ok(())
}
