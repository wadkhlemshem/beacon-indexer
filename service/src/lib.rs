pub mod epoch;
pub mod validator;

use async_trait::async_trait;

#[async_trait]
pub trait Service: Sync + Send {
    async fn get_participation_rate_for_epoch(epoch: u64) -> f64;
    async fn get_participation_rate_for_validator(validator: u64) -> f64;
    async fn create_validator(validator: u64) -> bool;
    async fn update_validator(validator: u64) -> bool;
    async fn create_epoch(epoch: u64) -> bool;
    async fn update_epoch(epoch: u64) -> bool;
}
