use crate::template::ReviewTemplate;

pub struct ArchitectureReview;

impl ReviewTemplate for ArchitectureReview {
    fn name(&self) -> &str {
        "architecture"
    }

    fn role(&self) -> &str {
        "Architecture Reviewer"
    }

    fn description(&self) -> &str {
        "Structural integrity, coupling, cohesion, and design pattern adherence"
    }

    fn focus_areas(&self) -> &[&str] {
        &[
            "Module boundaries and separation of concerns",
            "Coupling between components",
            "Cohesion within modules",
            "Consistency with existing architecture patterns",
            "API surface area and abstraction leaks",
            "Extensibility and future-proofing trade-offs",
        ]
    }

    fn prompt_template_name(&self) -> &str {
        "review.architecture"
    }
}
