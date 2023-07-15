use common::validator::ValidatorData;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ValidatorResponse {
    pub execution_optimistic: bool,
    pub finalized: bool,
    pub data: Vec<ValidatorData>,
}
