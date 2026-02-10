use crate::{Generator, GeneratedFile};
use anyhow::Result;
use std::path::Path;

/// Generates CI/CD configuration files (GitHub Actions / GitLab CI)
pub struct CiCdGenerator;

impl Generator for CiCdGenerator {
    fn name(&self) -> &'static str {
        "cicd"
    }

    fn description(&self) -> &'static str {
        "Generate CI/CD pipeline configuration"
    }

    fn generate(&self, project_root: &Path) -> Result<Vec<GeneratedFile>> {
        let mut files = Vec::new();
        let has_cargo = project_root.join("Cargo.toml").exists();
        let has_package_json = project_root.join("package.json").exists();

        // Always generate GitHub Actions (most common)
        let workflow = if has_cargo {
            GITHUB_ACTIONS_RUST.to_string()
        } else if has_package_json {
            GITHUB_ACTIONS_NODE.to_string()
        } else {
            GITHUB_ACTIONS_GENERIC.to_string()
        };

        let path = ".github/workflows/ci.yml";
        let overwritten = project_root.join(path).exists();

        files.push(GeneratedFile {
            path: path.to_string(),
            content: workflow,
            overwritten,
        });

        Ok(files)
    }
}

const GITHUB_ACTIONS_RUST: &str = r#"name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - uses: Swatinem/rust-cache@v2

      - name: Format
        run: cargo fmt --all --check

      - name: Clippy
        run: cargo clippy --all-targets -- -D warnings

      - name: Test
        run: cargo test --workspace

      - name: Build
        run: cargo build --release
"#;

const GITHUB_ACTIONS_NODE: &str = r#"name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'

      - run: npm ci
      - run: npm run lint
      - run: npm test
      - run: npm run build
"#;

const GITHUB_ACTIONS_GENERIC: &str = r#"name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      # Add your build and test steps here
"#;
