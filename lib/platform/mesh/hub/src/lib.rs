use std::collections::HashMap;

use futures_util::StreamExt;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;

use refractive_swan_configuration::load_env;
use refractive_swan_mesh_dto::{
    MeshError, MeshErrorCode, MeshErrorKind, MeshJobDescriptor, MeshJobResult, MeshJobStatus,
    MeshNodeId, NodeCapabilities,
};

pub mod analytics;
pub mod eval;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HubConfig {
    pub hub_id: String,
    pub base_url: String,
    pub timeout_secs: u64,
}

impl HubConfig {
    pub fn from_env() -> Self {
        // Best-effort load of namespaced env; non-strict so hub can start with defaults.
        let _ = load_env("platform.mesh.hub");
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    Unknown,
    Online,
    Offline,
}

impl Default for NodeStatus {
    fn default() -> Self {
        NodeStatus::Unknown
    }
}

#[derive(Debug, Clone)]
pub struct NodeMetadata {
    pub node_id: MeshNodeId,
    pub url: Url,
    pub capabilities: NodeCapabilities,
    pub last_seen_ms: Option<u128>,
    pub status: NodeStatus,
}

#[derive(Debug, Default, Clone)]
pub struct NodeRegistry {
    nodes: HashMap<MeshNodeId, NodeMetadata>,
    stats: HashMap<MeshNodeId, NodeStats>,
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

    pub fn record_success(&mut self, node_id: &MeshNodeId) {
        let entry = self.stats.entry(node_id.clone()).or_default();
        entry.successes += 1;
        entry.last_error = None;
    }

    pub fn record_failure(&mut self, node_id: &MeshNodeId, error: String) {
        let entry = self.stats.entry(node_id.clone()).or_default();
        entry.failures += 1;
        entry.last_error = Some(error);
    }

    pub fn stats(&self, node_id: &MeshNodeId) -> Option<&NodeStats> {
        self.stats.get(node_id)
    }
}

#[derive(Debug, Default, Clone)]
pub struct NodeStats {
    pub successes: u64,
    pub failures: u64,
    pub last_error: Option<String>,
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
    registry: std::sync::Arc<Mutex<NodeRegistry>>,
    concurrency: usize,
}

impl JobQueue {
    pub fn new(registry: NodeRegistry, timeout_secs: u64) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .expect("build http client");
        Self {
            client,
            registry: std::sync::Arc::new(Mutex::new(registry)),
            concurrency: 4,
        }
    }

    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency.max(1);
        self
    }

    pub async fn dispatch(
        &self,
        job: &MeshJobDescriptor,
        targets: Option<Vec<MeshNodeId>>,
    ) -> Result<Vec<MeshJobResult>, HubError> {
        let nodes: Vec<NodeMetadata> = {
            let registry = self.registry.lock().await;
            match targets {
                Some(ids) => ids.into_iter().filter_map(|id| registry.get(&id)).collect(),
                None => registry.all(),
            }
        };

        let client = self.client.clone();
        let registry = self.registry.clone();
        let job_id = job.job_id.clone();

        let mut stream = futures_util::stream::iter(nodes.into_iter().map(|node| {
            let client = client.clone();
            let registry = registry.clone();
            let job = job.clone();
            async move {
                let node_id = node.node_id.clone();
                let result: Result<MeshJobResult, HubError> = async {
                    let url = node
                        .url
                        .join("mesh/job")
                        .map_err(|err| HubError::InvalidUrl(err.to_string()))?;
                    let resp = client
                        .post(url)
                        .json(&job)
                        .send()
                        .await
                        .map_err(|err| HubError::Http(err.to_string()))?;
                    let parsed: MeshJobResult = resp
                        .json()
                        .await
                        .map_err(|err| HubError::Http(err.to_string()))?;
                    Ok(parsed)
                }
                .await;

                let mut reg = registry.lock().await;
                match &result {
                    Ok(_) => reg.record_success(&node_id),
                    Err(err) => reg.record_failure(&node_id, err.to_string()),
                }
                result.map_err(|e| (node_id, e))
            }
        }))
        .buffer_unordered(self.concurrency);

        let mut results = Vec::new();
        while let Some(item) = stream.next().await {
            match item {
                Ok(result) => results.push(result),
                Err((node_id, err)) => results.push(MeshJobResult {
                    job_id: job_id.clone(),
                    status: MeshJobStatus::Failed,
                    metrics: None,
                    output: None,
                    error: Some(MeshError {
                        kind: MeshErrorKind::NodeUnavailable,
                        code: MeshErrorCode::new("hub_dispatch_error"),
                        message: format!("{}: {}", node_id, err),
                        context: None,
                    }),
                }),
            }
        }

        Ok(results)
    }
}

