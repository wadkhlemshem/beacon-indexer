use std::{net::TcpListener, sync::Arc};

use actix_web::{guard, middleware, web, App, HttpServer};
use anyhow::Result;
use api::Query;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use envconfig::Envconfig;
use store::{
    attestation::PostgresAttestationRepository, committee::PostgresCommitteeRepository, epoch::PostgresEpochRepository,
    validator::PostgresValidatorRepository, DbConfig,
};
use url::Url;

#[derive(Envconfig, Clone)]
pub struct AppConfig {
    #[envconfig(from = "HTTP_RPC_URL")]
    pub http_rpc_url: Url,

    #[envconfig(from = "APP_HOST", default = "127.0.0.1")]
    pub host: String,

    #[envconfig(from = "APP_PORT", default = "8080")]
    pub port: u16,
}

impl AppConfig {
    pub fn connection_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let db_config = DbConfig::init_from_env()?;
    let db_pool = store::connect(db_config).await;

    let app_config = AppConfig::init_from_env()?;
    let listener = TcpListener::bind(app_config.connection_string())?;

    let epoch_repository = Arc::new(PostgresEpochRepository::new(db_pool.clone()));
    let validator_repository = Arc::new(PostgresValidatorRepository::new(db_pool.clone()));
    let attestation_repository = Arc::new(PostgresAttestationRepository::new(db_pool.clone()));
    let committee_repository = Arc::new(PostgresCommitteeRepository::new(db_pool.clone()));
    let service = Arc::new(service::ServiceImpl::new(
        epoch_repository,
        validator_repository,
        attestation_repository,
        committee_repository,
    ));

    let schema = Schema::build(Query::default(), EmptyMutation, EmptySubscription)
        .data(service)
        .finish();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .wrap(middleware::Logger::default())
            .service(web::resource("/").guard(guard::Get()).to(api::index_playground))
            .service(web::resource("/").guard(guard::Post()).to(api::index))
    })
    .listen(listener)?
    .run();

    Ok(server.await?)
}
