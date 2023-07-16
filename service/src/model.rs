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
    pub attestations: u64,
    pub activation_epoch: u64,
    pub exit_epoch: u64,
}

#[derive(Debug)]
pub struct ValidatorDataInput {
    pub index: u64,
    pub pubkey: String,
    pub activation_epoch: u64,
    pub exit_epoch: u64,
}

#[derive(Debug, Clone)]
pub struct AttestationData {
    pub epoch: u64,
    pub validator: u64,
    pub slot: u64,
    pub committee_index: u8,
    pub attested: bool,
}

#[derive(Debug, Clone)]
pub struct Committee {
    pub index: u8,
    pub slot: u64,
    pub validators: Vec<u64>,
}

impl From<common::committee::Committee> for Committee {
    fn from(committee: common::committee::Committee) -> Self {
        Self {
            index: committee.index,
            slot: committee.slot,
            validators: committee.validators,
        }
    }
}
