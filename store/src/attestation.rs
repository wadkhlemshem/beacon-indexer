use anyhow::Result;
use async_trait::async_trait;
use deadpool_postgres::Pool;
use rust_decimal::Decimal;
use service::{model::AttestationData, AttestationRepository};

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
    async fn get_attestation_for_slot_and_validator(&self, slot: u64, validator: u64) -> Result<Option<bool>> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT attested FROM attestation
                WHERE slot = $1 AND validator_index = $2",
                &[&Decimal::from(slot), &Decimal::from(validator)],
            )
            .await?;
        Ok(row.map(|row| row.get("attested")))
    }

    async fn get_attestation_for_epoch_and_validator(&self, epoch: u64, validator: u64) -> Result<Option<bool>> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT attested FROM attestation
                WHERE epoch_index = $1 AND validator_index = $2",
                &[&Decimal::from(epoch), &Decimal::from(validator)],
            )
            .await?;
        Ok(row.map(|row| row.get("attested")))
    }

    async fn get_attestations(&self, epoch_validators: &[(u64, u64)]) -> Result<Vec<Option<bool>>> {
        let client = self.pool.get().await?;
        let epoch_indices = epoch_validators
            .iter()
            .map(|(epoch, _)| Decimal::from(*epoch))
            .collect::<Vec<_>>();
        let validator_indices = epoch_validators
            .iter()
            .map(|(_, validator)| Decimal::from(*validator))
            .collect::<Vec<_>>();
        let rows = client
            .query(
                "SELECT epochs.epoch_index, epochs.validator_index, attestation.attested
                FROM UNNEST($1::NUMERIC(20,0)[], $2::NUMERIC(20,0)[]) as epochs(epoch_index, validator_index)
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

    async fn attestation_count_for_slot(&self, slot: u64) -> Result<u64> {
        let client = self.pool.get().await?;
        let row = client
            .query_one(
                "SELECT COUNT(*) FROM attestation
                WHERE slot = $1",
                &[&Decimal::from(slot)],
            )
            .await?;
        let count: i64 = row.get(0);
        Ok(u64::try_from(count)?)
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
                    &Decimal::from(data.validator),
                    &Decimal::from(data.slot),
                    &(data.committee_index as i16),
                    &data.attested,
                ],
            )
            .await?;
        Ok(())
    }

    async fn create_attestation_batch(&self, batch: &[AttestationData]) -> Result<()> {
        if batch.is_empty() {
            return Ok(());
        }
        let client = self.pool.get().await?;
        let epoch_indices = batch.iter().map(|data| Decimal::from(data.epoch)).collect::<Vec<_>>();
        let validator_indices = batch
            .iter()
            .map(|data| Decimal::from(data.validator))
            .collect::<Vec<_>>();
        let slots = batch.iter().map(|data| Decimal::from(data.slot)).collect::<Vec<_>>();
        let committee_indices = batch.iter().map(|data| data.committee_index as i16).collect::<Vec<_>>();
        let attested = batch.iter().map(|data| data.attested).collect::<Vec<_>>();
        client
            .execute(
                "INSERT INTO attestation (epoch_index, validator_index, slot, committee_index, attested)
                SELECT * FROM UNNEST($1::NUMERIC(20,0)[], $2::NUMERIC(20,0)[], $3::NUMERIC(20,0)[], $4::SMALLINT[], $5::BOOLEAN[])
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
