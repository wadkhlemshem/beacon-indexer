use serde::{Deserialize, Serialize};

use crate::util::deserialize_num;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Checkpoint {
    #[serde(deserialize_with = "deserialize_num")]
    pub epoch: u64,
    pub root: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FinalityCheckpoints {
    pub previous_justified: Checkpoint,
    pub current_justified: Checkpoint,
    pub finalized: Checkpoint,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FinalityCheckpointResponse {
    pub data: FinalityCheckpoints,
    pub execution_optimistic: bool,
    pub finalized: bool,
}
