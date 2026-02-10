use anyhow::{Context, Result};
use include_dir::{include_dir, Dir};
use std::fs;
use std::path::Path;
use tera::Tera;

static BUILT_IN_TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates/prompts");

pub struct PromptEngine {
    provider: String,
}

impl PromptEngine {
    pub fn new(provider: &str) -> Self {
        let provider = if provider == "auto" {
            "_default".to_string()
        } else {
            provider.to_string()
        };
        Self { provider }
    }

    pub fn render(&self, template_name: &str, ctx: &tera::Context) -> Result<String> {
        let filename = format!("{}.md.tera", template_name);
        let template_content = self.resolve_template(&filename)?;

        let mut tera = Tera::default();
        tera.add_raw_template(&filename, &template_content)
            .with_context(|| format!("Failed to parse template '{}'", filename))?;

        tera.render(&filename, ctx)
            .with_context(|| format!("Failed to render template '{}'", filename))
    }

    fn resolve_template(&self, filename: &str) -> Result<String> {
        // 1. Project-local: .process/prompts/<provider>/
        let local_provider = Path::new(".process/prompts")
            .join(&self.provider)
            .join(filename);
        if local_provider.exists() {
            return fs::read_to_string(&local_provider)
                .with_context(|| format!("Failed to read {}", local_provider.display()));
        }

        // 2. Project-local: .process/prompts/_default/
        let local_default = Path::new(".process/prompts/_default").join(filename);
        if local_default.exists() {
            return fs::read_to_string(&local_default)
                .with_context(|| format!("Failed to read {}", local_default.display()));
        }

        // 3. Built-in: <provider>/
        let builtin_provider = format!("{}/{}", self.provider, filename);
        if let Some(file) = BUILT_IN_TEMPLATES.get_file(&builtin_provider) {
            return Ok(file
                .contents_utf8()
                .context("Built-in template is not valid UTF-8")?
                .to_string());
        }

        // 4. Built-in: _default/
        let builtin_default = format!("_default/{}", filename);
        if let Some(file) = BUILT_IN_TEMPLATES.get_file(&builtin_default) {
            return Ok(file
                .contents_utf8()
                .context("Built-in template is not valid UTF-8")?
                .to_string());
        }

        anyhow::bail!(
            "Template '{}' not found for provider '{}' in any lookup location",
            filename,
            self.provider
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_diverge_template() {
        let engine = PromptEngine::new("auto");
        let mut ctx = tera::Context::new();
        ctx.insert("seed", "name: test-project\ngoal: build something");
        let result = engine.render("diverge", &ctx).unwrap();
        assert!(result.contains("software architect"));
        assert!(result.contains("test-project"));
        assert!(result.contains("--- SEED ---"));
    }

    #[test]
    fn test_render_converge_template() {
        let engine = PromptEngine::new("auto");
        let mut ctx = tera::Context::new();
        ctx.insert("seed", "name: test");
        ctx.insert("diverge_summary", "proposals: []");
        let result = engine.render("converge", &ctx).unwrap();
        assert!(result.contains("--- SEED ---"));
        assert!(result.contains("--- DIVERGE SUMMARY ---"));
    }

    #[test]
    fn test_auto_provider_maps_to_default() {
        let engine = PromptEngine::new("auto");
        assert_eq!(engine.provider, "_default");
    }

    #[test]
    fn test_missing_template_errors() {
        let engine = PromptEngine::new("auto");
        let ctx = tera::Context::new();
        let result = engine.render("nonexistent", &ctx);
        assert!(result.is_err());
    }
}
