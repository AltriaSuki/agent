use crate::template::ReviewTemplate;

pub struct SecurityReview;

impl ReviewTemplate for SecurityReview {
    fn name(&self) -> &str {
        "security"
    }

    fn role(&self) -> &str {
        "Security Auditor"
    }

    fn description(&self) -> &str {
        "Injection, privilege escalation, data leaks, and supply chain risks"
    }

    fn focus_areas(&self) -> &[&str] {
        &[
            "Injection vulnerabilities (SQL, command, path traversal)",
            "Authentication and authorization flaws",
            "Sensitive data exposure (keys, tokens, PII)",
            "Input validation and sanitization",
            "Dependency vulnerabilities",
            "Privilege escalation paths",
        ]
    }

    fn prompt_template_name(&self) -> &str {
        "review.security"
    }
}
