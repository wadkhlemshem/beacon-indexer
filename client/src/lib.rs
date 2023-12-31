use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use futures_core::Stream;
use futures_util::StreamExt;
use model::{
    attestation::Attestation,
    block::{BlockHeader, BlockHeaderResponse, BlockId},
    checkpoint::{FinalityCheckpointResponse, FinalityCheckpoints},
    committee::Committee,
    proposer::Proposer,
    state::{StateId, StateRootResponse},
    validator::{ValidatorData, ValidatorId, ValidatorResponse, ValidatorStatus},
};
use serde::de::DeserializeOwned;
use subscription::Subscribable;
use url::Url;

use crate::model::{attestation::AttestationResponse, committee::CommitteeResponse, proposer::ProposerResponse};

pub mod model;
pub mod subscription;
pub mod util;

#[async_trait]
pub trait JsonRpcClient: Sync + Send {
    async fn get_header_for_block(&self, block_id: BlockId) -> Result<Option<BlockHeader>>;
    async fn get_root_for_block(&self, block_id: BlockId) -> Result<String>;
    async fn get_attestations_for_block(&self, block_id: BlockId) -> Result<Option<Vec<Attestation>>>;
    async fn get_root_for_state(&self, state_id: StateId) -> Result<String>;
    async fn get_committees_for_state(
        &self,
        state_id: StateId,
        epoch: Option<u64>,
        index: Option<u8>,
        slot: Option<u64>,
    ) -> Result<Vec<Committee>>;

    async fn validators_for_state(
        &self,
        state_id: StateId,
        id: &[ValidatorId],
        status: Option<ValidatorStatus>,
    ) -> Result<Vec<ValidatorData>>;
    async fn validator_count(&self, state_id: StateId, validator_status: Option<ValidatorStatus>) -> Result<usize>;

    async fn get_finality_checkpoints(&self, state_id: StateId) -> Result<FinalityCheckpoints>;

    async fn get_proposers_for_epoch(&self, epoch: u64) -> Result<Vec<Proposer>>;
}

pub struct HttpClient {
    http_rpc_url: Url,
    client: Arc<reqwest::Client>,
}

impl HttpClient {
    pub fn new(http_rpc_url: Url) -> Self {
        let client = Arc::new(reqwest::Client::new());
        Self { http_rpc_url, client }
    }

    pub async fn subscribe<T: Subscribable + DeserializeOwned>(&self) -> Result<impl Stream<Item = Result<T>>> {
        let event = T::subscribe_event();
        let mut url = self.http_rpc_url.join("eth/v1/events")?;
        url.query_pairs_mut().append_pair("topics", &event.to_string());
        log::debug!("GET {url}");
        // Ok(self.client.get(url).send().await?.bytes_stream())
        let stream = self.client.get(url).send().await?.bytes_stream();
        let stream = stream.map(|bytes| {
            let bytes = bytes?;
            let line = std::str::from_utf8(&bytes)?.lines().collect::<Vec<_>>()[1];
            let data = serde_json::from_str(&line[6..])?;
            Ok(data)
        });

        Ok(stream)
    }
}

