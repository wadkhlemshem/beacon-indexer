use anyhow::{anyhow, Result};
use async_trait::async_trait;
use deadpool_postgres::Pool;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use service::{
    model::{Validator, ValidatorDataInput},
    ValidatorRepository,
};
use tokio_postgres::Row;

use crate::get_client;

pub struct PostgresValidator {
    pub index: i64,
    pub pubkey: String,
    pub attestations: i64,
    pub activation_epoch: u64,
    pub exit_epoch: u64,
}

impl TryFrom<Row> for PostgresValidator {
    type Error = anyhow::Error;

    fn try_from(value: Row) -> Result<Self, Self::Error> {
        Ok(PostgresValidator {
            index: value.try_get("index")?,
            pubkey: value.try_get("pubkey")?,
            attestations: value.try_get("attestations")?,
            activation_epoch: value
                .get::<_, Decimal>("activation_epoch")
                .to_u64()
                .ok_or(anyhow::anyhow!("Invalid activation epoch"))?,
            exit_epoch: value
                .get::<_, Decimal>("exit_epoch")
                .to_u64()
                .ok_or(anyhow!("Invalid exit epoch"))?,
        })
    }
}

impl TryFrom<PostgresValidator> for Validator {
    type Error = anyhow::Error;

    fn try_from(value: PostgresValidator) -> Result<Self, Self::Error> {
        Ok(Validator {
            index: u64::try_from(value.index)?,
            pubkey: value.pubkey,
            attestations: u64::try_from(value.attestations)?,
            activation_epoch: value.activation_epoch,
            exit_epoch: value.exit_epoch,
        })
    }
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
    async fn create_or_update_validator(&self, validator: &ValidatorDataInput) -> Result<()> {
        let client = get_client(&self.pool).await?;
        client
            .execute(
                "INSERT INTO validator (index, pubkey, activation_epoch, exit_epoch)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (index) DO UPDATE SET pubkey = EXCLUDED.pubkey, activation_epoch = EXCLUDED.activation_epoch, exit_epoch = EXCLUDED.exit_epoch",
                &[&i64::try_from(validator.index)?, &validator.pubkey, &Decimal::from(validator.activation_epoch), &Decimal::from(validator.exit_epoch)],
            )
            .await?;
        Ok(())
    }

    async fn get_active_validators(&self, epoch_index: u64) -> Result<Vec<Validator>> {
        let client = get_client(&self.pool).await?;
        let rows = client
            .query(
                "SELECT validator.index, validator.pubkey, COALESCE(attestation.attestations, 0) as attestations, validator.activation_epoch, validator.exit_epoch
                FROM validator
                LEFT JOIN (
                    SELECT validator_index, COUNT(attested) AS attestations
                    FROM attestation
                    WHERE attested = true
                    GROUP BY validator_index
                ) AS attestation
                ON validator.index = attestation.validator_index
                WHERE validator.activation_epoch <= $1 AND validator.exit_epoch > $1",
                &[&Decimal::from(epoch_index)],
            )
            .await?;
        let validators = rows
            .into_iter()
            .map(PostgresValidator::try_from)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(Validator::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(validators)
    }

    async fn active_validator_count(&self, epoch_index: u64) -> Result<u64> {
        let client = get_client(&self.pool).await?;
        let row = client
            .query_one(
                "SELECT COUNT(*)
                FROM validator
                WHERE validator.activation_epoch <= $1 AND validator.exit_epoch > $1",
                &[&Decimal::from(epoch_index)],
            )
            .await?;
        let count: i64 = row.get(0);
        Ok(u64::try_from(count)?)
    }

    async fn total_validator_count(&self, epoch_index: u64) -> Result<u64> {
        let client = get_client(&self.pool).await?;
        let row = client
            .query_one(
                "SELECT COUNT(*)
                FROM validator
                WHERE validator.activation_epoch <= $1",
                &[&Decimal::from(epoch_index)],
            )
            .await?;
        let count: i64 = row.get(0);
        Ok(u64::try_from(count)?)
    }

    async fn create_or_update_validator_batch(&self, batch: &[ValidatorDataInput]) -> Result<()> {
        let client = get_client(&self.pool).await?;
        let indices = batch
            .iter()
            .map(|data| i64::try_from(data.index))
            .collect::<Result<Vec<_>, _>>()?;
        let pubkeys = batch.iter().map(|data| &data.pubkey).collect::<Vec<_>>();
        let activation_epochs = batch
            .iter()
            .map(|data| Decimal::from(data.activation_epoch))
            .collect::<Vec<_>>();
        let exit_epochs = batch
            .iter()
            .map(|data| Decimal::from(data.exit_epoch))
            .collect::<Vec<_>>();
        client
            .execute(
                "INSERT INTO validator (index, pubkey, activation_epoch, exit_epoch)
                SELECT * FROM UNNEST($1::BIGINT[], $2::VARCHAR[], $3::NUMERIC(20,0)[], $4::NUMERIC(20,0)[])
                ON CONFLICT (index) DO UPDATE SET pubkey = EXCLUDED.pubkey, activation_epoch = EXCLUDED.activation_epoch, exit_epoch = EXCLUDED.exit_epoch",
                &[&indices, &pubkeys, &activation_epochs, &exit_epochs],
            )
            .await?;
        Ok(())
    }

    async fn get_validator(&self, index: u64) -> Result<Option<Validator>> {
        let client = get_client(&self.pool).await?;
        let row = client
            .query_opt(
                "SELECT index, pubkey, COALESCE(validator_history.active_epochs, 0) as active_epochs, COALESCE(attestation.attestations, 0) as attestations
                FROM validator
                LEFT JOIN (
                    SELECT validator_index, COUNT(is_active) AS active_epochs
                    FROM validator_history
                    WHERE is_active = true
                    GROUP BY validator_index
                ) AS validator_history
                ON validator.index = validator_history.validator_index
                LEFT JOIN (
                    SELECT validator_index, COUNT(attested) AS attestations
                    FROM attestation
                    WHERE attested = true
                    GROUP BY validator_index
                ) AS attestation
                ON validator.index = attestation.validator_index
                WHERE validator.index = $1",
                &[&i64::try_from(index)?],
            )
            .await?;
        row.map(PostgresValidator::try_from)
            .transpose()?
            .map(Validator::try_from)
            .transpose()
    }
}
