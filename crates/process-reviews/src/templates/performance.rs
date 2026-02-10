use crate::template::ReviewTemplate;

pub struct PerformanceReview;

impl ReviewTemplate for PerformanceReview {
    fn name(&self) -> &str {
        "performance"
    }

    fn role(&self) -> &str {
        "Performance Engineer"
    }

    fn description(&self) -> &str {
        "Hot paths, latency, memory usage, and concurrency issues"
    }

    fn focus_areas(&self) -> &[&str] {
        &[
            "Hot path efficiency and algorithmic complexity",
            "Memory allocation patterns and leaks",
            "Concurrency correctness (races, deadlocks)",
            "I/O bottlenecks and unnecessary blocking",
            "Cache-friendliness and data locality",
            "Resource cleanup and connection pooling",
        ]
    }

    fn prompt_template_name(&self) -> &str {
        "review.performance"
    }
}
