use serde::{Deserialize, Serialize};

use crate::util::deserialize_num;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Proposer {
    pub pubkey: String,
    #[serde(deserialize_with = "deserialize_num")]
    pub validator_index: u64,
    #[serde(deserialize_with = "deserialize_num")]
    pub slot: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProposerResponse {
    pub data: Vec<Proposer>,
    pub execution_optimistic: bool,
    pub dependent_root: String,
}
