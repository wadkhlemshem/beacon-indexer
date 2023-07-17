use std::sync::Arc;

use anyhow::Result;
use client::{model::attestation::Attestation, JsonRpcClient};
use futures_util::{stream::BoxStream, StreamExt};
use service::Service;

use crate::util::process_attestation;

pub async fn index_attestations(
    client: Arc<dyn JsonRpcClient>,
    service: Arc<dyn Service>,
    mut stream: BoxStream<'_, Result<Attestation>>,
) -> Result<()> {
    while let Some(attestation) = stream.next().await {
        let attestation = match attestation {
            Ok(attestation) => attestation,
            Err(e) => {
                log::error!("Error receiving attestation: {:?}", e);
                continue;
            }
        };
        let client = client.clone();
        let service = service.clone();
        tokio::spawn(async move {
            match process_attestation(client, service, attestation).await {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Error processing attestation: {:?}", e);
                }
            }
        });
    }
    Ok(())
}
