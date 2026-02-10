use crate::{Check, CheckResult, Finding, Severity};
use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// Runs the project's test suite (cargo test, npm test, pytest, etc.)
pub struct TestCheck;

impl Check for TestCheck {
    fn name(&self) -> &'static str {
        "test"
    }

    fn description(&self) -> &'static str {
        "Run project tests (auto-detects cargo test / npm test / pytest)"
    }

    fn run(&self, project_root: &Path) -> Result<CheckResult> {
        let (cmd, args, tool_name) = if project_root.join("Cargo.toml").exists() {
            ("cargo", vec!["test", "--workspace"], "cargo test")
        } else if project_root.join("package.json").exists() {
            ("npm", vec!["test"], "npm test")
        } else if project_root.join("pyproject.toml").exists() || project_root.join("pytest.ini").exists() {
            ("pytest", vec!["-q"], "pytest")
        } else {
            return Ok(CheckResult {
                check_name: self.name().to_string(),
                passed: true,
                findings: vec![],
                summary: "No supported test framework detected".to_string(),
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

                let mut findings = Vec::new();
                if !passed {
                    // Extract last 15 lines (usually contain the failure summary)
                    let lines: Vec<&str> = combined.lines().collect();
                    let start = lines.len().saturating_sub(15);
                    for line in &lines[start..] {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            findings.push(Finding {
                                severity: Severity::Error,
                                file: "".to_string(),
                                line: None,
                                message: trimmed.to_string(),
                            });
                        }
                    }
                }

                let summary = if passed {
                    format!("{}: all tests passed", tool_name)
                } else {
                    format!("{}: test failures detected", tool_name)
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
