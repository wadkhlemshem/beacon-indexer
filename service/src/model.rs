use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Epoch {
    pub index: u64,
    pub active_validators: u64,
    pub total_validators: u64,
    pub attestations: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Validator {
    pub index: u64,
    pub pubkey: String,
    pub active: bool,
    pub attestations: u64,
    pub epochs: u64,
}
