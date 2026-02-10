use anyhow::{Result, Context};
use colored::Colorize;
use std::fs;
use std::path::Path;
use process_core::state::ProcessState;
// use process_core::phase::Phase;

pub async fn execute(force: bool) -> Result<()> {
    let process_dir = Path::new(".process");

    if process_dir.exists() && !force {
        println!("{}", "Project already initialized. Use --force to re-initialize.".yellow());
        return Ok(());
    }

    // 1. Create directory structure
    fs::create_dir_all(process_dir).context("Failed to create .process directory")?;
    
    // 2. Create seed.yaml template
    let seed_path = process_dir.join("seed.yaml");
    if !seed_path.exists() || force {
        let seed_content = r#"# .process/seed.yaml — 结构化输入，缺一不可
idea: "一句话描述核心想法"
target_user: "谁会用这个？具体场景是什么？"
constraints:
  - "硬约束1 (e.g. 必须纯离线运行)"
  - "硬约束2"
non_goals:
  - "明确不做的事1"
  - "明确不做的事2"
success_criteria:
  - "可验证的成功标准1"
  - "可验证的成功标准2"
reversibility_budget: "high"
# high = 可以大胆实验; medium = 适度谨慎; low = 每步都要可回退
"#;
        fs::write(&seed_path, seed_content).context("Failed to write seed.yaml")?;
        println!("{} Created {}", "✔".green(), seed_path.display());
    }

    // 3. Create config.yaml
    let config_path = process_dir.join("config.yaml");
    if !config_path.exists() || force {
        let config_content = r#"ai:
  provider: auto
  # claude:
  #   api_key: "YOUR_API_KEY"  # Or set ANTHROPIC_API_KEY environment variable
settings:
  auto_save: true
"#;
        fs::write(&config_path, config_content).context("Failed to write config.yaml")?;
        println!("{} Created {}", "✔".green(), config_path.display());
    }

    // 4. Initialize State
    let state = ProcessState::default(); 
    state.save().context("Failed to save initial state")?;
    println!("{} Initialized state to Seed", "✔".green());
    
    println!("{}", "Project initialized successfully! ".green().bold());
    println!("Next: Edit {} and run {}", ".process/seed.yaml".bold(), "process seed-validate".bold());

    Ok(())
}