impl JobQueue {
    pub fn registry(&self) -> std::sync::Arc<Mutex<NodeRegistry>> {
        self.registry.clone()
    }

    pub fn http_client(&self) -> reqwest::Client {
        self.client.clone()
    }
}

/// Register a node that proactively advertises its capabilities.
pub async fn register_node(
    registry: &Mutex<NodeRegistry>,
    capabilities: NodeCapabilities,
    url: Url,
) {
    let meta = NodeMetadata {
        node_id: capabilities.node_id.clone(),
        url,
        capabilities,
        last_seen_ms: Some(now_ms()),
        status: NodeStatus::Online,
    };
    let mut guard = registry.lock().await;
    guard.upsert(meta);
}

/// Health check nodes and update registry status/stats.
pub async fn health_check_nodes(queue: &JobQueue) {
    let nodes = {
        let reg = queue.registry.lock().await;
        reg.all()
    };
    let client = queue.http_client();
    let registry = queue.registry();
    let mut stream = futures_util::stream::iter(nodes.into_iter().map(|node| {
        let client = client.clone();
        let registry = registry.clone();
        async move {
            let health_url = match node.url.join("mesh/health") {
                Ok(url) => url,
                Err(err) => {
                    let mut reg = registry.lock().await;
                    reg.record_failure(&node.node_id, err.to_string());
                    reg.upsert(NodeMetadata {
                        status: NodeStatus::Offline,
                        ..node
                    });
                    return;
                }
            };
            let resp = client.get(health_url).send().await;
            let mut reg = registry.lock().await;
            match resp {
                Ok(ok) if ok.status().is_success() => {
                    reg.record_success(&node.node_id);
                    reg.upsert(NodeMetadata {
                        status: NodeStatus::Online,
                        last_seen_ms: Some(now_ms()),
                        ..node
                    });
                }
                Ok(failed) => {
                    reg.record_failure(&node.node_id, failed.status().to_string());
                    reg.upsert(NodeMetadata {
                        status: NodeStatus::Offline,
                        ..node
                    });
                }
                Err(err) => {
                    reg.record_failure(&node.node_id, err.to_string());
                    reg.upsert(NodeMetadata {
                        status: NodeStatus::Offline,
                        ..node
                    });
                }
            }
        }
    }))
    .buffer_unordered(queue.concurrency);

    while let Some(_) = stream.next().await {}
}

fn now_ms() -> u128 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// Minimal hub runtime wrapper around config/registry/queue.
#[derive(Clone)]
pub struct HubRuntime {
    pub config: HubConfig,
    pub queue: JobQueue,
}

impl HubRuntime {
    pub fn new(config: HubConfig, registry: NodeRegistry) -> Self {
        let queue = JobQueue::new(registry, config.timeout_secs);
        Self { config, queue }
    }

    pub async fn list_nodes(&self) -> Vec<NodeMetadata> {
        let reg = self.queue.registry.lock().await;
        reg.all()
    }

    /// Register a node proactively.
    pub async fn register_node(&self, capabilities: NodeCapabilities, url: Url) {
        crate::register_node(&self.queue.registry, capabilities, url).await;
    }

    /// Poll mesh node health and update registry status/stats.
    pub async fn health_check_nodes(&self) {
        crate::health_check_nodes(&self.queue).await;
    }

