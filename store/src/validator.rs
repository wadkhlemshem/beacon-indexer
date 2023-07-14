use anyhow::Result;
use async_trait::async_trait;
use deadpool_postgres::Pool;
use service::model::Validator;
use tokio_postgres::Row;

use crate::get_client;

pub struct PostgresValidator {
    pub index: i64,
    pub pubkey: String,
    pub active: bool,
    pub attestations: i64,
    pub epochs: i64,
}

impl TryFrom<Row> for PostgresValidator {
    type Error = anyhow::Error;

    fn try_from(value: Row) -> Result<Self, Self::Error> {
        Ok(PostgresValidator {
            index: value.try_get("index")?,
            pubkey: value.try_get("pubkey")?,
            active: value.try_get("active")?,
            attestations: value.try_get("attestations")?,
            epochs: value.try_get("epochs")?,
        })
    }
}

impl TryFrom<PostgresValidator> for Validator {
    type Error = anyhow::Error;

    fn try_from(value: PostgresValidator) -> Result<Self, Self::Error> {
        Ok(Validator {
            index: u64::try_from(value.index)?,
            pubkey: value.pubkey,
            active: value.active,
            attestations: u64::try_from(value.attestations)?,
            epochs: u64::try_from(value.epochs)?,
        })
    }
}

#[async_trait]
pub trait ValidatorRepository: Sync + Send {
    async fn get_validator(&self, pubkey: &str) -> Result<Option<Validator>>;
    async fn create_validator(&self, pubkey: &str, active: bool, epochs: u64) -> Result<()>;
    async fn update_active(&self, pubkey: &str, active: bool) -> Result<()>;
    async fn update_epochs(&self, pubkey: &str, epochs: u64) -> Result<()>;
}

pub struct PostgresValidatorRepository {
    pool: Pool,
}

impl PostgresValidatorRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ValidatorRepository for PostgresValidatorRepository {
    async fn create_validator(&self, pubkey: &str, active: bool, epochs: u64) -> Result<()> {
        let client = get_client(&self.pool).await?;
        client
            .execute(
                "INSERT INTO validators (pubkey, active, epochs) VALUES ($1, $2, $3)",
                &[&pubkey, &active, &(i64::try_from(epochs)?)],
            )
            .await?;
        Ok(())
    }

    async fn get_validator(&self, pubkey: &str) -> Result<Option<Validator>> {
        let client = get_client(&self.pool).await?;
        let row = client
            .query_opt(
                "SELECT pubkey, active, epochs
                FROM validator
                LEFT JOIN (
                    SELECT validator, COALESCE(COUNT(attested), 0) AS attestations
                    FROM attestation
                    GROUP BY validator
                    WHERE attested = true
                ) AS attestation
                ON validator.pubkey = attestation.validator
                WHERE validator.pubkey = $1",
                &[&pubkey],
            )
            .await?;
        row.map(PostgresValidator::try_from)
            .transpose()?
            .map(Validator::try_from)
            .transpose()
    }

    async fn update_active(&self, pubkey: &str, active: bool) -> Result<()> {
        let client = get_client(&self.pool).await?;
        client
            .execute(
                "UPDATE validators SET active = $1 WHERE pubkey = $2",
                &[&active, &pubkey],
            )
            .await?;
        Ok(())
    }

    async fn update_epochs(&self, pubkey: &str, epochs: u64) -> Result<()> {
        let client = get_client(&self.pool).await?;
        client
            .execute(
                "UPDATE validators SET epochs = $1 WHERE pubkey = $2",
                &[&(i64::try_from(epochs)?), &pubkey],
            )
            .await?;
        Ok(())
    }
}
