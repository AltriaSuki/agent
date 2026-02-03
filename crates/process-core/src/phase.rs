use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Phase {
    Seed = 0,
    Diverge = 1,
    Converge = 2,
    Skeleton = 3,
    Branching = 4,
    Stabilize = 5,
    Postmortem = 6,
    Done = 7,
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Phase::Seed => write!(f, "0. Seed"),
            Phase::Diverge => write!(f, "1. Diverge"),
            Phase::Converge => write!(f, "2. Converge"),
            Phase::Skeleton => write!(f, "3. Skeleton"),
            Phase::Branching => write!(f, "4. Branching"),
            Phase::Stabilize => write!(f, "5. Stabilize"),
            Phase::Postmortem => write!(f, "6. Postmortem"),
            Phase::Done => write!(f, "7. Done"),
        }
    }
}
