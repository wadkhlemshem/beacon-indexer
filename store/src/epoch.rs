use anyhow::{anyhow, Result};
use async_trait::async_trait;
use deadpool_postgres::Pool;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use service::{model::Epoch, EpochRepository};
use tokio_postgres::Row;

pub struct PostgresEpoch {
    pub index: u64,
    pub active_validators: u64,
    pub total_validators: u64,
    pub attestations: i64,
}

impl TryFrom<Row> for PostgresEpoch {
    type Error = anyhow::Error;

    fn try_from(value: Row) -> std::result::Result<Self, Self::Error> {
        Ok(PostgresEpoch {
            index: value
                .get::<_, Decimal>("index")
                .to_u64()
                .ok_or(anyhow::anyhow!("Invalid epoch index"))?,
            active_validators: value
                .get::<_, Decimal>("active_validators")
                .to_u64()
                .ok_or(anyhow!("Invalid number of active validators"))?,
            total_validators: value
                .get::<_, Decimal>("total_validators")
                .to_u64()
                .ok_or(anyhow!("Invalid number of total validators"))?,
            attestations: value.try_get("attestations")?,
        })
    }
}

impl TryFrom<PostgresEpoch> for Epoch {
    type Error = anyhow::Error;

    fn try_from(value: PostgresEpoch) -> Result<Self, Self::Error> {
        Ok(Epoch {
            index: value.index,
            active_validators: value.active_validators,
            total_validators: value.total_validators,
            attestations: u64::try_from(value.attestations)?,
        })
    }
}

pub struct PostgresEpochRepository {
    pool: Pool,
}

impl PostgresEpochRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EpochRepository for PostgresEpochRepository {
    async fn get_epoch(&self, index: u64) -> Result<Option<Epoch>> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT index, active_validators, total_validators, COALESCE(attestation.attestations, 0) as attestations
                FROM epoch
                LEFT JOIN (
                    SELECT epoch_index, COUNT(attested) AS attestations
                    FROM attestation
                    WHERE attested = true
                    GROUP BY epoch_index
                ) AS attestation
                ON epoch.index = attestation.epoch_index
                WHERE index = $1
                ",
                &[&Decimal::from(index)],
            )
            .await?;
        row.map(PostgresEpoch::try_from)
            .transpose()?
            .map(Epoch::try_from)
            .transpose()
    }

    async fn create_epoch(&self, epoch_index: u64, active_validators: u64, total_validators: u64) -> Result<()> {
        let client = self.pool.get().await?;
        client
            .execute(
                "INSERT INTO epoch (index, active_validators, total_validators)
                VALUES ($1, $2, $3)
                ON CONFLICT (index) DO NOTHING",
                &[
                    &Decimal::from(epoch_index),
                    &Decimal::from(active_validators),
                    &Decimal::from(total_validators),
                ],
            )
            .await?;
        Ok(())
    }
}
