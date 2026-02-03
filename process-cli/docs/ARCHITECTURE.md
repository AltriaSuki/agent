# Process CLI Architecture

## Overview

Process CLI is designed with extensibility as a core principle. The system uses trait-based abstractions for all major extension points, allowing new functionality to be added without modifying core code.

## Design Principles

1. **Trait-based Extension** - All extension points are defined as traits
2. **Registry Pattern** - Components are registered at startup, enabling runtime discovery
3. **Layered Configuration** - Multiple config sources with clear precedence
4. **Separation of Concerns** - Each crate has a single responsibility

---

## Crate Structure

### `process-core`

Core domain logic with no external dependencies beyond std.

```
process-core/
├── phase.rs      # Phase enum and state machine
├── branch.rs     # Branch struct and status
├── paths.rs      # Standard file paths
└── error.rs      # Domain errors
```

**Key Types:**
- `Phase` - Enum representing workflow phases (Seed → Diverge → Converge → Skeleton → Branching → Stabilize → Postmortem → Done)
- `PhaseRequirement` - Specifies what phase a command requires
- `PhaseState` - Manages phase persistence
- `Branch` - Branch hypothesis and metadata
- `ProcessPaths` - Standard paths for all process files

### `process-config`

Layered configuration system.

```
process-config/
├── config.rs     # Config structs
└── error.rs      # Config errors
```

**Config Precedence (highest to lowest):**
1. Environment variables (`PROCESS_CLI_*`, `ANTHROPIC_API_KEY`, etc.)
2. Project config (`.process/config.yaml`)
3. Global config (`~/.config/process-cli/config.yaml`)
4. Built-in defaults

### `process-ai`

AI provider abstraction layer.

```
process-ai/
├── provider.rs   # AiProvider trait
├── registry.rs   # Provider registry
└── providers/    # Built-in providers
    ├── claude_cli.rs
    ├── claude_api.rs
    ├── openai.rs
    ├── ollama.rs
    └── manual.rs
```

### `process-generators`

Code and config generators.

```
process-generators/
├── generator.rs  # Generator trait
├── registry.rs   # Generator registry
└── generators/   # Built-in generators
    ├── git_hooks.rs
    ├── ci_cd.rs
    ├── makefile.rs
    └── ide.rs
```

### `process-checks`

Automated checks and validations.

```
process-checks/
├── check.rs      # Check trait
├── registry.rs   # Check registry
└── checks/       # Built-in checks
    ├── lint.rs
    ├── test.rs
    ├── sensitive.rs
    ├── todo.rs
    ├── audit.rs
    └── coverage.rs
```

### `process-reviews`

Review templates for multi-perspective code review.

```
process-reviews/
├── template.rs   # ReviewTemplate trait
├── registry.rs   # Template registry
└── templates/    # Built-in templates
    ├── general.rs
    ├── security.rs
    ├── performance.rs
    └── architecture.rs
```

---

## Core Traits

### 1. Command Trait

```rust
#[async_trait]
pub trait Command: Send + Sync {
    /// Command name (e.g., "init", "diverge")
    fn name(&self) -> &'static str;

    /// Alternative names
    fn aliases(&self) -> &[&'static str] { &[] }

    /// Phase requirement for this command
    fn required_phase(&self) -> Option<PhaseRequirement>;

    /// Execute the command
    async fn execute(&self, ctx: &AppContext, args: &CommandArgs) -> Result<()>;
}
```

### 2. AiProvider Trait

```rust
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Provider name (e.g., "claude-cli", "openai")
    fn name(&self) -> &'static str;

    /// Priority for auto-detection (higher = preferred)
    fn priority(&self) -> u8;

    /// Check if provider is available
    async fn is_available(&self) -> bool;

    /// Send completion request
    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse>;
}
```

**Built-in Providers:**

| Provider | Priority | Detection |
|----------|----------|-----------|
| Claude CLI | 100 | `which claude` |
| Claude API | 90 | `ANTHROPIC_API_KEY` env var |
| OpenAI | 80 | `OPENAI_API_KEY` env var |
| Ollama | 70 | `http://localhost:11434` reachable |
| Manual | 0 | Always available (fallback) |

