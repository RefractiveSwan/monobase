use std::collections::HashMap;

use futures_util::future::join_all;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use refractive_swan_mesh_dto::{MeshJobDescriptor, MeshJobResult, MeshNodeId, NodeCapabilities};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HubConfig {
    pub hub_id: String,
    pub base_url: String,
    pub timeout_secs: u64,
}

impl HubConfig {
    pub fn from_env() -> Self {
        let hub_id = std::env::var("refractive_swan_HUB_ID").unwrap_or_else(|_| "hub-local".into());
        let base_url = std::env::var("refractive_swan_HUB_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8080".into());
        let timeout_secs = std::env::var("refractive_swan_HUB_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(15);
        Self {
            hub_id,
            base_url,
            timeout_secs,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeMetadata {
    pub node_id: MeshNodeId,
    pub url: Url,
    pub capabilities: NodeCapabilities,
    pub last_seen_ms: Option<u128>,
}

#[derive(Debug, Default, Clone)]
pub struct NodeRegistry {
    nodes: HashMap<MeshNodeId, NodeMetadata>,
}

impl NodeRegistry {
    pub fn upsert(&mut self, meta: NodeMetadata) {
        self.nodes.insert(meta.node_id.clone(), meta);
    }

    pub fn all(&self) -> Vec<NodeMetadata> {
        self.nodes.values().cloned().collect()
    }

    pub fn get(&self, id: &MeshNodeId) -> Option<NodeMetadata> {
        self.nodes.get(id).cloned()
    }

    pub fn merge(&mut self, metas: impl IntoIterator<Item = NodeMetadata>) {
        for meta in metas {
            self.upsert(meta);
        }
    }
}

#[derive(Debug, Error)]
pub enum HubError {
    #[error("http error: {0}")]
    Http(String),
    #[error("invalid url: {0}")]
    InvalidUrl(String),
}

#[derive(Clone)]
pub struct JobQueue {
    client: reqwest::Client,
    registry: NodeRegistry,
}

impl JobQueue {
    pub fn new(registry: NodeRegistry, timeout_secs: u64) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .expect("build http client");
        Self { client, registry }
    }

    pub async fn dispatch(
        &self,
        job: &MeshJobDescriptor,
        targets: Option<Vec<MeshNodeId>>,
    ) -> Result<Vec<MeshJobResult>, HubError> {
        let nodes: Vec<NodeMetadata> = match targets {
            Some(ids) => ids
                .into_iter()
                .filter_map(|id| self.registry.get(&id))
                .collect(),
            None => self.registry.all(),
        };
        let futures = nodes.into_iter().map(|node| async move {
            let url = node
                .url
                .join("mesh/job")
                .map_err(|err| HubError::InvalidUrl(err.to_string()))?;
            let resp = self
                .client
                .post(url)
                .json(job)
                .send()
                .await
                .map_err(|err| HubError::Http(err.to_string()))?;
            let parsed: MeshJobResult = resp
                .json()
                .await
                .map_err(|err| HubError::Http(err.to_string()))?;
            Ok::<MeshJobResult, HubError>(parsed)
        });
        let joined = join_all(futures).await;
        let mut results = Vec::new();
        for item in joined {
            match item {
                Ok(result) => results.push(result),
                Err(err) => {
                    results.push(MeshJobResult {
                        job_id: job.job_id.clone(),
                        status: refractive_swan_mesh_dto::MeshJobStatus::Failed,
                        metrics: None,
                        output: None,
                        error: Some(refractive_swan_mesh_dto::MeshError {
                            kind: refractive_swan_mesh_dto::MeshErrorKind::NodeUnavailable,
                            code: refractive_swan_mesh_dto::MeshErrorCode::new("hub_dispatch_error"),
                            message: err.to_string(),
                            context: None,
                        }),
                    });
                }
            }
        }
        Ok(results)
    }
}
