use std::{collections::BTreeMap, sync::Arc};

use anyhow::{anyhow, Result};
use client::EthClient;
use common::{block::BlockId, committee::Committee, state::StateId};
use service::{
    model::{AttestationData, ValidatorDataInput},
    Service,
};

pub struct Indexer {
    pub client: Arc<dyn EthClient>,
    pub service: Arc<dyn Service>,
}

impl Indexer {
    pub fn new(client: Arc<dyn EthClient>, service: Arc<dyn Service>) -> Self {
        Self { client, service }
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
            let service = self.service.clone();
            service.create_or_update_validator_batch(&validator_data).await?;
            added += chunk.len();
            log::info!("Added {added}/{total_validator_count} validators");
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

    pub async fn get_committee_map_for_epoch(&self, epoch: u64) -> Result<BTreeMap<(u64, u8), Committee>> {
        let start_slot = epoch * 32;
        let committees = self
            .client
            .get_committees_for_state(StateId::Slot(start_slot), Some(epoch), None, None)
            .await?;
        let mut committee_map = BTreeMap::new();
        for committee in committees {
            committee_map.insert((committee.slot, committee.index), committee.clone());
        }
        Ok(committee_map)
    }

    pub async fn run_for_epoch(&self, epoch: u64) -> Result<()> {
        log::info!("Processing epoch {epoch}");
        let start_slot = epoch * 32;
        self.create_epoch(epoch).await?;
        if epoch != 0 {
            self.create_epoch(epoch - 1).await?;
        }
        let committee_map = self.get_committee_map_for_epoch(epoch).await?;
        let previous_committee_map = match epoch {
            0 => BTreeMap::new(),
            _ => self.get_committee_map_for_epoch(epoch - 1).await?,
        };
        for slot in start_slot..start_slot + 32 {
            log::info!("Processing slot {slot}");
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
                let committee = if attestation.data.target.epoch < epoch {
                    log::info!("Processing delayed attestation at slot {slot} and index {index}");
                    previous_committee_map
                        .get(&(attestation.data.slot, attestation.data.index))
                        .ok_or(anyhow!("Committee not found"))?
                } else {
                    committee_map
                        .get(&(attestation.data.slot, attestation.data.index))
                        .ok_or(anyhow!("Committee not found"))?
                };

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
