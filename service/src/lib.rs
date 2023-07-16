pub mod model;

use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use model::{AttestationData, Committee, Epoch, Validator, ValidatorDataInput};

#[async_trait]
pub trait EpochRepository: Sync + Send {
    async fn get_epoch(&self, index: u64) -> Result<Option<Epoch>>;
    async fn create_epoch(&self, epoch_index: u64, active_validators: u64, total_validators: u64) -> Result<()>;
    async fn current_epoch(&self) -> Result<u64>;
}

#[async_trait]
pub trait ValidatorRepository: Sync + Send {
    async fn get_validator(&self, index: u64) -> Result<Option<Validator>>;
    async fn get_active_validators(&self, epoch: u64) -> Result<Vec<Validator>>;
    async fn active_validator_count(&self, epoch: u64) -> Result<u64>;
    async fn total_validator_count(&self, epoch: u64) -> Result<u64>;
    async fn create_or_update_validator(&self, validator: &ValidatorDataInput) -> Result<()>;
    async fn create_or_update_validator_batch(&self, validators: &[ValidatorDataInput]) -> Result<()>;
}

#[async_trait]
pub trait AttestationRepository: Sync + Send {
    async fn create_attestation(&self, attestation_data: AttestationData) -> Result<()>;
    async fn create_attestation_batch(&self, attestations: &[AttestationData]) -> Result<()>;
    async fn get_attestation(&self, epoch: u64, validator: u64) -> Result<Option<bool>>;
    async fn get_attestations(&self, epoch_validators: &[(u64, u64)]) -> Result<Vec<Option<bool>>>;
}

#[async_trait]
pub trait CommitteeRepository: Sync + Send {
    async fn create_committee(&self, committee: &Committee) -> Result<()>;
    async fn create_committee_batch(&self, committees: &[Committee]) -> Result<()>;
    async fn get_committee(&self, slot: u64, index: u8) -> Result<Option<Committee>>;
    async fn get_committees(&self, inputs: &[(u64, u8)]) -> Result<Vec<Committee>>;
}

#[async_trait]
pub trait Service: Sync + Send {
    async fn get_participation_rate_for_epoch(&self, epoch: u64) -> Result<f64>;
    async fn get_participation_rate_for_validator(&self, validator: u64) -> Result<f64>;

    async fn get_validator(&self, index: u64) -> Result<Option<Validator>>;
    async fn get_active_validators(&self, epoch: u64) -> Result<Vec<Validator>>;
    async fn active_validator_count(&self, epoch: u64) -> Result<u64>;
    async fn total_validator_count(&self, epoch: u64) -> Result<u64>;
    async fn create_or_update_validator(&self, validator: &ValidatorDataInput) -> Result<()>;
    async fn create_or_update_validator_batch(&self, validators: &[ValidatorDataInput]) -> Result<()>;

    async fn get_epoch(&self, index: u64) -> Result<Option<Epoch>>;
    async fn create_epoch(&self, epoch_index: u64, active_validators: u64, total_validators: u64) -> Result<()>;

    async fn create_or_update_attestation(&self, attestation_data: AttestationData) -> Result<()>;
    async fn create_or_update_attestation_batch(&self, attestation_data: &[AttestationData]) -> Result<()>;

    async fn create_or_update_committee(&self, committee: &Committee) -> Result<()>;
    async fn create_or_update_committee_batch(&self, committees: &[Committee]) -> Result<()>;
    async fn get_committee(&self, slot: u64, index: u8) -> Result<Option<Committee>>;
    async fn get_committees(&self, inputs: &[(u64, u8)]) -> Result<Vec<Committee>>;
}

#[derive(Clone)]
pub struct ServiceImpl {
    epoch_repository: Arc<dyn EpochRepository>,
    validator_repository: Arc<dyn ValidatorRepository>,
    attestation_repository: Arc<dyn AttestationRepository>,
    committee_repository: Arc<dyn CommitteeRepository>,
}

impl ServiceImpl {
    pub fn new(
        epoch_repository: Arc<dyn EpochRepository>,
        validator_repository: Arc<dyn ValidatorRepository>,
        attestation_repository: Arc<dyn AttestationRepository>,
        committee_repository: Arc<dyn CommitteeRepository>,
    ) -> Self {
        Self {
            epoch_repository,
            validator_repository,
            attestation_repository,
            committee_repository,
        }
    }
}

