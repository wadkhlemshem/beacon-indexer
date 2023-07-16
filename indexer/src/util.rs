use std::sync::Arc;

use anyhow::{anyhow, Result};
use client::{
    model::{attestation::Attestation, state::StateId},
    JsonRpcClient,
};
use service::{model::AttestationData, Service};

pub async fn get_committee_for_slot_and_index(
    client: Arc<dyn JsonRpcClient>,
    service: Arc<dyn Service>,
    slot: u64,
    index: u8,
) -> Result<Option<service::model::Committee>> {
    let committee = service.get_committee(slot, index).await?;
    match committee {
        Some(committee) => Ok(Some(committee)),
        None => {
            let committees = client
                .get_committees_for_state(StateId::Slot(slot), None, Some(index), Some(slot))
                .await?;
            if committees.is_empty() {
                Ok(None)
            } else {
                let committee = committees[0].clone().into();
                service.create_or_update_committee(&committee).await?;
                Ok(Some(committee))
            }
        }
    }
}

pub async fn process_attestation(
    client: Arc<dyn JsonRpcClient>,
    service: Arc<dyn Service>,
    attestation: Attestation,
) -> Result<()> {
    let epoch = attestation.data.target.epoch;
    let slot = attestation.data.slot;
    let index = attestation.data.index;
    log::info!("Processing attestation at slot {slot} and index {index}");
    let committee = get_committee_for_slot_and_index(client.clone(), service.clone(), slot, index)
        .await?
        .ok_or(anyhow!("Committee not found for slot {slot} and index {index}"))?;
    let aggregation_bits = attestation.aggregation_bits.clone();
    log::debug!("Aggregation bits: {aggregation_bits}");
    let aggregation_bits = aggregation_bits.trim_start_matches("0x");
    let aggregation_bits = hex::decode(aggregation_bits)?;
    let bit_vec = bit_vec::BitVec::from_bytes(&aggregation_bits);

    let mut batch = Vec::new();
    for (i, validator) in committee.validators.iter().enumerate() {
        let attested = bit_vec[i];
        batch.push(AttestationData {
            epoch,
            validator: *validator,
            slot,
            committee_index: index,
            attested,
        });
    }
    log::info!("Adding attestations for slot {slot} and index {index}");
    service.create_or_update_attestation_batch(&batch).await?;
    Ok(())
}
