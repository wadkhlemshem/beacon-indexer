use std::sync::Arc;

use anyhow::Result;
use client::{model::attestation::Attestation, HttpClient};
use envconfig::Envconfig;
use futures_util::StreamExt;
use indexer::polling::PollingIndexer;
use service::ServiceImpl;
use store::{
    attestation::PostgresAttestationRepository, committee::PostgresCommitteeRepository, epoch::PostgresEpochRepository,
    validator::PostgresValidatorRepository, DbConfig,
};
use tokio::task::JoinSet;
use url::Url;

#[derive(Debug, Envconfig)]
struct IndexerConfig {
    #[envconfig(from = "HTTP_RPC_URL")]
    http_rpc_url: Url,
    #[envconfig(from = "MAX_EPOCH")]
    max_epoch: Option<u64>,
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

    let mut handle_set = JoinSet::new();

    let polling_indexer = PollingIndexer::new(client.clone(), service.clone(), indexer_config.max_epoch);

    handle_set.spawn(polling_indexer.run());

    if indexer_config.max_epoch.is_none() {
        let stream = client.subscribe::<Attestation>().await?.boxed();
        handle_set.spawn(indexer::pubsub::index_attestations(
            client.clone(),
            service.clone(),
            stream,
        ));
    }
    while let Some(result) = handle_set.join_next().await {
        result??;
    }

    Ok(())
}