#[async_trait]
impl Service for ServiceImpl {
    async fn get_participation_rate_for_epoch(&self, epoch: u64) -> Result<f64> {
        let active_validator_count = self.validator_repository.active_validator_count(epoch).await?;
        let attestation_count = self
            .epoch_repository
            .get_epoch(epoch)
            .await?
            .map(|e| e.attestations)
            .ok_or(anyhow!("Epoch not found"))?;
        Ok(attestation_count as f64 / active_validator_count as f64)
    }

    async fn get_participation_rate_for_validator(&self, validator: u64) -> Result<f64> {
        let current_epoch = self.epoch_repository.current_epoch().await?;
        let validator = self
            .validator_repository
            .get_validator(validator)
            .await?
            .ok_or(anyhow!("Validator not found"))?;
        let active_epoch_count = if current_epoch > validator.exit_epoch {
            validator.exit_epoch - validator.activation_epoch
        } else {
            current_epoch - validator.activation_epoch
        };
        let attestation_count = validator.attestations;
        Ok(attestation_count as f64 / active_epoch_count as f64)
    }

    async fn get_validator(&self, index: u64) -> Result<Option<Validator>> {
        self.validator_repository.get_validator(index).await
    }

    async fn get_active_validators(&self, epoch: u64) -> Result<Vec<Validator>> {
        self.validator_repository.get_active_validators(epoch).await
    }

    async fn active_validator_count(&self, epoch: u64) -> Result<u64> {
        self.validator_repository.active_validator_count(epoch).await
    }

    async fn total_validator_count(&self, epoch: u64) -> Result<u64> {
        self.validator_repository.total_validator_count(epoch).await
    }

    async fn create_or_update_validator(&self, validator: &ValidatorDataInput) -> Result<()> {
        self.validator_repository.create_or_update_validator(validator).await
    }

    async fn create_or_update_validator_batch(&self, validator_histories: &[ValidatorDataInput]) -> Result<()> {
        self.validator_repository
            .create_or_update_validator_batch(validator_histories)
            .await
    }

    async fn get_epoch(&self, index: u64) -> Result<Option<Epoch>> {
        self.epoch_repository.get_epoch(index).await
    }

    async fn create_epoch(&self, epoch_index: u64, active_validators: u64, total_validators: u64) -> Result<()> {
        self.epoch_repository
            .create_epoch(epoch_index, active_validators, total_validators)
            .await
    }

    async fn create_or_update_attestation(&self, attestation_data: AttestationData) -> Result<()> {
        match self
            .attestation_repository
            .get_attestation(attestation_data.epoch, attestation_data.validator)
            .await?
        {
            None | Some(false) => self.attestation_repository.create_attestation(attestation_data).await,
            _ => Ok(()),
        }
    }

    async fn create_or_update_attestation_batch(&self, attestation_data: &[AttestationData]) -> Result<()> {
        let mut batch = Vec::new();
        let epoch_validators = attestation_data
            .iter()
            .map(|data| (data.epoch, data.validator))
            .collect::<Vec<_>>();
        let attestations = self.attestation_repository.get_attestations(&epoch_validators).await?;
        for (attestation, attested) in attestation_data.iter().zip(attestations) {
            if attested.is_none() || attested == Some(false) {
                batch.push(attestation.clone())
            }
        }
        self.attestation_repository.create_attestation_batch(&batch).await
    }

    async fn create_or_update_committee(&self, committee: &Committee) -> Result<()> {
        self.committee_repository.create_committee(committee).await
    }

    async fn create_or_update_committee_batch(&self, committees: &[Committee]) -> Result<()> {
        self.committee_repository.create_committee_batch(committees).await
    }

    async fn get_committee(&self, slot: u64, index: u8) -> Result<Option<Committee>> {
        self.committee_repository.get_committee(slot, index).await
    }

    async fn get_committees(&self, inputs: &[(u64, u8)]) -> Result<Vec<Committee>> {
        self.committee_repository.get_committees(inputs).await
    }
}