### 3. Generator Trait

```rust
pub trait Generator: Send + Sync {
    /// Generator name (e.g., "git-hooks", "ci")
    fn name(&self) -> &'static str;

    /// Category for grouping (e.g., "git", "ci", "ide")
    fn category(&self) -> &'static str;

    /// Check if generator applies to this project
    fn is_applicable(&self, project: &ProjectInfo) -> bool;

    /// Generate output
    fn generate(&self, ctx: &GeneratorContext) -> Result<GeneratorOutput>;
}
```

**Built-in Generators:**

| Generator | Output | Description |
|-----------|--------|-------------|
| `git-hooks` | `.git/hooks/pre-commit`, `pre-push` | Git hooks with fmt, lint, test, sensitive check |
| `ci-github` | `.github/workflows/ci.yml` | GitHub Actions CI |
| `ci-gitlab` | `.gitlab-ci.yml` | GitLab CI |
| `makefile` | `Makefile` | Standard make targets |
| `ide-vscode` | `.vscode/settings.json`, `tasks.json` | VS Code config |

### 4. Check Trait

```rust
#[async_trait]
pub trait Check: Send + Sync {
    /// Check name (e.g., "lint", "sensitive")
    fn name(&self) -> &'static str;

    /// Category for grouping
    fn category(&self) -> CheckCategory;

    /// Default severity of issues found
    fn severity(&self) -> Severity;

    /// Run the check
    async fn run(&self, ctx: &CheckContext) -> Result<CheckResult>;

    /// Whether this check supports auto-fix
    fn can_auto_fix(&self) -> bool { false }

    /// Attempt to auto-fix issues
    async fn auto_fix(&self, issues: &[Issue]) -> Result<FixResult> {
        Err(anyhow!("Auto-fix not supported"))
    }
}
```

**Built-in Checks:**

| Check | Category | Severity | Auto-fix |
|-------|----------|----------|----------|
| `lint` | Lint | Error | Partial |
| `test` | Test | Error | No |
| `sensitive` | Security | Critical | No |
| `todo` | Style | Info | No |
| `audit` | Security | Warning | No |
| `coverage` | Test | Warning | No |

**Sensitive Info Patterns:**
```
password|passwd|pwd\s*=
api[_-]?key\s*=
secret|token\s*=
aws[_-]?(access[_-]?key|secret)
-----BEGIN (RSA |DSA |EC )?PRIVATE KEY-----
jdbc://[^:]+:[^@]+@
```

### 5. ReviewTemplate Trait

```rust
pub trait ReviewTemplate: Send + Sync {
    /// Template name (e.g., "general", "security")
    fn name(&self) -> &'static str;

    /// Review perspectives to use
    fn perspectives(&self) -> &[ReviewPerspective];

    /// Build the prompt for AI review
    fn build_prompt(&self, ctx: &ReviewContext) -> Result<String>;

    /// Parse AI response into structured result
    fn parse_response(&self, response: &str) -> Result<ReviewResult>;
}
```

**Built-in Templates:**

| Template | Perspectives |
|----------|--------------|
| `general` | Security Auditor, Performance Engineer, User Advocate, Maintainability Expert |
| `security` | Injection, Auth/Authz, Data Protection, CSRF, Rate Limiting |
| `performance` | N+1 Queries, Algorithm Complexity, Caching, Memory Leaks |
| `architecture` | SOLID Principles, Module Design, Coupling, Testability |

---

## Registry Pattern

All extensible components use a registry for discovery and management:

```rust
pub struct Registry<T: ?Sized> {
    items: Vec<Box<T>>,
}

impl<T: ?Sized> Registry<T> {
    pub fn new() -> Self { ... }
    pub fn register(&mut self, item: Box<T>) { ... }
    pub fn get(&self, name: &str) -> Option<&T> { ... }
    pub fn list(&self) -> Vec<&T> { ... }
}
```

