use anyhow::{anyhow, Result};
use async_trait::async_trait;
use deadpool_postgres::Pool;
use postgres_types::ToSql;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use service::{model::Proposer, ProposerRepository};

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

    async fn create_proposers(&self, proposers: &[Proposer]) -> Result<()> {
        if proposers.is_empty() {
            return Ok(());
        }
        let client = self.pool.get().await?;
        let proposers = proposers
            .iter()
            .map(|proposer| (Decimal::from(proposer.slot), Decimal::from(proposer.validator_index)))
            .collect::<Vec<_>>();
        let mut query = "INSERT INTO proposer (slot, validator_index) VALUES ".to_string();
        let mut params: Vec<&(dyn ToSql + Sync)> = Vec::new();
        for (i, proposer) in proposers.iter().enumerate() {
            query.push_str(&format!("(${}, ${})", i * 2 + 1, i * 2 + 2));
            params.push(&proposer.0);
            params.push(&proposer.1);
            if i < proposers.len() - 1 {
                query.push_str(", ");
            }
        }
        client.execute(&query, &params).await?;
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
