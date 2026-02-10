use anyhow::Result;
use crate::cli::CheckCommands;
use colored::Colorize;
use process_checks::{Check, CheckResult, Severity};
use process_checks::sensitive::SensitiveInfoCheck;
use process_checks::todo::TodoCheck;
use process_checks::lint::LintCheck;
use process_checks::test::TestCheck;

pub fn execute(command: &CheckCommands) -> Result<()> {
    let cwd = std::env::current_dir()?;

    match command {
        CheckCommands::Sensitive => { run_check(&SensitiveInfoCheck, &cwd)?; Ok(()) }
        CheckCommands::Todo => { run_check(&TodoCheck, &cwd)?; Ok(()) }
        CheckCommands::Lint => { run_check(&LintCheck, &cwd)?; Ok(()) }
        CheckCommands::Test => { run_check(&TestCheck, &cwd)?; Ok(()) }
        CheckCommands::All => {
            println!("{}", "Running all checks...".bold());
            let checks: Vec<Box<dyn Check>> = vec![
                Box::new(SensitiveInfoCheck),
                Box::new(TodoCheck),
                Box::new(LintCheck),
                Box::new(TestCheck),
            ];
            let mut all_passed = true;
            for check in &checks {
                let result = run_check(check.as_ref(), &cwd)?;
                if !result.passed {
                    all_passed = false;
                }
            }
            if all_passed {
                println!("\n{}", "✅ All checks passed".green().bold());
            } else {
                println!("\n{}", "❌ Some checks failed".red().bold());
            }
            Ok(())
        }
    }
}

fn run_check(check: &dyn Check, project_root: &std::path::Path) -> Result<CheckResult> {
    println!("{} {} — {}", "▶".cyan(), check.name().bold(), check.description());
    let result = check.run(project_root)?;

    let status_icon = if result.passed {
        "✅".to_string()
    } else {
        "❌".to_string()
    };

    // Show findings (limit to 10)
    for (i, finding) in result.findings.iter().enumerate() {
        if i >= 10 {
            println!("  ... and {} more", result.findings.len() - 10);
            break;
        }
        let sev_colored = match finding.severity {
            Severity::Error => finding.severity.to_string().red().to_string(),
            Severity::Warning => finding.severity.to_string().yellow().to_string(),
            Severity::Info => finding.severity.to_string().blue().to_string(),
        };

        let location = if finding.file.is_empty() {
            String::new()
        } else if let Some(line) = finding.line {
            format!("{}:{} ", finding.file, line)
        } else {
            format!("{} ", finding.file)
        };

        println!("  [{}] {}{}", sev_colored, location, finding.message);
    }

    println!("  {} {}", status_icon, result.summary);
    Ok(result)
}