#[async_trait]
impl JsonRpcClient for HttpClient {
    async fn get_header_for_block(&self, block_id: BlockId) -> Result<Option<BlockHeader>> {
        let url = self.http_rpc_url.join(&format!("eth/v1/beacon/headers/{block_id}"))?;
        log::debug!("GET {url}");
        let response = self.client.get(url).send().await?;
        match response.error_for_status_ref() {
            Ok(_) => {
                let data = response.json::<BlockHeaderResponse>().await?.data;
                let header = data.header;
                Ok(Some(header))
            }
            Err(err) if err.status().map(|s| s.as_u16()) == Some(404) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    async fn get_root_for_block(&self, block_id: BlockId) -> Result<String> {
        let url = self
            .http_rpc_url
            .join(&format!("eth/v1/beacon/blocks/{block_id}/root"))?;
        log::debug!("GET {url}");
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let data = response.json::<StateRootResponse>().await?.data;
        let root = data.root;
        Ok(root)
    }

    async fn get_attestations_for_block(&self, block_id: BlockId) -> Result<Option<Vec<Attestation>>> {
        let url = self
            .http_rpc_url
            .join(&format!("eth/v1/beacon/blocks/{block_id}/attestations"))?;
        log::debug!("GET {url}");
        let response = self.client.get(url).send().await?;
        match response.error_for_status_ref() {
            Ok(_) => {
                let attestations = response.json::<AttestationResponse>().await?.data;
                Ok(Some(attestations))
            }
            Err(err) if err.status().map(|s| s.as_u16()) == Some(404) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    async fn get_root_for_state(&self, state_id: StateId) -> Result<String> {
        let url = self
            .http_rpc_url
            .join(&format!("eth/v1/beacon/states/{state_id}/root"))?;
        log::debug!("GET {url}");
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let data = response.json::<StateRootResponse>().await?.data;
        let root = data.root;
        Ok(root)
    }

    async fn get_committees_for_state(
        &self,
        state_id: StateId,
        epoch: Option<u64>,
        index: Option<u8>,
        slot: Option<u64>,
    ) -> Result<Vec<Committee>> {
        let mut url = self
            .http_rpc_url
            .join(&format!("eth/v1/beacon/states/{state_id}/committees"))?;
        if let Some(epoch) = epoch {
            url.query_pairs_mut().append_pair("epoch", &epoch.to_string());
        }
        if let Some(index) = index {
            url.query_pairs_mut().append_pair("index", &index.to_string());
        }
        if let Some(slot) = slot {
            url.query_pairs_mut().append_pair("slot", &slot.to_string());
        }
        log::debug!("GET {url}");
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let committees = response.json::<CommitteeResponse>().await?.data;

        Ok(committees)
    }

    async fn validators_for_state(
        &self,
        state_id: StateId,
        id: &[ValidatorId],
        status: Option<ValidatorStatus>,
    ) -> Result<Vec<ValidatorData>> {
        let id = if id.is_empty() {
            None
        } else {
            Some(id.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(","))
        };
        let mut url = self
            .http_rpc_url
            .join(&format!("eth/v1/beacon/states/{state_id}/validators"))?;
        if let Some(status) = status {
            url.query_pairs_mut().append_pair("status", &status.to_string());
        }
        if let Some(id) = id {
            url.query_pairs_mut().append_pair("id", &id);
        }
        log::debug!("GET {url}");
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let body = response.json::<ValidatorResponse>().await?;
        let validators = body.data;
        Ok(validators)
    }

    async fn validator_count(&self, state_id: StateId, validator_status: Option<ValidatorStatus>) -> Result<usize> {
        let mut url = self
            .http_rpc_url
            .join(&format!("eth/v1/beacon/states/{state_id}/validators"))?;
        if let Some(status) = validator_status {
            url.query_pairs_mut().append_pair("status", &status.to_string());
        }
        log::debug!("GET {url}");
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let body = response.json::<ValidatorResponse>().await?;
        let active_validator_count = body.data.len();
        Ok(active_validator_count)
    }

    async fn get_finality_checkpoints(&self, state_id: StateId) -> Result<FinalityCheckpoints> {
        let url = self
            .http_rpc_url
            .join(&format!("eth/v1/beacon/states/{state_id}/finality_checkpoints"))?;
        log::debug!("GET {url}");
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let body = response.json::<FinalityCheckpointResponse>().await?;
        let finality_checkpoints = body.data;
        Ok(finality_checkpoints)
    }

    async fn get_proposers_for_epoch(&self, epoch: u64) -> Result<Vec<Proposer>> {
        let url = self
            .http_rpc_url
            .join(&format!("eth/v1/validator/duties/proposer/{epoch}"))?;
        log::debug!("GET {url}");
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let body = response.json::<ProposerResponse>().await?;
        let proposers = body.data;
        Ok(proposers)
    }
}
