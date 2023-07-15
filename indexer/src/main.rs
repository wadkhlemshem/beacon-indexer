use std::sync::Arc;

use anyhow::{anyhow, Result};
use client::RpcClient;
use envconfig::Envconfig;
use indexer::Indexer;
use service::{Service, ServiceImpl};
use store::{
    attestation::PostgresAttestationRepository, epoch::PostgresEpochRepository, validator::PostgresValidatorRepository,
    DbConfig,
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
    let client = RpcClient::new(indexer_config.http_rpc_url);

    let db_config = DbConfig::init_from_env()?;
    let db_pool = store::connect(db_config).await;

    let client = Arc::new(client);
    let epoch_repository = Arc::new(PostgresEpochRepository::new(db_pool.clone()));
    let validator_repository = Arc::new(PostgresValidatorRepository::new(db_pool.clone()));
    let attestation_repository = Arc::new(PostgresAttestationRepository::new(db_pool));
    let service = Arc::new(ServiceImpl::new(
        epoch_repository,
        validator_repository,
        attestation_repository,
    ));
    let indexer = Indexer::new(client.clone(), service.clone());
    indexer.index_current_validators().await?;
    let epoch = 214889;
    indexer.run_for_epoch(epoch).await?;

    let epoch = service.get_epoch(214889).await?.ok_or(anyhow!("Epoch not found"))?;
    let attestations_count = epoch.attestations;
    let active_validator_count = epoch.active_validators;
    println!("attestations count = {}", attestations_count);
    let participation_rate = (attestations_count as f64) / (active_validator_count as f64);
    println!("participation rate = {}", participation_rate);

    Ok(())
}
