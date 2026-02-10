use crate::{Check, CheckResult, Finding, Severity};
use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// Runs the project's linter (cargo clippy, eslint, ruff, etc.)
pub struct LintCheck;

impl Check for LintCheck {
    fn name(&self) -> &'static str {
        "lint"
    }

    fn description(&self) -> &'static str {
        "Run project linter (auto-detects cargo clippy / eslint / ruff)"
    }

    fn run(&self, project_root: &Path) -> Result<CheckResult> {
        let mut findings = Vec::new();

        let (cmd, args, tool_name) = if project_root.join("Cargo.toml").exists() {
            ("cargo", vec!["clippy", "--all-targets", "--message-format=short"], "cargo clippy")
        } else if project_root.join("package.json").exists() {
            ("npx", vec!["eslint", "."], "eslint")
        } else if project_root.join("pyproject.toml").exists() || project_root.join("ruff.toml").exists() {
            ("ruff", vec!["check", "."], "ruff")
        } else {
            return Ok(CheckResult {
                check_name: self.name().to_string(),
                passed: true,
                findings: vec![],
                summary: "No supported linter detected".to_string(),
            });
        };

        let output = Command::new(cmd)
            .args(&args)
            .current_dir(project_root)
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let combined = format!("{}{}", stdout, stderr);
                let passed = output.status.success();

                if !passed {
                    // Parse first 20 lines as findings
                    for line in combined.lines().take(20) {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            findings.push(Finding {
                                severity: Severity::Warning,
                                file: "".to_string(),
                                line: None,
                                message: trimmed.to_string(),
                            });
                        }
                    }
                }

                let summary = if passed {
                    format!("{}: no issues", tool_name)
                } else {
                    format!("{}: {} issue(s)", tool_name, findings.len())
                };

                Ok(CheckResult {
                    check_name: self.name().to_string(),
                    passed,
                    findings,
                    summary,
                })
            }
            Err(e) => {
                Ok(CheckResult {
                    check_name: self.name().to_string(),
                    passed: false,
                    findings: vec![Finding {
                        severity: Severity::Error,
                        file: "".to_string(),
                        line: None,
                        message: format!("Failed to run {}: {}", tool_name, e),
                    }],
                    summary: format!("{} not available", tool_name),
                })
            }
        }
    }
}
