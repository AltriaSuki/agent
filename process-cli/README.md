# Process CLI - Rust Rewrite Design

> **Status: Design Document**
> This directory contains architecture and API documentation for the planned Rust rewrite of `process-cli.sh`.

## Overview

Process CLI is an extensible development workflow tool. This design document describes the planned Rust implementation with trait-based extensibility.

## Documentation

| Document | Description |
|----------|-------------|
| [Architecture](docs/ARCHITECTURE.md) | System design, crate structure, core traits |
| [Commands](docs/COMMANDS.md) | Complete command reference |
| [Configuration](docs/CONFIGURATION.md) | Configuration options and examples |
| [Extending](docs/EXTENDING.md) | How to add providers, generators, checks |

## Planned Project Structure

```
process-cli/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── process-core/             # Phase state machine, Branch, Paths
│   ├── process-ai/               # AI Provider abstraction
│   ├── process-generators/       # Code/config generators
│   ├── process-checks/           # Automated checks
│   ├── process-reviews/          # Review templates
│   └── process-config/           # Configuration system
├── src/                          # Main CLI binary
│   ├── main.rs
│   ├── cli.rs
│   ├── commands/
│   └── output.rs
└── templates/                    # Tera templates
```

## Core Extension Points

### 1. AI Providers
```rust
#[async_trait]
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn priority(&self) -> u8;
    async fn is_available(&self) -> bool;
    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse>;
}
```

### 2. Generators
```rust
pub trait Generator: Send + Sync {
    fn name(&self) -> &'static str;
    fn category(&self) -> &'static str;
    fn is_applicable(&self, project: &ProjectInfo) -> bool;
    fn generate(&self, ctx: &GeneratorContext) -> Result<GeneratorOutput>;
}
```

### 3. Checks
```rust
#[async_trait]
pub trait Check: Send + Sync {
    fn name(&self) -> &'static str;
    fn category(&self) -> CheckCategory;
    fn severity(&self) -> Severity;
    async fn run(&self, ctx: &CheckContext) -> Result<CheckResult>;
    fn can_auto_fix(&self) -> bool { false }
}
```

### 4. Review Templates
```rust
pub trait ReviewTemplate: Send + Sync {
    fn name(&self) -> &'static str;
    fn perspectives(&self) -> &[ReviewPerspective];
    fn build_prompt(&self, ctx: &ReviewContext) -> Result<String>;
    fn parse_response(&self, response: &str) -> Result<ReviewResult>;
}
```

## Key Dependencies

```toml
tokio = "1"           # Async runtime
clap = "4"            # CLI framework
serde = "1"           # Serialization
serde_yaml = "0.9"    # YAML support
reqwest = "0.12"      # HTTP client
tera = "1.19"         # Template engine
anyhow = "1"          # Error handling
colored = "2"         # Terminal colors
```

## Current Implementation

The working implementation is in the parent directory:
- `../process-cli.sh` - Bash implementation (1,519 lines)

## Reference Files

- `../auto/1e-project/githooks/pre-commit` - Git hooks reference
- `../auto/1e-project/github-workflows/ci.yml` - CI reference
- `../auto/universal/scripts/prepare-review.sh` - Review template reference
