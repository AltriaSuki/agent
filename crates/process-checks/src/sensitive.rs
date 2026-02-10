use crate::{Check, CheckResult, Finding, Severity};
use anyhow::Result;
use std::path::Path;
use std::fs;

/// Scans files for sensitive information (API keys, passwords, certificates)
pub struct SensitiveInfoCheck;

impl SensitiveInfoCheck {
    /// Patterns that likely indicate sensitive data
    const PATTERNS: &'static [(&'static str, &'static str)] = &[
        (r#"(?i)api[_-]?key\s*[:=]\s*['"][a-zA-Z0-9\-_]{20,}"#, "API key"),
        (r#"(?i)secret[_-]?key\s*[:=]\s*['"][a-zA-Z0-9\-_]{20,}"#, "Secret key"),
        (r#"(?i)password\s*[:=]\s*['"][^'"]{8,}"#, "Password"),
        (r#"(?i)(aws_access_key_id|aws_secret_access_key)\s*[:=]"#, "AWS credential"),
        (r"sk-[a-zA-Z0-9]{32,}", "OpenAI API key"),
        (r"sk-ant-[a-zA-Z0-9\-]{40,}", "Anthropic API key"),
        (r"ghp_[a-zA-Z0-9]{36}", "GitHub personal access token"),
        (r"-----BEGIN (RSA |EC |DSA )?PRIVATE KEY-----", "Private key"),
    ];

    fn scan_file(path: &Path) -> Vec<Finding> {
        let mut findings = Vec::new();
        
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return findings, // Skip binary files
        };

        let path_str = path.to_string_lossy().to_string();

        for (line_num, line) in content.lines().enumerate() {
            for (pattern, label) in Self::PATTERNS {
                if let Ok(re) = regex::Regex::new(pattern) {
                    if re.is_match(line) {
                        findings.push(Finding {
                            severity: Severity::Error,
                            file: path_str.clone(),
                            line: Some(line_num + 1),
                            message: format!("Possible {} detected", label),
                        });
                    }
                }
            }
        }

        findings
    }
}

impl Check for SensitiveInfoCheck {
    fn name(&self) -> &'static str {
        "sensitive"
    }

    fn description(&self) -> &'static str {
        "Scan for sensitive information (API keys, passwords, certificates)"
    }

    fn run(&self, project_root: &Path) -> Result<CheckResult> {
        let mut findings = Vec::new();
        
        let ignore_dirs: &[&str] = &[
            "target", "node_modules", ".git", "dist", "build", "__pycache__",
            ".venv", "vendor", ".process",
        ];
        let ignore_exts: &[&str] = &[
            "png", "jpg", "jpeg", "gif", "ico", "woff", "woff2", "ttf", "eot",
            "so", "dylib", "dll", "exe", "lock",
        ];

        fn walk(dir: &Path, ignore_dirs: &[&str], ignore_exts: &[&str], findings: &mut Vec<Finding>) {
            let entries = match fs::read_dir(dir) {
                Ok(e) => e,
                Err(_) => return,
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    if !ignore_dirs.contains(&name.as_ref()) {
                        walk(&path, ignore_dirs, ignore_exts, findings);
                    }
                } else if path.is_file() {
                    let ext = path.extension()
                        .map(|e| e.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if !ignore_exts.contains(&ext.as_str()) {
                        findings.extend(SensitiveInfoCheck::scan_file(&path));
                    }
                }
            }
        }

        walk(project_root, ignore_dirs, ignore_exts, &mut findings);

        let count = findings.len();
        let passed = findings.is_empty();
        let summary = if passed {
            "No sensitive information detected".to_string()
        } else {
            format!("{} potential sensitive item(s) found", count)
        };

        Ok(CheckResult {
            check_name: self.name().to_string(),
            passed,
            findings,
            summary,
        })
    }
}
