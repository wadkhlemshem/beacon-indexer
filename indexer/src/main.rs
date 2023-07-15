use std::{collections::BTreeMap, sync::Arc};

use anyhow::{anyhow, Result};
use client::{EthClient, RpcClient};
use common::{block::BlockId, committee::Committee, state::StateId, validator::ValidatorStatus};
use envconfig::Envconfig;
use service::{
    model::{AttestationData, ValidatorDataInput},
    Service, ServiceImpl,
};
use store::{
    attestation::PostgresAttestationRepository, epoch::PostgresEpochRepository, validator::PostgresValidatorRepository,
    DbConfig,
};
use url::Url;

#[derive(Debug, Envconfig)]
struct IndexerConfig {
    #[envconfig(from = "HTTP_RPC_URL")]
    http_rpc_url: Url,
}

async fn add_validator_batch(service: Arc<ServiceImpl>, validator_histories: &[ValidatorDataInput]) -> Result<()> {
    service.create_or_update_validator_batch(validator_histories).await?;

    Ok(())
}

async fn create_or_update_attestation_batch(service: Arc<ServiceImpl>, attestations: &[AttestationData]) -> Result<()> {
    service.create_or_update_attestation_batch(attestations).await?;

    Ok(())
}

async fn index_epoch_with_validators(client: Arc<RpcClient>, service: Arc<ServiceImpl>, epoch: u64) -> Result<()> {
    let start_slot = epoch * 32;
    log::info!("Retrieving validators for epoch {epoch}");
    let validators = client
        .validators_for_state(StateId::Slot(start_slot), &[], None)
        .await?;
    let total_validator_count = validators.len() as u64;
    let active_validator_count = validators
        .iter()
        .filter(|v| {
            v.status == ValidatorStatus::ActiveOngoing
                || v.status == ValidatorStatus::ActiveSlashed
                || v.status == ValidatorStatus::ActiveExiting
        })
        .count() as u64;
    service
        .create_epoch(epoch, active_validator_count, total_validator_count)
        .await?;
    log::debug!("Epoch = {:#?}", service.get_epoch(epoch).await?);
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
        let service = service.clone();
        add_validator_batch(service, &validator_data).await?;
        added += chunk.len();
        log::info!("Added {added}/{total_validator_count} validators");
    }

    log::info!("active validator count = {active_validator_count}");
    log::info!("total validator count = {total_validator_count}");

    Ok(())
}

async fn get_committee_map_for_epoch(client: Arc<RpcClient>, epoch: u64) -> Result<BTreeMap<(u64, u8), Committee>> {
    let start_slot = epoch * 32;
    let committees = client
        .get_committees_for_state(StateId::Slot(start_slot), None, None, None)
        .await?;
    let mut committee_map = BTreeMap::new();
    for committee in committees {
        committee_map.insert((committee.slot, committee.index), committee.clone());
    }
    Ok(committee_map)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv()?;
    env_logger::init();
    let indexer_config = IndexerConfig::init_from_env()?;
    let client = RpcClient::new(indexer_config.http_rpc_url);

    let db_config = DbConfig::init_from_env()?;
    let db_pool = store::connect(db_config).await;

    let client = Arc::new(client);
    let epoch_repository = Arc::new(PostgresEpochRepository::new(db_pool.clone()));
    let validator_repository = Arc::new(PostgresValidatorRepository::new(db_pool.clone()));
    let attestation_repository = Arc::new(PostgresAttestationRepository::new(db_pool));
    let service = Arc::new(ServiceImpl::new(
        epoch_repository,
        validator_repository,
        attestation_repository,
    ));
    index_epoch_with_validators(client.clone(), service.clone(), 214888).await?;
    for epoch in 214889..=214891 {
        log::info!("Processing epoch {epoch}");
        let start_slot = epoch * 32;
        index_epoch_with_validators(client.clone(), service.clone(), epoch).await?;
        let committee_map = get_committee_map_for_epoch(client.clone(), epoch).await?;

        let previous_committee_map = match epoch {
            0 => None,
            _ => Some(get_committee_map_for_epoch(client.clone(), epoch - 1).await?),
        };
        for slot in start_slot..start_slot + 32 {
            log::info!("Processing slot {slot}");
            let attestations = match client.get_attestations_for_block(BlockId::Slot(slot)).await? {
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
                        .as_ref()
                        .ok_or(anyhow!("Previous committee map not found"))?
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
            create_or_update_attestation_batch(service.clone(), &batch).await?;
        }
    }
    let epoch = service.get_epoch(214889).await?.ok_or(anyhow!("Epoch not found"))?;
    let attestations_count = epoch.attestations;
    let active_validator_count = epoch.active_validators;
    println!("attestations count = {}", attestations_count);
    let participation_rate = (attestations_count as f64) / (active_validator_count as f64);
    println!("participation rate = {}", participation_rate);

    Ok(())
}
