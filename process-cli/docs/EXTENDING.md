# Extending Process CLI

This guide explains how to add new functionality to Process CLI.

## Table of Contents

1. [Adding a New AI Provider](#adding-a-new-ai-provider)
2. [Adding a New Generator](#adding-a-new-generator)
3. [Adding a New Check](#adding-a-new-check)
4. [Adding a New Review Template](#adding-a-new-review-template)
5. [Adding a New Command](#adding-a-new-command)

---

## Adding a New AI Provider

### Step 1: Implement the AiProvider trait

Create a new file in `crates/process-ai/src/providers/`:

```rust
// crates/process-ai/src/providers/my_provider.rs

use async_trait::async_trait;
use crate::{AiError, AiProvider, CompletionRequest, CompletionResponse};

pub struct MyProvider {
    config: Config,
}

impl MyProvider {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

#[async_trait]
impl AiProvider for MyProvider {
    fn name(&self) -> &'static str {
        "my-provider"
    }

    fn priority(&self) -> u8 {
        75  // Between Ollama (70) and OpenAI (80)
    }

    async fn is_available(&self) -> bool {
        // Check if this provider can be used
        // e.g., check for API key, running service, etc.
        std::env::var("MY_PROVIDER_API_KEY").is_ok()
    }

    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse, AiError> {
        // Implement the actual API call
        let api_key = std::env::var("MY_PROVIDER_API_KEY")
            .map_err(|_| AiError::Other("MY_PROVIDER_API_KEY not set".into()))?;

        // Make HTTP request to your provider
        let client = reqwest::Client::new();
        let response = client
            .post("https://api.myprovider.com/v1/complete")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "prompt": request.prompt,
                "max_tokens": request.max_tokens.unwrap_or(4096)
            }))
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let content = json["text"].as_str()
            .ok_or_else(|| AiError::ApiError("Invalid response".into()))?
            .to_string();

        Ok(CompletionResponse {
            content,
            usage: None,
        })
    }
}
```

### Step 2: Register the provider

Add to `crates/process-ai/src/providers/mod.rs`:

```rust
mod my_provider;
pub use my_provider::MyProvider;
```

Add to `crates/process-ai/src/lib.rs`:

```rust
pub fn create_registry(config: &Config) -> AiRegistry {
    let mut registry = AiRegistry::new();

    // ... existing providers ...
    registry.register(Box::new(providers::MyProvider::new(config.clone())));

    registry
}
```

### Step 3: Add configuration (optional)

Add to `crates/process-config/src/config.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MyProviderConfig {
    pub api_key: String,
    pub model: String,
    pub endpoint: String,
}
```

---

## Adding a New Generator

### Step 1: Implement the Generator trait

Create a new file in `crates/process-generators/src/generators/`:

```rust
// crates/process-generators/src/generators/docker.rs

use crate::{Generator, GeneratorContext, GeneratorOutput, ProjectInfo};
use anyhow::Result;

pub struct DockerGenerator;

impl DockerGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Generator for DockerGenerator {
    fn name(&self) -> &'static str {
        "docker"
    }

    fn category(&self) -> &'static str {
        "container"
    }

    fn is_applicable(&self, project: &ProjectInfo) -> bool {
        // Only generate if it's a deployable application
        project.has_file("Cargo.toml") && !project.has_file("Dockerfile")
    }

    fn generate(&self, ctx: &GeneratorContext) -> Result<GeneratorOutput> {
        let dockerfile = self.generate_dockerfile(ctx)?;
        let dockerignore = self.generate_dockerignore()?;
        let compose = self.generate_compose(ctx)?;

        Ok(GeneratorOutput {
            files: vec![
                ("Dockerfile".into(), dockerfile),
                (".dockerignore".into(), dockerignore),
                ("docker-compose.yml".into(), compose),
            ],
        })
    }
}

impl DockerGenerator {
    fn generate_dockerfile(&self, ctx: &GeneratorContext) -> Result<String> {
        // Use Tera template or generate directly
        let template = include_str!("../../templates/docker/Dockerfile.tera");
        let mut tera = tera::Tera::default();
        tera.add_raw_template("Dockerfile", template)?;

        let mut context = tera::Context::new();
        context.insert("project_name", &ctx.project_name);
        context.insert("rust_version", "1.75");

        Ok(tera.render("Dockerfile", &context)?)
    }

    fn generate_dockerignore(&self) -> Result<String> {
        Ok(r#"target/
.git/
.process/
*.log
"#.to_string())
    }

    fn generate_compose(&self, ctx: &GeneratorContext) -> Result<String> {
        Ok(format!(r#"version: '3.8'
services:
  app:
    build: .
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
"#))
    }
}
```

### Step 2: Create template file (optional)

Create `templates/docker/Dockerfile.tera`:

```dockerfile
# Build stage
FROM rust:{{ rust_version }} as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/{{ project_name }} /usr/local/bin/
CMD ["{{ project_name }}"]
```

### Step 3: Register the generator

Add to `crates/process-generators/src/generators/mod.rs`:

```rust
mod docker;
pub use docker::DockerGenerator;
```

### Step 4: Add CLI subcommand

Add to `src/cli.rs`:

```rust
#[derive(Subcommand)]
enum GenerateCommand {
    // ... existing commands ...

    /// Generate Docker configuration
    Docker,
}
```

---

## Adding a New Check

### Step 1: Implement the Check trait

Create a new file in `crates/process-checks/src/checks/`:

```rust
// crates/process-checks/src/checks/complexity.rs

use async_trait::async_trait;
use crate::{Check, CheckCategory, CheckContext, CheckResult, Issue, Severity};
use anyhow::Result;

pub struct ComplexityCheck {
    max_complexity: u32,
}

impl ComplexityCheck {
    pub fn new() -> Self {
        Self { max_complexity: 10 }
    }

    pub fn with_max_complexity(mut self, max: u32) -> Self {
        self.max_complexity = max;
        self
    }
}

#[async_trait]
impl Check for ComplexityCheck {
    fn name(&self) -> &'static str {
        "complexity"
    }

    fn category(&self) -> CheckCategory {
        CheckCategory::Style
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    async fn run(&self, ctx: &CheckContext) -> Result<CheckResult> {
        let mut issues = Vec::new();

        // Find Rust files and analyze complexity
        for file in ctx.find_files("**/*.rs")? {
            let content = std::fs::read_to_string(&file)?;

            // Simple heuristic: count nested braces
            let complexity = self.calculate_complexity(&content);

            if complexity > self.max_complexity {
                issues.push(Issue {
                    file: Some(file.display().to_string()),
                    line: None,
                    severity: self.severity(),
                    message: format!(
                        "Cyclomatic complexity {} exceeds threshold {}",
                        complexity, self.max_complexity
                    ),
                    suggestion: Some("Consider breaking this into smaller functions".into()),
                });
            }
        }

        Ok(CheckResult {
            passed: issues.is_empty(),
            issues,
            summary: format!("Analyzed {} files", ctx.file_count()),
        })
    }

    fn can_auto_fix(&self) -> bool {
        false  // Complexity issues require manual refactoring
    }
}

impl ComplexityCheck {
    fn calculate_complexity(&self, content: &str) -> u32 {
        // Simplified complexity calculation
        // Real implementation would use a proper parser
        let mut complexity = 1;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("if ")
                || trimmed.starts_with("else if ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("for ")
                || trimmed.starts_with("match ")
                || trimmed.contains(" && ")
                || trimmed.contains(" || ")
            {
                complexity += 1;
            }
        }
        complexity
    }
}
```

### Step 2: Register the check

Add to `crates/process-checks/src/checks/mod.rs`:

```rust
mod complexity;
pub use complexity::ComplexityCheck;
```

### Step 3: Add to default gate checks (optional)

In `crates/process-config/src/config.rs`:

```rust
fn default_gate_checks() -> Vec<String> {
    vec![
        "lint".to_string(),
        "test".to_string(),
        "sensitive".to_string(),
        "audit".to_string(),
        "complexity".to_string(),  // Add here
    ]
}
```

---

## Adding a New Review Template

### Step 1: Implement the ReviewTemplate trait

Create a new file in `crates/process-reviews/src/templates/`:

```rust
// crates/process-reviews/src/templates/accessibility.rs

use crate::{ReviewTemplate, ReviewContext, ReviewPerspective, ReviewResult};
use anyhow::Result;

pub struct AccessibilityReviewTemplate;

impl ReviewTemplate for AccessibilityReviewTemplate {
    fn name(&self) -> &'static str {
        "accessibility"
    }

    fn perspectives(&self) -> &[ReviewPerspective] {
        &[
            ReviewPerspective {
                role: "Screen Reader User",
                focus: vec![
                    "Alt text for images",
                    "Semantic HTML structure",
                    "ARIA labels",
                    "Focus management",
                ],
            },
            ReviewPerspective {
                role: "Keyboard-Only User",
                focus: vec![
                    "Tab order",
                    "Focus visibility",
                    "Keyboard shortcuts",
                    "Skip links",
                ],
            },
            ReviewPerspective {
                role: "Color Blind User",
                focus: vec![
                    "Color contrast",
                    "Color-only information",
                    "Pattern differentiation",
                ],
            },
            ReviewPerspective {
                role: "Motor Impaired User",
                focus: vec![
                    "Click target size",
                    "Gesture alternatives",
                    "Time limits",
                    "Error recovery",
                ],
            },
        ]
    }

    fn build_prompt(&self, ctx: &ReviewContext) -> Result<String> {
        let perspectives = self.perspectives()
            .iter()
            .enumerate()
            .map(|(i, p)| {
                format!(
                    "Role {}: {} - Focus on: {}",
                    i + 1,
                    p.role,
                    p.focus.join(", ")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(format!(r#"
You are an accessibility expert. Review the following code changes from multiple perspectives.

--- CHANGES ---
{}
--- END CHANGES ---

Review Perspectives:
{}

Output YAML format:

reviews:
  - role: "Screen Reader User"
    verdict: "pass | conditional_pass | fail"
    issues:
      - severity: "high | medium | low"
        wcag_criterion: "1.1.1"
        description: "Issue description"
        suggestion: "How to fix"

overall_verdict: "pass | conditional_pass | fail"
wcag_compliance_level: "A | AA | AAA | none"
"#, ctx.diff, perspectives))
    }

    fn parse_response(&self, response: &str) -> Result<ReviewResult> {
        // Parse YAML response into ReviewResult
        let result: ReviewResult = serde_yaml::from_str(response)?;
        Ok(result)
    }
}
```

### Step 2: Register the template

Add to `crates/process-reviews/src/templates/mod.rs`:

```rust
mod accessibility;
pub use accessibility::AccessibilityReviewTemplate;
```

### Step 3: Add CLI option

The template becomes available via:
```bash
process branch review my-branch --template accessibility
```

---

## Adding a New Command

### Step 1: Create command implementation

Create a new file in `src/commands/`:

```rust
// src/commands/metrics.rs

use anyhow::Result;
use clap::Args;
use crate::context::AppContext;

#[derive(Args)]
pub struct MetricsArgs {
    /// Output format
    #[arg(long, default_value = "text")]
    format: String,

    /// Include historical data
    #[arg(long)]
    history: bool,
}

pub async fn execute(ctx: &AppContext, args: MetricsArgs) -> Result<()> {
    let paths = &ctx.paths;

    // Gather metrics
    let mut metrics = Metrics::default();

    // Count branches
    let branches = paths.list_branch_files()?;
    metrics.total_branches = branches.len();

    for branch_path in branches {
        let branch = Branch::load(&branch_path)?;
        match branch.status {
            BranchStatus::Merged => metrics.merged_branches += 1,
            BranchStatus::Rejected => metrics.rejected_branches += 1,
            _ => metrics.active_branches += 1,
        }
    }

    // Count learnings
    if paths.learnings_file().exists() {
        let content = std::fs::read_to_string(paths.learnings_file())?;
        metrics.learnings = content.matches("- timestamp:").count();
    }

    // Count friction points
    if paths.friction_file().exists() {
        let content = std::fs::read_to_string(paths.friction_file())?;
        metrics.friction_points = content.matches("- branch:").count();
    }

    // Output
    match args.format.as_str() {
        "json" => println!("{}", serde_json::to_string_pretty(&metrics)?),
        "yaml" => println!("{}", serde_yaml::to_string(&metrics)?),
        _ => {
            println!("Project Metrics");
            println!("===============");
            println!("Branches: {} total ({} merged, {} rejected, {} active)",
                metrics.total_branches,
                metrics.merged_branches,
                metrics.rejected_branches,
                metrics.active_branches
            );
            println!("Learnings recorded: {}", metrics.learnings);
            println!("Friction points: {}", metrics.friction_points);
        }
    }

    Ok(())
}

#[derive(Default, serde::Serialize)]
struct Metrics {
    total_branches: usize,
    merged_branches: usize,
    rejected_branches: usize,
    active_branches: usize,
    learnings: usize,
    friction_points: usize,
}
```

### Step 2: Add to CLI definition

In `src/cli.rs`:

```rust
#[derive(Subcommand)]
pub enum Command {
    // ... existing commands ...

    /// Show project metrics
    Metrics(commands::metrics::MetricsArgs),
}
```

### Step 3: Add to command dispatch

In `src/main.rs`:

```rust
match cli.command {
    // ... existing matches ...
    Command::Metrics(args) => commands::metrics::execute(&ctx, args).await?,
}
```

---

## Best Practices

### 1. Follow Existing Patterns
Look at existing implementations for guidance on error handling, logging, and output formatting.

### 2. Add Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_feature() {
        // ...
    }

    #[tokio::test]
    async fn test_async_feature() {
        // ...
    }
}
```

### 3. Update Documentation
- Add command to `docs/COMMANDS.md`
- Update `docs/ARCHITECTURE.md` if adding new extension points
- Add examples to README

### 4. Consider Configuration
If your extension has configurable options, add them to the config system rather than hardcoding values.

### 5. Handle Errors Gracefully
Use `anyhow::Result` for functions that can fail, and provide helpful error messages.
