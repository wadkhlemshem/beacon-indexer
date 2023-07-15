use common::block::BlockHeaderData;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BlockRootResponse {
    pub data: BlockRootData,
    pub execution_optimistic: bool,
    pub finalized: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BlockRootData {
    pub root: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BlockHeaderResponse {
    pub data: BlockHeaderData,
    pub execution_optimistic: bool,
    pub finalized: bool,
}
