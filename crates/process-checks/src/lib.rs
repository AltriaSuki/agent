pub mod sensitive;
pub mod todo;
pub mod lint;
pub mod test;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Trait for all automated checks
pub trait Check {
    /// Check name
    fn name(&self) -> &'static str;

    /// Short description
    fn description(&self) -> &'static str;

    /// Run the check and return findings
    fn run(&self, project_root: &std::path::Path) -> Result<CheckResult>;
}

/// Result of a check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub check_name: String,
    pub passed: bool,
    pub findings: Vec<Finding>,
    pub summary: String,
}

/// A single finding from a check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: Severity,
    pub file: String,
    pub line: Option<usize>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "ERROR"),
            Severity::Warning => write!(f, "WARN"),
            Severity::Info => write!(f, "INFO"),
        }
    }
}
