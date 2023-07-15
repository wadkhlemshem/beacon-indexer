use serde::{Deserialize, Serialize};

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
