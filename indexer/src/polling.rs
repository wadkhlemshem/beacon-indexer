use std::sync::Arc;

use anyhow::{anyhow, Result};
use client::{
    model::{block::BlockId, state::StateId},
    JsonRpcClient,
};
use service::{
    model::{AttestationData, ValidatorDataInput},
    Service,
};

use crate::util::get_committee_for_slot_and_index;

pub struct PollingIndexer {
    pub client: Arc<dyn JsonRpcClient>,
    pub service: Arc<dyn Service>,
    pub max_epoch: Option<u64>,
}

impl PollingIndexer {
    pub fn new(client: Arc<dyn JsonRpcClient>, service: Arc<dyn Service>, max_epoch: Option<u64>) -> Self {
        Self {
            client,
            service,
            max_epoch,
        }
    }

    pub async fn run(self) -> Result<()> {
        let finality_checkpoints = self.client.get_finality_checkpoints(StateId::Head).await?;
        self.index_current_validators().await?;
        let current_epoch = finality_checkpoints.current_justified.epoch;
        let max_epoch = match self.max_epoch {
            Some(max) if max > current_epoch => max,
            _ => current_epoch,
        };
        for epoch in 0..=max_epoch {
            self.run_for_epoch(epoch).await?;
        }
        if self.max_epoch.is_none() {
            loop {
                self.run_for_epoch(max_epoch + 1).await?;
            }
        }
        Ok(())
    }

    pub async fn index_current_validators(&self) -> Result<()> {
        let validators = self.client.validators_for_state(StateId::Head, &[], None).await?;
        let total_validator_count = validators.len() as u64;
        log::info!("Adding validators");
        let mut added = 0;
        for chunk in validators.chunks(1000) {
            let mut validator_data = Vec::new();
            for data in chunk {
                let pubkey = data.validator.pubkey.clone();
                let activation_epoch = data.validator.activation_epoch;
                let exit_epoch = data.validator.exit_epoch;
                validator_data.push(ValidatorDataInput {
                    index: data.index,
                    pubkey,
                    activation_epoch,
                    exit_epoch,
                });
            }
            self.service.create_or_update_validator_batch(&validator_data).await?;
            added += chunk.len();
            log::info!("Added {added}/{total_validator_count} validators");
        }

        Ok(())
    }

    pub async fn index_committees_for_epoch(&self, epoch: u64) -> Result<()> {
        let slot = epoch * 32;
        let committees = self
            .client
            .get_committees_for_state(StateId::Slot(slot), Some(epoch), None, None)
            .await?;
        log::info!("Adding committees for epoch {epoch}");
        let total_committee_count = committees.len();
        let mut added = 0;
        for chunk in committees.chunks(1000) {
            let mut committee_data = Vec::new();
            for data in chunk {
                let mut validators = Vec::new();
                for validator in &data.validators {
                    validators.push(*validator);
                }
                committee_data.push(service::model::Committee {
                    slot: data.slot,
                    index: data.index,
                    validators,
                });
            }
            let service = self.service.clone();
            service.create_or_update_committee_batch(&committee_data).await?;
            added += chunk.len();
            log::info!("Added {added}/{total_committee_count} committees");
        }
        Ok(())
    }

    pub async fn create_epoch(&self, epoch: u64) -> Result<()> {
        let active_validator_count = self.service.active_validator_count(epoch).await?;
        let total_validator_count = self.service.total_validator_count(epoch).await?;
        log::info!("Creating epoch {epoch} with {active_validator_count} active validators and {total_validator_count} total validators");
        self.service
            .create_epoch(epoch, active_validator_count, total_validator_count)
            .await?;
        Ok(())
    }

    pub async fn run_for_epoch(&self, epoch: u64) -> Result<()> {
        log::info!("Processing epoch {epoch}");
        let start_slot = epoch * 32;
        self.create_epoch(epoch).await?;
        if epoch != 0 {
            self.create_epoch(epoch - 1).await?;
        }
        self.index_committees_for_epoch(epoch).await?;
        for slot in start_slot..start_slot + 32 {
            log::info!("Processing slot {slot}");
            // Can't use get_proposers_for_epoch because it doesn't return the correct proposer for the genesis slot
            let header = self.client.get_header_for_block(BlockId::Slot(slot)).await?;
            if let Some(header) = header {
                let proposer = header.message.proposer_index;
                self.service.create_proposer(slot, proposer).await?;
            }
            let attestations = match self.client.get_attestations_for_block(BlockId::Slot(slot)).await? {
                Some(attestations) => attestations,
                None => continue,
            };
            log::debug!("attestations.len() = {}", attestations.len());

            let mut batch = Vec::new();
            for attestation in attestations {
                let index = attestation.data.index;
                log::debug!(
                    "Processing attestation with slot = {}, index = {}, target epoch = {}",
                    attestation.data.slot,
                    index,
                    attestation.data.target.epoch
                );
                let committee = get_committee_for_slot_and_index(
                    self.client.clone(),
                    self.service.clone(),
                    attestation.data.slot,
                    attestation.data.index,
                )
                .await?
                .ok_or(anyhow!("Committee not found"))?;

                let aggregation_bits = attestation.aggregation_bits.clone();

                log::debug!("Aggregation bits: {aggregation_bits}");

                let aggregation_bits = aggregation_bits.trim_start_matches("0x");
                let aggregation_bits = hex::decode(aggregation_bits)?;
                let bit_vec = bit_vec::BitVec::from_bytes(&aggregation_bits);

                for (i, validator) in committee.validators.iter().enumerate() {
                    let attested = bit_vec[i];
                    batch.push(AttestationData {
                        epoch: attestation.data.target.epoch,
                        validator: *validator,
                        slot: attestation.data.slot,
                        committee_index: attestation.data.index,
                        attested,
                    });
                }
            }
            log::info!("Adding attestations for slot {slot}");
            self.service.create_or_update_attestation_batch(&batch).await?;
        }

        Ok(())
    }
}
