use crate::{Check, CheckResult, Finding, Severity};
use anyhow::Result;
use std::path::Path;
use std::fs;

/// Scans for TODO, FIXME, HACK, XXX comments
pub struct TodoCheck;

impl Check for TodoCheck {
    fn name(&self) -> &'static str {
        "todo"
    }

    fn description(&self) -> &'static str {
        "Scan for TODO/FIXME/HACK/XXX comments"
    }

    fn run(&self, project_root: &Path) -> Result<CheckResult> {
        let mut findings = Vec::new();
        let markers = ["TODO", "FIXME", "HACK", "XXX"];

        let ignore_dirs: &[&str] = &[
            "target", "node_modules", ".git", "dist", "build", "__pycache__",
            ".venv", "vendor",
        ];

        fn walk(dir: &Path, markers: &[&str], ignore_dirs: &[&str], findings: &mut Vec<Finding>) {
            let entries = match fs::read_dir(dir) {
                Ok(e) => e,
                Err(_) => return,
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    if !ignore_dirs.contains(&name.as_ref()) {
                        walk(&path, markers, ignore_dirs, findings);
                    }
                } else if path.is_file() {
                    if let Ok(content) = fs::read_to_string(&path) {
                        let path_str = path.to_string_lossy().to_string();
                        for (line_num, line) in content.lines().enumerate() {
                            for marker in markers {
                                if line.contains(marker) {
                                    let severity = match *marker {
                                        "FIXME" | "HACK" | "XXX" => Severity::Warning,
                                        _ => Severity::Info,
                                    };
                                    findings.push(Finding {
                                        severity,
                                        file: path_str.clone(),
                                        line: Some(line_num + 1),
                                        message: format!("{}: {}", marker, line.trim()),
                                    });
                                    break; // One finding per line
                                }
                            }
                        }
                    }
                }
            }
        }

        walk(project_root, &markers, ignore_dirs, &mut findings);

        let count = findings.len();
        let summary = format!("{} TODO/FIXME marker(s) found", count);

        Ok(CheckResult {
            check_name: self.name().to_string(),
            passed: true, // TODOs are informational, not failures
            findings,
            summary,
        })
    }
}
