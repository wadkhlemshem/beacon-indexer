use anyhow::{anyhow, Result};
use async_trait::async_trait;
use deadpool_postgres::Pool;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use service::ProposerRepository;

pub struct PostgresProposerRepository {
    pool: Pool,
}

impl PostgresProposerRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProposerRepository for PostgresProposerRepository {
    async fn create_proposer(&self, slot: u64, validator: u64) -> Result<()> {
        let client = self.pool.get().await?;
        client
            .execute(
                "INSERT INTO proposer (slot, validator_index)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING",
                &[&Decimal::from(slot), &Decimal::from(validator)],
            )
            .await?;
        Ok(())
    }

    async fn get_proposer_for_slot(&self, slot: u64) -> Result<Option<u64>> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT validator_index FROM proposer
                WHERE slot = $1",
                &[&Decimal::from(slot)],
            )
            .await?;
        row.map(|row| {
            row.get::<_, Decimal>("validator_index")
                .to_u64()
                .ok_or(anyhow!("Invalid validator index"))
        })
        .transpose()
    }

    async fn get_proposers_for_epoch(&self, epoch: u64) -> Result<Vec<u64>> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT validator_index FROM proposer
                WHERE slot >= $1 AND slot < $2",
                &[&Decimal::from(epoch * 32), &Decimal::from((epoch + 1) * 32)],
            )
            .await?;
        let proposers = rows
            .into_iter()
            .map(|row| {
                row.get::<_, Decimal>("validator_index")
                    .to_u64()
                    .ok_or(anyhow!("Invalid validator index"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(proposers)
    }
}
