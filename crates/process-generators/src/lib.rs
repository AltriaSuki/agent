pub mod githooks;
pub mod cicd;
pub mod makefile;
pub mod ide;

use anyhow::Result;

/// Trait for all generators
pub trait Generator {
    /// Generator name
    fn name(&self) -> &'static str;
    
    /// Short description
    fn description(&self) -> &'static str;
    
    /// Generate files into the project directory
    fn generate(&self, project_root: &std::path::Path) -> Result<Vec<GeneratedFile>>;
}

/// Represents a generated file
pub struct GeneratedFile {
    pub path: String,
    pub content: String,
    pub overwritten: bool,
}
