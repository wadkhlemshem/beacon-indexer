use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub enum StateId {
    Head,
    Genesis,
    Finalized,
    Justified,
    Slot(u64),
    StateRoot(String),
}

impl Display for StateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateId::Head => write!(f, "head"),
            StateId::Genesis => write!(f, "genesis"),
            StateId::Finalized => write!(f, "finalized"),
            StateId::Justified => write!(f, "justified"),
            StateId::Slot(slot) => write!(f, "{slot}"),
            StateId::StateRoot(root) => write!(f, "{root}"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct StateRootResponse {
    pub execution_optimistic: bool,
    pub finalized: bool,
    pub data: StateRootData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct StateRootData {
    pub root: String,
}
