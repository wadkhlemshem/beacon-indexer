use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::util::deserialize_num;

pub enum ValidatorId {
    Pubkey(String),
    Index(u64),
}

impl Display for ValidatorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidatorId::Pubkey(pubkey) => write!(f, "{pubkey}"),
            ValidatorId::Index(index) => write!(f, "{index}"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Validator {
    pub pubkey: String,
    pub slashed: bool,
    #[serde(deserialize_with = "deserialize_num")]
    pub activation_eligibility_epoch: u64,
    #[serde(deserialize_with = "deserialize_num")]
    pub activation_epoch: u64,
    pub effective_balance: String,
    #[serde(deserialize_with = "deserialize_num")]
    pub exit_epoch: u64,
    #[serde(deserialize_with = "deserialize_num")]
    pub withdrawable_epoch: u64,
    pub withdrawal_credentials: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidatorStatus {
    Active,
    Pending,
    Exited,
    PendingInitialized,
    PendingQueued,
    ActiveOngoing,
    ActiveExiting,
    ActiveSlashed,
    ExitedUnslashed,
    ExitedSlashed,
    WithdrawalPossible,
    WithdrawalDone,
}

impl FromStr for ValidatorStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(Self::Active),
            "pending" => Ok(Self::Pending),
            "exited" => Ok(Self::Exited),
            "pending_initialized" => Ok(Self::PendingInitialized),
            "pending_queued" => Ok(Self::PendingQueued),
            "active_ongoing" => Ok(Self::ActiveOngoing),
            "active_exiting" => Ok(Self::ActiveExiting),
            "active_slashed" => Ok(Self::ActiveSlashed),
            "exited_unslashed" => Ok(Self::ExitedUnslashed),
            "exited_slashed" => Ok(Self::ExitedSlashed),
            "withdrawal_possible" => Ok(Self::WithdrawalPossible),
            "withdrawal_done" => Ok(Self::WithdrawalDone),
            _ => Err(anyhow::anyhow!("Invalid status: {}", s)),
        }
    }
}

impl Display for ValidatorStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidatorStatus::Active => write!(f, "active"),
            ValidatorStatus::Pending => write!(f, "pending"),
            ValidatorStatus::Exited => write!(f, "exited"),
            ValidatorStatus::PendingInitialized => write!(f, "pending_initialized"),
            ValidatorStatus::PendingQueued => write!(f, "pending_queued"),
            ValidatorStatus::ActiveOngoing => write!(f, "active_ongoing"),
            ValidatorStatus::ActiveExiting => write!(f, "active_exiting"),
            ValidatorStatus::ActiveSlashed => write!(f, "active_slashed"),
            ValidatorStatus::ExitedUnslashed => write!(f, "exited_unslashed"),
            ValidatorStatus::ExitedSlashed => write!(f, "exited_slashed"),
            ValidatorStatus::WithdrawalPossible => write!(f, "withdrawal_possible"),
            ValidatorStatus::WithdrawalDone => write!(f, "withdrawal_done"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ValidatorData {
    #[serde(deserialize_with = "deserialize_num")]
    pub index: u64,
    pub balance: String,
    pub status: ValidatorStatus,
    pub validator: Validator,
}
