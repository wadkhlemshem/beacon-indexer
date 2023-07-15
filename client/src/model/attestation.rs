use common::attestation::Attestation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AttestationResponse {
    pub data: Vec<Attestation>,
    pub execution_optimistic: bool,
    pub finalized: bool,
}
