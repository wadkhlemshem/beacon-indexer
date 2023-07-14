use anyhow::Result;
use async_trait::async_trait;
use deadpool_postgres::Pool;
use service::model::Epoch;
use tokio_postgres::Row;

// TODO: Use better types
pub struct PostgresEpoch {
    pub index: i64,
    pub active_validators: i64,
    pub total_validators: i64,
    pub attestations: i64,
}

impl TryFrom<Row> for PostgresEpoch {
    type Error = anyhow::Error;

    fn try_from(value: Row) -> std::result::Result<Self, Self::Error> {
        Ok(PostgresEpoch {
            index: value.try_get("index")?,
            active_validators: value.try_get("active_validators")?,
            total_validators: value.try_get("total_validators")?,
            attestations: value.try_get("attestations")?,
        })
    }
}

impl TryFrom<PostgresEpoch> for Epoch {
    type Error = anyhow::Error;

    fn try_from(value: PostgresEpoch) -> Result<Self, Self::Error> {
        Ok(Epoch {
            index: u64::try_from(value.index)?,
            active_validators: u64::try_from(value.active_validators)?,
            total_validators: u64::try_from(value.total_validators)?,
            attestations: u64::try_from(value.attestations)?,
        })
    }
}

#[async_trait]
pub trait EpochRepository: Sync + Send {
    async fn get_epoch(&self, index: u64) -> Result<Option<Epoch>>;
    async fn create_epoch(
        &self,
        epoch_index: u64,
        active_validators: u64,
        total_validators: u64,
    ) -> Result<()>;
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
                "SELECT index, active_validators, total_validators
                FROM epochs
                LEFT JOIN (
                    SELECT epoch, COALESCE(COUNT(attested), 0) AS attestations
                    FROM attestation
                    GROUP BY epoch
                    WHERE attested = true
                ) AS attestation
                ON epoch.index = attestation.epoch
                WHERE epoch.index = $1
                ",
                &[&i64::try_from(index)?],
            )
            .await?;
        row.map(PostgresEpoch::try_from)
            .transpose()?
            .map(Epoch::try_from)
            .transpose()
    }

    async fn create_epoch(
        &self,
        epoch_index: u64,
        active_validators: u64,
        total_validators: u64,
    ) -> Result<()> {
        let client = self.pool.get().await?;
        client
            .execute(
                "INSERT INTO epochs (index, active_validators, total_validators) VALUES ($1, $2, $3, $4)",
                &[&i64::try_from(epoch_index)?, &i64::try_from(active_validators)?, &i64::try_from(total_validators)?],
            )
            .await?;
        Ok(())
    }
}
