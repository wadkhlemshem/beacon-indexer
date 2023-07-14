use deadpool_postgres::{Client, Config, ManagerConfig, Pool, PoolError, RecyclingMethod, Runtime};
use envconfig::Envconfig;
use tokio_postgres::NoTls;

pub mod attestation;
pub mod epoch;
pub mod validator;

#[derive(Envconfig, Clone)]
pub struct DbConfig {
    #[envconfig(from = "DB_HOST")]
    pub host: String,

    #[envconfig(from = "DB_PORT", default = "5432")]
    pub port: u16,

    #[envconfig(from = "DB_USER")]
    pub username: String,

    #[envconfig(from = "DB_PASSWORD")]
    pub password: String,

    #[envconfig(from = "DB_NAME")]
    pub database: String,
}

impl DbConfig {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
}

pub async fn connect(db_config: DbConfig) -> Pool {
    let mut cfg = Config::new();
    cfg.host = Some(db_config.host);
    cfg.port = Some(db_config.port);
    cfg.user = Some(db_config.username);
    cfg.password = Some(db_config.password);
    cfg.dbname = Some(db_config.database);
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap()
}

pub async fn get_client(pool: &Pool) -> Result<Client, PoolError> {
    pool.get().await
}
