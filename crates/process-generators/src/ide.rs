use crate::{Generator, GeneratedFile};
use anyhow::Result;
use std::path::Path;

/// Generates VS Code workspace settings, tasks, and recommended extensions
pub struct IdeGenerator;

impl Generator for IdeGenerator {
    fn name(&self) -> &'static str {
        "ide"
    }

    fn description(&self) -> &'static str {
        "Generate VS Code settings, tasks, and extensions"
    }

    fn generate(&self, project_root: &Path) -> Result<Vec<GeneratedFile>> {
        let mut files = Vec::new();
        let has_cargo = project_root.join("Cargo.toml").exists();

        // Extensions
        let extensions = if has_cargo {
            EXTENSIONS_RUST.to_string()
        } else {
            EXTENSIONS_GENERIC.to_string()
        };

        let ext_path = ".vscode/extensions.json";
        files.push(GeneratedFile {
            path: ext_path.to_string(),
            content: extensions,
            overwritten: project_root.join(ext_path).exists(),
        });

        // Settings
        let settings = if has_cargo {
            SETTINGS_RUST.to_string()
        } else {
            SETTINGS_GENERIC.to_string()
        };

        let settings_path = ".vscode/settings.json";
        files.push(GeneratedFile {
            path: settings_path.to_string(),
            content: settings,
            overwritten: project_root.join(settings_path).exists(),
        });

        Ok(files)
    }
}

const EXTENSIONS_RUST: &str = r#"{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "tamasfe.even-better-toml",
    "serayuzgur.crates",
    "vadimcn.vscode-lldb"
  ]
}
"#;

const EXTENSIONS_GENERIC: &str = r#"{
  "recommendations": [
    "editorconfig.editorconfig",
    "streetsidesoftware.code-spell-checker"
  ]
}
"#;

const SETTINGS_RUST: &str = r#"{
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "rust-lang.rust-analyzer",
  "[rust]": {
    "editor.formatOnSave": true
  },
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.cargo.allTargets": true,
  "files.watcherExclude": {
    "**/target/**": true
  }
}
"#;

const SETTINGS_GENERIC: &str = r#"{
  "editor.formatOnSave": true,
  "editor.tabSize": 2,
  "files.trimTrailingWhitespace": true,
  "files.insertFinalNewline": true
}
"#;
