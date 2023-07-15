use anyhow::Result;
use async_trait::async_trait;
use deadpool_postgres::Pool;
use rust_decimal::Decimal;
use service::{model::AttestationData, AttestationRepository};

use crate::get_client;

pub struct PostgresAttestationRepository {
    pool: Pool,
}

impl PostgresAttestationRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AttestationRepository for PostgresAttestationRepository {
    async fn get_attestation(&self, epoch: u64, validator: u64) -> Result<Option<bool>> {
        let client = get_client(&self.pool).await?;
        let row = client
            .query_opt(
                "SELECT attested FROM attestation
                WHERE epoch_index = $1 AND validator_index = $2",
                &[&Decimal::from(epoch), &i64::try_from(validator)?],
            )
            .await?;
        Ok(row.map(|row| row.get("attested")))
    }

    async fn get_attestations(&self, epoch_validators: &[(u64, u64)]) -> Result<Vec<Option<bool>>> {
        let client = get_client(&self.pool).await?;
        let epoch_indices = epoch_validators
            .iter()
            .map(|(epoch, _)| Decimal::from(*epoch))
            .collect::<Vec<_>>();
        let validator_indices = epoch_validators
            .iter()
            .map(|(_, validator)| i64::try_from(*validator))
            .collect::<Result<Vec<_>, _>>()?;
        let rows = client
            .query(
                "SELECT epochs.epoch_index, epochs.validator_index, attestation.attested
                FROM UNNEST($1::NUMERIC(20,0)[], $2::BIGINT[]) as epochs(epoch_index, validator_index)
                LEFT JOIN attestation
                ON attestation.epoch_index = epochs.epoch_index AND attestation.validator_index = epochs.validator_index
                ",
                &[&epoch_indices, &validator_indices],
            )
            .await?;
        let attestations = rows
            .into_iter()
            .map(|row| {
                let attested: Option<bool> = row.get("attested");
                attested
            })
            .collect();
        Ok(attestations)
    }

    async fn create_attestation(&self, data: AttestationData) -> Result<()> {
        let client = self.pool.get().await?;
        client
            .execute(
                "INSERT INTO attestation (epoch_index, validator_index, slot, committee_index, attestation)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (epoch_index, validator_index) DO NOTHING",
                &[
                    &Decimal::from(data.epoch),
                    &i64::try_from(data.validator)?,
                    &i64::try_from(data.slot)?,
                    &(data.committee_index as i16),
                    &data.attested,
                ],
            )
            .await?;
        Ok(())
    }

    async fn create_attestation_batch(&self, batch: &[AttestationData]) -> Result<()> {
        let client = self.pool.get().await?;
        let epoch_indices = batch.iter().map(|data| Decimal::from(data.epoch)).collect::<Vec<_>>();
        let validator_indices = batch
            .iter()
            .map(|data| i64::try_from(data.validator))
            .collect::<Result<Vec<_>, _>>()?;
        let slots = batch
            .iter()
            .map(|data| i64::try_from(data.slot))
            .collect::<Result<Vec<_>, _>>()?;
        let committee_indices = batch.iter().map(|data| data.committee_index as i16).collect::<Vec<_>>();
        let attested = batch.iter().map(|data| data.attested).collect::<Vec<_>>();
        client
            .execute(
                "INSERT INTO attestation (epoch_index, validator_index, slot, committee_index, attested)
                SELECT * FROM UNNEST($1::NUMERIC(20,0)[], $2::BIGINT[], $3::BIGINT[], $4::SMALLINT[], $5::BOOLEAN[])
                ON CONFLICT (epoch_index, validator_index) DO NOTHING",
                &[
                    &epoch_indices,
                    &validator_indices,
                    &slots,
                    &committee_indices,
                    &attested,
                ],
            )
            .await?;
        Ok(())
    }
}
