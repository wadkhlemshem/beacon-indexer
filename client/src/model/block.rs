use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::util::deserialize_num;

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

pub enum BlockId {
    Head,
    Genesis,
    Finalized,
    Justified,
    Slot(u64),
    BlockRoot(String),
}

impl Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockId::Head => write!(f, "head"),
            BlockId::Genesis => write!(f, "genesis"),
            BlockId::Finalized => write!(f, "finalized"),
            BlockId::Justified => write!(f, "justified"),
            BlockId::Slot(slot) => write!(f, "{slot}"),
            BlockId::BlockRoot(root) => write!(f, "{root}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_id_display() {
        assert_eq!(format!("{}", BlockId::Head), "head");
        assert_eq!(format!("{}", BlockId::Genesis), "genesis");
        assert_eq!(format!("{}", BlockId::Finalized), "finalized");
        assert_eq!(format!("{}", BlockId::Justified), "justified");
        assert_eq!(format!("{}", BlockId::Slot(123)), "123");
        assert_eq!(
            format!("{}", BlockId::BlockRoot("0x1234567890abcdef".to_string())),
            "0x1234567890abcdef"
        );
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BlockHeaderData {
    pub root: String,
    pub canonical: bool,
    pub header: BlockHeader,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BlockHeader {
    pub message: BlockHeaderMessage,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BlockHeaderMessage {
    #[serde(deserialize_with = "deserialize_num")]
    pub slot: u64,
    #[serde(deserialize_with = "deserialize_num")]
    pub proposer_index: u64,
    pub parent_root: String,
    pub state_root: String,
    pub body_root: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BlockHeaderResponse {
    pub data: BlockHeaderData,
    pub execution_optimistic: bool,
    pub finalized: bool,
}
