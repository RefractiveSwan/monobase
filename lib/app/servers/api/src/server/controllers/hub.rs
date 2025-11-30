use axum::Json;
use serde::{Deserialize, Serialize};

use crate::utils::ApiState;
use axum::response::IntoResponse;
use log::info;
use refractive_swan_contracts::AnalyticsSummaryResponse;
use refractive_swan_mesh_dto::NodeCapabilities;
use refractive_swan_mesh_hub::{
    HubConfig, JobQueue, NodeMetadata, NodeRegistry, NodeStatus, analytics::global_ncit_summary,
    eval::federated_eval as hub_federated_eval,
};
use refractive_swan_web_dto::{FederatedEvalView, NodeEvalView, NodeView};
use reqwest::Url;

#[derive(Debug, Serialize)]
pub struct HubNodesResponse {
    pub nodes: Vec<NodeView>,
}

#[derive(Debug, Serialize)]
pub struct HubJobResponse {
    pub status: &'static str,
    pub message: String,
}

/// Stubbed hub node listing (local node only for now).
pub async fn list_nodes(
    state: axum::extract::State<ApiState>,
) -> impl axum::response::IntoResponse {
    let capabilities = state.plane.capabilities("");
    let metrics = state.plane.metrics();
    let metrics = metrics.lock().await.clone();
    Json(HubNodesResponse {
        nodes: vec![NodeView {
            node_id: capabilities.node_id.clone(),
            status: Some(NodeStatus::Online.to_string()),
            last_seen_ms: None,
            vector_backend: capabilities.vector_backend,
            warehouse_backend: capabilities.warehouse_backend,
            cache_backend: None,
            compliance_mode: capabilities.compliance_mode,
            tags: capabilities.tags,
            metrics: Some(metrics),
        }],
    })
}

/// Detail view for a specific node (single-node stub).
pub async fn node_details(
    state: axum::extract::State<ApiState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let capabilities = state.plane.capabilities("");
    let metrics = state.plane.metrics();
    let metrics = metrics.lock().await.clone();
    if capabilities.node_id.to_string() == id {
        return Json(NodeView {
            node_id: capabilities.node_id.clone(),
            status: Some(NodeStatus::Online.to_string()),
            last_seen_ms: None,
            vector_backend: capabilities.vector_backend,
            warehouse_backend: capabilities.warehouse_backend,
            cache_backend: None,
            compliance_mode: capabilities.compliance_mode,
            tags: capabilities.tags,
            metrics: Some(metrics),
        })
        .into_response();
    }
    axum::http::StatusCode::NOT_FOUND.into_response()
}

/// Federated NCIt summary (single-node stub).
pub async fn federated_ncit_summary(
    state: axum::extract::State<ApiState>,
) -> impl axum::response::IntoResponse {
    let queue = local_queue(&state);
    let aggregated = global_ncit_summary(&queue)
        .await
        .unwrap_or_else(|_err| AnalyticsSummaryResponse { rows: vec![] });
    Json(aggregated)
}

/// Federated eval (single-node stub) requiring `dataset` query parameter.
pub async fn federated_eval(
    state: axum::extract::State<ApiState>,
    axum::extract::Query(query): axum::extract::Query<EvalQuery>,
) -> impl axum::response::IntoResponse {
    let queue = local_queue(&state);
    let summary = hub_federated_eval(&queue, &query.dataset)
        .await
        .unwrap_or_default();
    let per_node = summary
        .by_node
        .into_iter()
        .map(|(node_id, summary)| NodeEvalView {
            node_id: refractive_swan_contracts::MeshNodeId(node_id),
            summary,
        })
        .collect();
    Json(FederatedEvalView {
        dataset: query.dataset,
        aggregated: summary.aggregated,
        per_node,
    })
}

#[derive(Debug, Deserialize)]
pub struct EvalQuery {
    pub dataset: String,
}

fn local_queue(state: &ApiState) -> JobQueue {
    let hub_cfg = HubConfig::from_env();
    let mut registry = NodeRegistry::default();
    if !hub_cfg.seed_nodes.is_empty() {
        for seed in &hub_cfg.seed_nodes {
            let caps = NodeCapabilities {
                node_id: seed.node_id.clone(),
                vector_backend: "unknown".into(),
                warehouse_backend: "unknown".into(),
                compliance_mode: "unknown".into(),
                max_dataset_size: 0,
                tags: vec![],
            };
            registry.upsert(NodeMetadata {
                node_id: seed.node_id.clone(),
                url: seed.url.clone(),
                capabilities: caps,
                last_seen_ms: None,
                status: NodeStatus::Unknown,
            });
        }
    } else {
        let capabilities = state.plane.capabilities("");
        let url = std::env::var("refractive_swan_API_BASE_URL")
            .ok()
            .and_then(|raw| Url::parse(&raw).ok())
            .unwrap_or_else(|| Url::parse("http://127.0.0.1:8080").unwrap());
        registry.upsert(NodeMetadata {
            node_id: capabilities.node_id.clone(),
            url,
            capabilities,
            last_seen_ms: None,
            status: NodeStatus::Unknown,
        });
    }
    info!(
        target: "refractive_swan_api",
        "hub queue built with hub_id={} timeout={}s",
        hub_cfg.hub_id,
        hub_cfg.timeout_secs
    );
    JobQueue::new(registry, hub_cfg.timeout_secs)
}

#[allow(dead_code)]
fn error_result(job_id: String, message: String) -> refractive_swan_contracts::MeshJobResult {
    refractive_swan_contracts::MeshJobResult {
        job_id,
        status: refractive_swan_contracts::MeshJobStatus::Failed,
        metrics: None,
        output: None,
        error: Some(refractive_swan_contracts::MeshError {
            kind: refractive_swan_contracts::MeshErrorKind::Internal,
            code: refractive_swan_contracts::MeshErrorCode::new("hub_dispatch_error"),
            message,
            context: None,
        }),
    }
}
