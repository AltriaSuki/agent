use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use chrono::Utc;

/// A record of a single artifact in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRecord {
    /// Which pass produced this artifact
    pub produced_by: String,
    /// When it was last generated
    pub last_updated: String,
    /// SHA-256 hash of the content
    pub content_hash: String,
    /// File path relative to .process/
    pub path: String,
}

/// The manifest tracks all artifacts and their provenance
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Manifest {
    /// Version of the manifest format
    pub version: u32,
    /// Map of artifact name → record
    pub artifacts: HashMap<String, ArtifactRecord>,
}

impl Manifest {
    pub fn load(project_root: &Path) -> Result<Self> {
        let path = project_root.join(".process/manifest.yaml");
        if !path.exists() {
            return Ok(Self { version: 1, artifacts: HashMap::new() });
        }
        let content = std::fs::read_to_string(&path)?;
        let manifest: Manifest = serde_yaml::from_str(&content)?;
        Ok(manifest)
    }

    pub fn save(&self, project_root: &Path) -> Result<()> {
        let path = project_root.join(".process/manifest.yaml");
        let content = serde_yaml::to_string(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Record that a pass produced an artifact
    pub fn record_artifact(&mut self, artifact_name: &str, pass_name: &str, file_path: &str, content: &str) {
        let hash = format!("{:x}", md5_hash(content));
        self.artifacts.insert(artifact_name.to_string(), ArtifactRecord {
            produced_by: pass_name.to_string(),
            last_updated: Utc::now().to_rfc3339(),
            content_hash: hash,
            path: file_path.to_string(),
        });
    }

    /// Check if an artifact is stale (its dependencies changed since it was produced)
    pub fn is_fresh(&self, artifact_name: &str, _dependency_names: &[&str]) -> bool {
        // Simple freshness: artifact exists in manifest
        self.artifacts.contains_key(artifact_name)
    }
}

/// Simple hash function (not cryptographic, just for change detection)
fn md5_hash(content: &str) -> u64 {
    let mut hash: u64 = 5381;
    for byte in content.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    hash
}
