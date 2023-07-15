use common::committee::Committee;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CommitteeResponse {
    pub data: Vec<Committee>,
    pub execution_optimistic: bool,
    pub finalized: bool,
}