    pub async fn global_ncit_summary(
        &self,
    ) -> Result<refractive_swan_contracts::AnalyticsSummaryResponse, HubError> {
        crate::analytics::global_ncit_summary(&self.queue).await
    }

    pub async fn federated_eval(
        &self,
        dataset: &str,
    ) -> Result<refractive_swan_contracts::FederatedEvalSummary, HubError> {
        crate::eval::federated_eval(&self.queue, dataset).await
    }
}

#[cfg(all(test, feature = "test-util"))]
mod tests {
    use super::*;
    use httpmock::MockServer;
    use reqwest::Url;

    fn registry_with_node(server: &MockServer, path: &str) -> NodeRegistry {
        let mut reg = NodeRegistry::default();
        let capabilities = NodeCapabilities {
            node_id: MeshNodeId("node-a".into()),
            vector_backend: "mock".into(),
            warehouse_backend: "sqlite".into(),
            compliance_mode: "open".into(),
            max_dataset_size: 10,
            tags: vec!["dev".into()],
        };
        reg.upsert(NodeMetadata {
            node_id: capabilities.node_id.clone(),
            url: Url::parse(&format!("{}{}", server.base_url(), path)).unwrap(),
            capabilities,
            last_seen_ms: None,
            status: NodeStatus::Unknown,
        });
        reg
    }

    #[tokio::test]
    async fn register_node_inserts_metadata() {
        let registry = Mutex::new(NodeRegistry::default());
        let caps = NodeCapabilities {
            node_id: MeshNodeId("node-reg".into()),
            vector_backend: "mock".into(),
            warehouse_backend: "sqlite".into(),
            compliance_mode: "open".into(),
            max_dataset_size: 1,
            tags: vec![],
        };
        let url = Url::parse("http://localhost:8080").unwrap();
        register_node(&registry, caps.clone(), url.clone()).await;
        let guard = registry.lock().await;
        let meta = guard.get(&caps.node_id).expect("node present");
        assert_eq!(meta.node_id, caps.node_id);
        assert_eq!(meta.url, url);
        assert_eq!(meta.status, NodeStatus::Online);
    }

    #[tokio::test]
    async fn health_check_nodes_marks_online_and_offline() {
        let server = MockServer::start();
        let _health_ok = server.mock(|when, then| {
            when.path("/ok/mesh/health");
            then.status(200);
        });
        let _health_fail = server.mock(|when, then| {
            when.path("/fail/mesh/health");
            then.status(500);
        });

        let mut reg = NodeRegistry::default();
        let base_caps = NodeCapabilities {
            node_id: MeshNodeId("node-a".into()),
            vector_backend: "mock".into(),
            warehouse_backend: "sqlite".into(),
            compliance_mode: "open".into(),
            max_dataset_size: 10,
            tags: vec![],
        };
        reg.upsert(NodeMetadata {
            node_id: base_caps.node_id.clone(),
            url: Url::parse(&format!("{}/ok/", server.base_url())).unwrap(),
            capabilities: base_caps.clone(),
            last_seen_ms: None,
            status: NodeStatus::Unknown,
        });
        reg.upsert(NodeMetadata {
            node_id: MeshNodeId("node-b".into()),
            url: Url::parse(&format!("{}/fail/", server.base_url())).unwrap(),
            capabilities: base_caps.clone(),
            last_seen_ms: None,
            status: NodeStatus::Unknown,
        });

        let queue = JobQueue::new(reg, 5);
        health_check_nodes(&queue).await;
        let guard = queue.registry.lock().await;
        let node_a = guard.get(&MeshNodeId("node-a".into())).unwrap();
        let node_b = guard.get(&MeshNodeId("node-b".into())).unwrap();
        assert_eq!(node_a.status, NodeStatus::Online);
        assert_eq!(node_b.status, NodeStatus::Offline);
        let stats_a = guard.stats(&node_a.node_id).unwrap();
        let stats_b = guard.stats(&node_b.node_id).unwrap();
        assert!(stats_a.successes >= 1);
        assert!(stats_b.failures >= 1);
    }
}
