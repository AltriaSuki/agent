use crate::template::ReviewTemplate;

pub struct GeneralReview;

impl ReviewTemplate for GeneralReview {
    fn name(&self) -> &str {
        "general"
    }

    fn role(&self) -> &str {
        "Maintainer"
    }

    fn description(&self) -> &str {
        "Code quality, readability, and long-term maintainability"
    }

    fn focus_areas(&self) -> &[&str] {
        &[
            "Code readability and clarity",
            "Naming conventions consistency",
            "Error handling completeness",
            "Documentation adequacy",
            "Test coverage gaps",
            "Dead code or unused imports",
        ]
    }

    fn prompt_template_name(&self) -> &str {
        "review.general"
    }
}
