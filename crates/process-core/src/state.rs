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
        let process_dir = Path::new(".process");
        if !process_dir.exists() {
            anyhow::bail!("Not a process project (no .process/ directory). Run 'process init' first.");
        }
        let path = process_dir.join(".state.yaml");
        if !path.exists() {
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

    pub fn check_phase(&self, expected: Phase) -> Result<()> {
        if self.current_phase < expected {
            let msg = format!("Process not ready. Current: {}, Required: {}. Run previous steps first.", self.current_phase, expected);
            // using anyhow::bail! equivalent
            return Err(anyhow::anyhow!(msg));
        }
        Ok(())
    }

    pub fn set_phase(&mut self, phase: Phase) {
        if phase > self.current_phase {
            self.current_phase = phase;
            self.last_updated = Utc::now();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Phase ordering tests --

    #[test]
    fn test_phase_ordering() {
        assert!(Phase::Seed < Phase::Diverge);
        assert!(Phase::Diverge < Phase::Converge);
        assert!(Phase::Converge < Phase::Skeleton);
        assert!(Phase::Skeleton < Phase::Branching);
        assert!(Phase::Branching < Phase::Stabilize);
        assert!(Phase::Stabilize < Phase::Postmortem);
        assert!(Phase::Postmortem < Phase::Done);
    }

    #[test]
    fn test_phase_equality() {
        assert_eq!(Phase::Seed, Phase::Seed);
        assert_ne!(Phase::Seed, Phase::Diverge);
    }

    // -- set_phase tests --

    #[test]
    fn test_set_phase_advances_forward() {
        let mut state = ProcessState::default();
        assert_eq!(state.current_phase, Phase::Seed);

        state.set_phase(Phase::Diverge);
        assert_eq!(state.current_phase, Phase::Diverge);

        state.set_phase(Phase::Skeleton);
        assert_eq!(state.current_phase, Phase::Skeleton);
    }

    #[test]
    fn test_set_phase_refuses_backward() {
        let mut state = ProcessState::default();
        state.set_phase(Phase::Converge);
        assert_eq!(state.current_phase, Phase::Converge);

        // Try going backward — should be silently ignored
        state.set_phase(Phase::Seed);
        assert_eq!(state.current_phase, Phase::Converge);

        state.set_phase(Phase::Diverge);
        assert_eq!(state.current_phase, Phase::Converge);
    }

    #[test]
    fn test_set_phase_same_phase_noop() {
        let mut state = ProcessState::default();
        state.set_phase(Phase::Diverge);
        let ts = state.last_updated;

        // Same phase — should not update timestamp
        state.set_phase(Phase::Diverge);
        assert_eq!(state.current_phase, Phase::Diverge);
        assert_eq!(state.last_updated, ts);
    }

    // -- check_phase tests --

    #[test]
    fn test_check_phase_ok_when_at_or_beyond() {
        let mut state = ProcessState::default();
        state.set_phase(Phase::Converge);

        // At exact phase — OK
        assert!(state.check_phase(Phase::Converge).is_ok());
        // Beyond required phase — also OK
        assert!(state.check_phase(Phase::Seed).is_ok());
        assert!(state.check_phase(Phase::Diverge).is_ok());
    }

    #[test]
    fn test_check_phase_err_when_behind() {
        let state = ProcessState::default();
        assert_eq!(state.current_phase, Phase::Seed);

        let result = state.check_phase(Phase::Diverge);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("not ready"));
    }

    // -- load/save tests (combined to avoid set_current_dir race) --

    #[test]
    fn test_load_save_lifecycle() {
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();

        // 1. No .process/ dir → error
        let result = ProcessState::load();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not a process project"));

        // 2. .process/ exists but no state file → default
        std::fs::create_dir_all(".process").unwrap();
        let state = ProcessState::load().unwrap();
        assert_eq!(state.current_phase, Phase::Seed);

        // 3. Save and reload → roundtrip
        let mut state = ProcessState::default();
        state.set_phase(Phase::Skeleton);
        state.metadata.insert("key".to_string(), "value".to_string());
        state.save().unwrap();

        let loaded = ProcessState::load().unwrap();
        assert_eq!(loaded.current_phase, Phase::Skeleton);
        assert_eq!(loaded.metadata.get("key").unwrap(), "value");
    }

    // -- Default tests --

    #[test]
    fn test_default_state() {
        let state = ProcessState::default();
        assert_eq!(state.current_phase, Phase::Seed);
        assert!(state.metadata.is_empty());
    }

    // -- Serialization tests --

    #[test]
    fn test_phase_display() {
        assert_eq!(format!("{}", Phase::Seed), "0. Seed");
        assert_eq!(format!("{}", Phase::Done), "7. Done");
    }

    #[test]
    fn test_phase_yaml_roundtrip() {
        let yaml = serde_yaml::to_string(&Phase::Diverge).unwrap();
        let parsed: Phase = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed, Phase::Diverge);
    }
}

