use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::collections::HashMap;

/// Kinds of artifacts produced/consumed by passes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArtifactKind {
    Seed,
    Proposals,
    Rules,
    Skeleton,
    DecisionLog,
    Learnings,
    Friction,
    BranchHypothesis(String),
    BranchReview(String),
    Postmortem,
    Custom(String),
}

impl std::fmt::Display for ArtifactKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArtifactKind::Seed => write!(f, "seed"),
            ArtifactKind::Proposals => write!(f, "proposals"),
            ArtifactKind::Rules => write!(f, "rules"),
            ArtifactKind::Skeleton => write!(f, "skeleton"),
            ArtifactKind::DecisionLog => write!(f, "decision_log"),
            ArtifactKind::Learnings => write!(f, "learnings"),
            ArtifactKind::Friction => write!(f, "friction"),
            ArtifactKind::BranchHypothesis(name) => write!(f, "branch.{}.hypothesis", name),
            ArtifactKind::BranchReview(name) => write!(f, "branch.{}.review", name),
            ArtifactKind::Postmortem => write!(f, "postmortem"),
            ArtifactKind::Custom(name) => write!(f, "custom.{}", name),
        }
    }
}

/// The kind of pass execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PassKind {
    /// Pure sync, no AI needed
    Sync,
    /// Requires AI completion
    AiAssisted,
    /// Requires human interaction (prompts, confirmations)
    Interactive,
}

/// Context provided to each pass during execution
pub struct PassContext<'a> {
    /// Project root directory
    pub project_root: &'a Path,
    /// Read an artifact file as a string
    pub artifacts: HashMap<ArtifactKind, String>,
}

impl<'a> PassContext<'a> {
    pub fn new(project_root: &'a Path) -> Self {
        Self {
            project_root,
            artifacts: HashMap::new(),
        }
    }

    /// Load an artifact from .process/ directory
    pub fn load_artifact(&mut self, kind: &ArtifactKind) -> Result<String> {
        let filename = self.artifact_filename(kind);
        let path = self.project_root.join(".process").join(&filename);
        let content = std::fs::read_to_string(&path)
            .map_err(|_| anyhow::anyhow!("Artifact '{}' not found at {}", kind, path.display()))?;
        self.artifacts.insert(kind.clone(), content.clone());
        Ok(content)
    }

    /// Save an artifact to .process/ directory
    pub fn save_artifact(&mut self, kind: &ArtifactKind, content: &str) -> Result<()> {
        let filename = self.artifact_filename(kind);
        let path = self.project_root.join(".process").join(&filename);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, content)?;
        self.artifacts.insert(kind.clone(), content.to_string());
        Ok(())
    }

    /// Map ArtifactKind to filename
    fn artifact_filename(&self, kind: &ArtifactKind) -> String {
        match kind {
            ArtifactKind::Seed => "seed.yaml".to_string(),
            ArtifactKind::Proposals => "proposals.yaml".to_string(),
            ArtifactKind::Rules => "rules.yaml".to_string(),
            ArtifactKind::Skeleton => "skeleton.yaml".to_string(),
            ArtifactKind::DecisionLog => "decision_log.yaml".to_string(),
            ArtifactKind::Learnings => "learnings.yaml".to_string(),
            ArtifactKind::Friction => "friction.yaml".to_string(),
            ArtifactKind::BranchHypothesis(name) => format!("branches/{}/hypothesis.yaml", name),
            ArtifactKind::BranchReview(name) => format!("branches/{}/review.yaml", name),
            ArtifactKind::Postmortem => "postmortem.yaml".to_string(),
            ArtifactKind::Custom(name) => format!("{}.yaml", name),
        }
    }
}

/// The core Pass trait — all process steps implement this
pub trait Pass: Send + Sync {
    /// Unique name (e.g., "diverge.generate")
    fn name(&self) -> &'static str;

    /// What this pass needs to run
    fn requires(&self) -> Vec<ArtifactKind>;

    /// What this pass produces
    fn produces(&self) -> Vec<ArtifactKind>;

    /// Execution kind
    fn kind(&self) -> PassKind;

    /// Human-readable description
    fn description(&self) -> &'static str;

    /// Execute the pass
    fn run(&self, ctx: &mut PassContext) -> Result<()>;
}
