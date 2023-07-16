use std::sync::Arc;

use anyhow::Result;
use client::{model::attestation::Attestation, HttpClient};
use envconfig::Envconfig;
use futures_util::StreamExt;
use indexer::Indexer;
use service::ServiceImpl;
use store::{
    attestation::PostgresAttestationRepository, committee::PostgresCommitteeRepository, epoch::PostgresEpochRepository,
    validator::PostgresValidatorRepository, DbConfig,
};
use url::Url;

#[derive(Debug, Envconfig)]
struct IndexerConfig {
    #[envconfig(from = "HTTP_RPC_URL")]
    http_rpc_url: Url,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv()?;
    env_logger::init();

    let indexer_config = IndexerConfig::init_from_env()?;
    let client = HttpClient::new(indexer_config.http_rpc_url.clone());

    let db_config = DbConfig::init_from_env()?;
    let db_pool = store::connect(db_config).await;

    let client = Arc::new(client);
    let epoch_repository = Arc::new(PostgresEpochRepository::new(db_pool.clone()));
    let validator_repository = Arc::new(PostgresValidatorRepository::new(db_pool.clone()));
    let attestation_repository = Arc::new(PostgresAttestationRepository::new(db_pool.clone()));
    let committee_repository = Arc::new(PostgresCommitteeRepository::new(db_pool.clone()));
    let service = Arc::new(ServiceImpl::new(
        epoch_repository,
        validator_repository,
        attestation_repository,
        committee_repository,
    ));
    let indexer = Indexer::new(client.clone(), service.clone());

    indexer.index_current_validators().await?;
    indexer.create_epoch(215410).await?;
    indexer.index_committees_for_epoch(215410).await?;

    let stream = client.subscribe::<Attestation>().await?.boxed();
    let handle = tokio::spawn(indexer::pubsub::index_attestations(
        client.clone(),
        service.clone(),
        stream,
    ));
    handle.await??;
    Ok(())
}
