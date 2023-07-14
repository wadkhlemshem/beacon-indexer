use anyhow::Result;
use async_trait::async_trait;
use deadpool_postgres::Pool;

use crate::get_client;

#[async_trait]
pub trait AttestationRepository {
    async fn create_attestation(
        &self,
        epoch: u64,
        validator: u64,
        slot: u8,
        committee_index: u64,
        attested: bool,
    ) -> Result<()>;
    async fn update_attestation(&self, epoch: u64, validator: u64, attested: bool) -> Result<()>;
}

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
    async fn create_attestation(
        &self,
        epoch: u64,
        validator: u64,
        slot: u8,
        committee_index: u64,
        attested: bool,
    ) -> Result<()> {
        let client = self.pool.get().await?;
        client
            .execute(
                "INSERT INTO attestations (epoch, validator, slot, committee_index, attested) VALUES ($1, $2, $3, $4, $5)",
                &[&i64::try_from(epoch)?, &i64::try_from(validator)?, &(slot as i16), &i64::try_from(committee_index)?, &attested],
            )
            .await?;
        Ok(())
    }

    async fn update_attestation(&self, epoch: u64, validator: u64, attested: bool) -> Result<()> {
        let client = get_client(&self.pool).await?;
        client
            .execute(
                "UPDATE attestations SET attested = $1 WHERE epoch = $2 AND validator = $3",
                &[
                    &attested,
                    &i64::try_from(epoch)?,
                    &i64::try_from(validator)?,
                ],
            )
            .await?;
        Ok(())
    }
}
