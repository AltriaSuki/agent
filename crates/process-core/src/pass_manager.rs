use crate::pass::{Pass, PassContext};
use crate::manifest::Manifest;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::Path;

/// The PassManager registers, resolves dependencies, and executes passes
pub struct PassManager {
    passes: HashMap<String, Box<dyn Pass>>,
    execution_order: Vec<String>,
}

impl PassManager {
    pub fn new() -> Self {
        Self {
            passes: HashMap::new(),
            execution_order: Vec::new(),
        }
    }

    /// Register a pass
    pub fn register<P: Pass + 'static>(&mut self, pass: P) {
        let name = pass.name().to_string();
        self.passes.insert(name, Box::new(pass));
    }

    /// List all registered passes
    pub fn list_passes(&self) -> Vec<(&str, &str)> {
        let mut passes: Vec<_> = self.passes.values()
            .map(|p| (p.name(), p.description()))
            .collect();
        passes.sort_by_key(|(name, _)| *name);
        passes
    }

    /// Resolve execution order using topological sort
    pub fn resolve_order(&mut self) -> Result<()> {
        let mut order = Vec::new();
        let mut visited = HashMap::new();

        for name in self.passes.keys() {
            self.visit(name, &mut visited, &mut order)?;
        }

        self.execution_order = order;
        Ok(())
    }

    fn visit(
        &self,
        name: &str,
        visited: &mut HashMap<String, bool>,
        order: &mut Vec<String>,
    ) -> Result<()> {
        if let Some(&in_progress) = visited.get(name) {
            if in_progress {
                return Err(anyhow!("Circular dependency detected at pass '{}'", name));
            }
            return Ok(()); // Already visited
        }

        visited.insert(name.to_string(), true); // Mark in progress

        // Visit dependencies
        if let Some(pass) = self.passes.get(name) {
            let requires = pass.requires();
            for req in &requires {
                // Find which pass produces this artifact
                for (pname, p) in &self.passes {
                    if p.produces().contains(req) && pname != name {
                        self.visit(pname, visited, order)?;
                    }
                }
            }
        }

        visited.insert(name.to_string(), false); // Mark done
        if !order.contains(&name.to_string()) {
            order.push(name.to_string());
        }
        Ok(())
    }

    /// Run a single pass by name
    pub fn run_pass(&self, name: &str, project_root: &Path) -> Result<()> {
        let pass = self.passes.get(name)
            .ok_or_else(|| anyhow!("Pass '{}' not found", name))?;

        let mut ctx = PassContext::new(project_root);
        let mut manifest = Manifest::load(project_root)?;

        // Load required artifacts
        for req in pass.requires() {
            if let Err(_) = ctx.load_artifact(&req) {
                return Err(anyhow!(
                    "Pass '{}' requires artifact '{}' which is not available. Run prerequisite passes first.",
                    name, req
                ));
            }
        }

        // Execute
        pass.run(&mut ctx)?;

        // Record produced artifacts in manifest
        for prod in pass.produces() {
            if let Some(content) = ctx.artifacts.get(&prod) {
                let filename = format!("{}.yaml", prod);
                manifest.record_artifact(&prod.to_string(), name, &filename, content);
            }
        }

        manifest.save(project_root)?;
        Ok(())
    }

    /// Run all passes in dependency order
    pub fn run_all(&mut self, project_root: &Path) -> Result<()> {
        self.resolve_order()?;

        let order = self.execution_order.clone();
        for name in &order {
            println!("  ▶ Running pass: {}", name);
            self.run_pass(name, project_root)?;
        }

        Ok(())
    }

    /// Run only the passes for a specific phase prefix (e.g., "diverge")
    pub fn run_phase(&mut self, phase: &str, project_root: &Path) -> Result<()> {
        self.resolve_order()?;

        let matching: Vec<String> = self.execution_order.iter()
            .filter(|name| name.starts_with(phase))
            .cloned()
            .collect();

        if matching.is_empty() {
            return Err(anyhow!("No passes found matching phase '{}'", phase));
        }

        for name in &matching {
            println!("  ▶ Running pass: {}", name);
            self.run_pass(name, project_root)?;
        }

        Ok(())
    }
}
