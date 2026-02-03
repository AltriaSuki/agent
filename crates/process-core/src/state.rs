use crate::phase::Phase;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessState {
    pub current_phase: Phase,
    pub last_updated: DateTime<Utc>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl Default for ProcessState {
    fn default() -> Self {
        Self {
            current_phase: Phase::Seed,
            last_updated: Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }
}

impl ProcessState {
    pub fn load() -> Result<Self> {
        let path = Path::new(".process/.state.yaml");
        if !path.exists() {
            // Default if not exists, or error? 
            // Better to return default for now or specific error
             return Ok(Self::default());
        }
        let content = fs::read_to_string(path).context("Failed to read state file")?;
        let state: Self = serde_yaml::from_str(&content).context("Failed to parse state file")?;
        Ok(state)
    }

    pub fn save(&self) -> Result<()> {
        let path = Path::new(".process/.state.yaml");
        let content = serde_yaml::to_string(self).context("Failed to serialize state")?;
        fs::write(path, content).context("Failed to write state file")?;
        Ok(())
    }
}
