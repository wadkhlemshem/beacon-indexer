use anyhow::{anyhow, Result};
use async_trait::async_trait;
use deadpool_postgres::Pool;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use service::{model::Committee, CommitteeRepository};
use tokio_postgres::{types::ToSql, Row};

pub struct PostgresCommittee {
    pub index: i16,
    pub slot: u64,
    pub validators: Vec<u64>,
}

impl TryFrom<PostgresCommittee> for Committee {
    type Error = anyhow::Error;
    fn try_from(committee: PostgresCommittee) -> Result<Self, Self::Error> {
        Ok(Self {
            index: u8::try_from(committee.index)?,
            slot: committee.slot,
            validators: committee.validators,
        })
    }
}

impl From<Committee> for PostgresCommittee {
    fn from(committee: Committee) -> Self {
        Self {
            index: i16::from(committee.index),
            slot: committee.slot,
            validators: committee.validators,
        }
    }
}

impl TryFrom<Row> for PostgresCommittee {
    type Error = anyhow::Error;

    fn try_from(row: Row) -> Result<Self, Self::Error> {
        Ok(Self {
            index: row.try_get("index")?,
            slot: row.get::<_, Decimal>("slot").to_u64().ok_or(anyhow!("Invalid slot"))?,
            validators: row
                .get::<_, Vec<Decimal>>("validators")
                .into_iter()
                .map(|v| v.to_u64().ok_or(anyhow!("Invalid validator")))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

pub struct PostgresCommitteeRepository {
    pool: Pool,
}

impl PostgresCommitteeRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommitteeRepository for PostgresCommitteeRepository {
    async fn get_committee(&self, slot: u64, index: u8) -> Result<Option<Committee>> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT index, slot, validators FROM committee
                WHERE slot = $1 AND index = $2",
                &[&Decimal::from(slot), &i16::from(index)],
            )
            .await?;
        row.map(PostgresCommittee::try_from)
            .transpose()?
            .map(Committee::try_from)
            .transpose()
    }

    async fn get_committees(&self, inputs: &[(u64, u8)]) -> Result<Vec<Committee>> {
        let client = self.pool.get().await?;
        let slots = inputs.iter().map(|(slot, _)| Decimal::from(*slot)).collect::<Vec<_>>();
        let indices = inputs.iter().map(|(_, index)| i16::from(*index)).collect::<Vec<_>>();
        let rows = client
            .query(
                "SELECT index, slot, validators FROM UNNEST($1::NUMERIC(20,0)[], $2::SMALLINT[]) as committees(slot, index)
                LEFT JOIN committee
                ON committee.slot = committees.slot AND committee.index = committees.index",
                &[&slots, &indices],
            )
            .await?;
        rows.into_iter()
            .map(PostgresCommittee::try_from)
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(Committee::try_from)
            .collect()
    }

    async fn get_committees_for_slot(&self, slot: u64) -> Result<Vec<Committee>> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT index, slot, validators FROM committee
                WHERE slot = $1",
                &[&Decimal::from(slot)],
            )
            .await?;
        rows.into_iter()
            .map(PostgresCommittee::try_from)
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(Committee::try_from)
            .collect()
    }

    async fn create_committee(&self, committee: &Committee) -> Result<()> {
        let client = self.pool.get().await?;
        let validators = committee
            .validators
            .iter()
            .map(|v| Decimal::from(*v))
            .collect::<Vec<_>>();
        client
            .execute(
                "INSERT INTO committee (index, slot, validators)
                VALUES ($1, $2, $3::NUMERIC(20,0)[])
                ON CONFLICT (slot, index) DO UPDATE SET validators = $3::NUMERIC(20,0)[]",
                &[&i16::from(committee.index), &Decimal::from(committee.slot), &validators],
            )
            .await?;
        log::info!(
            "Created committee with slot = {}, index = {}",
            committee.slot,
            committee.index
        );
        Ok(())
    }

    async fn create_committee_batch(&self, committees: &[Committee]) -> Result<()> {
        if committees.is_empty() {
            return Ok(());
        }
        let client = self.pool.get().await?;
        let mut batch = Vec::new();
        for committee in committees {
            let validators = committee
                .validators
                .iter()
                .map(|v| Decimal::from(*v))
                .collect::<Vec<_>>();
            batch.push((i16::from(committee.index), Decimal::from(committee.slot), validators));
        }
        let mut query = String::from("INSERT INTO committee (index, slot, validators) VALUES ");
        let mut params: Vec<&(dyn ToSql + Sync)> = Vec::new();
        for (i, r) in batch.iter().enumerate() {
            query.push_str(&format!("(${}, ${}, ${})", i * 3 + 1, i * 3 + 2, i * 3 + 3));
            if i != batch.len() - 1 {
                query.push_str(", ");
            }
            params.push(&r.0);
            params.push(&r.1);
            params.push(&r.2);
        }
        query.push_str(" ON CONFLICT (slot, index) DO UPDATE SET validators = EXCLUDED.validators");
        client.execute(query.as_str(), &params).await?;
        Ok(())
    }
}
