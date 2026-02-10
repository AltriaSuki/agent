use crate::{Generator, GeneratedFile};
use anyhow::Result;
use std::path::Path;

/// Generates a Makefile with standard targets
pub struct MakefileGenerator;

impl Generator for MakefileGenerator {
    fn name(&self) -> &'static str {
        "makefile"
    }

    fn description(&self) -> &'static str {
        "Generate Makefile with standard targets"
    }

    fn generate(&self, project_root: &Path) -> Result<Vec<GeneratedFile>> {
        let has_cargo = project_root.join("Cargo.toml").exists();
        let has_package_json = project_root.join("package.json").exists();

        let content = if has_cargo {
            MAKEFILE_RUST.to_string()
        } else if has_package_json {
            MAKEFILE_NODE.to_string()
        } else {
            MAKEFILE_GENERIC.to_string()
        };

        let overwritten = project_root.join("Makefile").exists();

        Ok(vec![GeneratedFile {
            path: "Makefile".to_string(),
            content,
            overwritten,
        }])
    }
}

const MAKEFILE_RUST: &str = r#".PHONY: build test lint fmt clean run check

build:
	cargo build

release:
	cargo build --release

test:
	cargo test --workspace

lint:
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

clean:
	cargo clean

run:
	cargo run

check: fmt-check lint test
	@echo "✅ All checks passed"
"#;

const MAKEFILE_NODE: &str = r#".PHONY: build test lint clean dev

install:
	npm ci

build:
	npm run build

test:
	npm test

lint:
	npm run lint

dev:
	npm run dev

clean:
	rm -rf node_modules dist

check: lint test
	@echo "✅ All checks passed"
"#;

const MAKEFILE_GENERIC: &str = r#".PHONY: build test lint clean

build:
	@echo "Add build command"

test:
	@echo "Add test command"

lint:
	@echo "Add lint command"

clean:
	@echo "Add clean command"

check: lint test
	@echo "✅ All checks passed"
"#;