At startup, `main.rs` creates all registries and populates them:

```rust
fn create_app() -> App {
    let config = load_config();

    let mut ai_registry = AiRegistry::new();
    ai_registry.register(Box::new(ClaudeCliProvider::new(&config)));
    ai_registry.register(Box::new(ClaudeApiProvider::new(&config)));
    // ... more providers

    let mut generator_registry = GeneratorRegistry::new();
    generator_registry.register(Box::new(GitHooksGenerator::new()));
    // ... more generators

    App {
        config,
        ai: ai_registry,
        generators: generator_registry,
        checks: check_registry,
        reviews: review_registry,
    }
}
```

---

## Phase State Machine

```
┌─────────────────┐
│  Uninitialized  │
└────────┬────────┘
         │ init
         ▼
┌─────────────────┐
│      Seed       │ ◀─── Edit seed.yaml
└────────┬────────┘
         │ diverge
         ▼
┌─────────────────┐
│     Diverge     │ ◀─── AI generates proposals
└────────┬────────┘
         │ converge
         ▼
┌─────────────────┐
│    Converge     │ ◀─── AI extracts rules
└────────┬────────┘
         │ skeleton
         ▼
┌─────────────────┐
│    Skeleton     │ ◀─── Generate project structure
└────────┬────────┘
         │ branch-new
         ▼
┌─────────────────┐
│    Branching    │ ◀─── Loop: new → start → review → abuse → gate → merge
└────────┬────────┘
         │ stabilize
         ▼
┌─────────────────┐
│    Stabilize    │ ◀─── Freeze invariants, bugfix only
└────────┬────────┘
         │ postmortem
         ▼
┌─────────────────┐
│   Postmortem    │ ◀─── AI generates retrospective
└────────┬────────┘
         │ done
         ▼
┌─────────────────┐
│      Done       │
└─────────────────┘
```

---

## File Layout

```
.process/
├── .state                    # Current phase (single line)
├── .ai_config               # AI configuration (legacy, use config.yaml)
├── config.yaml              # Project configuration
├── seed.yaml                # Project seed/hypothesis
├── diverge_summary.yaml     # Divergent proposals
├── rules.yaml               # Extracted rules/invariants
├── skeleton.yaml            # Project structure definition
├── decisions_log.yaml       # Decision log
├── friction.yaml            # Friction points
├── learnings.yaml           # Lessons learned
├── postmortem.yaml          # Retrospective
├── REJECTED_APPROACHES.md   # Rejected approaches log
└── branches/
    ├── feature-x.yaml       # Branch definition
    ├── feature-x-review.yaml
    └── feature-x-abuse.yaml
```

---

## Configuration Schema

```yaml
ai:
  provider: auto  # auto | claude | openai | ollama | manual
  claude:
    model: claude-sonnet-4-20250514
    api_key: ""        # or ANTHROPIC_API_KEY
    base_url: ""       # or ANTHROPIC_BASE_URL
    max_tokens: 8192
  openai:
    model: gpt-4o
    api_key: ""        # or OPENAI_API_KEY
    base_url: ""       # or OPENAI_BASE_URL
    max_tokens: 8192
  ollama:
    model: llama3
    endpoint: http://localhost:11434
  timeout_secs: 120

generators:
  git_hooks:
    pre_commit:
      run_fmt: true
      run_lint: true
      run_tests: true
      check_sensitive: true
      check_todo: true

checks:
  gate_checks:
    - lint
    - test
    - sensitive
    - audit
  fail_threshold: error  # info | warning | error | critical

reviews:
  default_template: general
```

---

## Dependencies

```toml
[workspace.dependencies]
# Async
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"

# CLI
clap = { version = "4", features = ["derive", "env"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1"

# HTTP
reqwest = { version = "0.12", features = ["json"] }

# Error handling
anyhow = "1"
thiserror = "1"

# Templates
tera = "1.19"

# Text processing
regex = "1"

# Date/time
chrono = { version = "0.4", features = ["serde"] }

# Terminal
colored = "2"

# File system
dirs = "5"
```
