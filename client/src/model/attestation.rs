use serde::{Deserialize, Serialize};

use crate::{
    subscription::{Subscribable, SubscribeEvent},
    util::deserialize_num,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Attestation {
    pub aggregation_bits: String,
    pub data: AggregationData,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AggregationData {
    pub beacon_block_root: String,
    #[serde(deserialize_with = "deserialize_num")]
    pub index: u8,
    #[serde(deserialize_with = "deserialize_num")]
    pub slot: u64,
    pub source: Checkpoint,
    pub target: Checkpoint,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Checkpoint {
    #[serde(deserialize_with = "deserialize_num")]
    pub epoch: u64,
    pub root: String,
}

impl Subscribable for Attestation {
    fn subscribe_event() -> SubscribeEvent {
        SubscribeEvent::Attestation
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AttestationResponse {
    pub data: Vec<Attestation>,
    pub execution_optimistic: bool,
    pub finalized: bool,
}
