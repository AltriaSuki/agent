use std::collections::HashMap;

/// A review template defines a specific review perspective.
pub trait ReviewTemplate: Send + Sync {
    /// Unique name for this review role (e.g. "security")
    fn name(&self) -> &str;

    /// Human-readable role title (e.g. "Security Auditor")
    fn role(&self) -> &str;

    /// Short description of what this reviewer focuses on
    fn description(&self) -> &str;

    /// Key focus areas for this review perspective
    fn focus_areas(&self) -> &[&str];

    /// The prompt template name used by PromptEngine (e.g. "review.security")
    fn prompt_template_name(&self) -> &str;

    /// Default severity weights for issue categories
    fn severity_weights(&self) -> HashMap<String, u8> {
        HashMap::new()
    }
}

/// Registry that holds all available review templates.
pub struct ReviewRegistry {
    templates: Vec<Box<dyn ReviewTemplate>>,
}

impl ReviewRegistry {
    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
        }
    }

    pub fn register(&mut self, template: Box<dyn ReviewTemplate>) {
        self.templates.push(template);
    }

    pub fn get(&self, name: &str) -> Option<&dyn ReviewTemplate> {
        self.templates
            .iter()
            .find(|t| t.name() == name)
            .map(|t| t.as_ref())
    }

    pub fn all(&self) -> &[Box<dyn ReviewTemplate>] {
        &self.templates
    }

    pub fn names(&self) -> Vec<&str> {
        self.templates.iter().map(|t| t.name()).collect()
    }
}

impl Default for ReviewRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(crate::templates::general::GeneralReview));
        registry.register(Box::new(crate::templates::security::SecurityReview));
        registry.register(Box::new(crate::templates::performance::PerformanceReview));
        registry.register(Box::new(crate::templates::architecture::ArchitectureReview));
        registry
    }
}
