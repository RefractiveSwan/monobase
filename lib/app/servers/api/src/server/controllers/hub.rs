use axum::Json;
use serde::{Deserialize, Serialize};

use crate::utils::ApiState;
use axum::response::IntoResponse;
use log::info;
use refractive_swan_contracts::{AnalyticsSummaryResponse, FederatedEvalSummary};
use refractive_swan_mesh_hub::{
    HubConfig, JobQueue, NodeMetadata, NodeRegistry, NodeStatus, analytics::global_ncit_summary,
    eval::federated_eval as hub_federated_eval,
};
use reqwest::Url;

#[derive(Debug, Serialize)]
pub struct HubNodesResponse {
    pub nodes: Vec<HubNodeEntry>,
}

#[derive(Debug, Serialize)]
pub struct HubNodeEntry {
    pub node_id: String,
    pub capabilities: serde_json::Value,
    pub metrics: serde_json::Value,
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
    let metrics_json = serde_json::to_value(metrics.lock().await.clone())
        .unwrap_or_else(|_| serde_json::json!({}));
    Json(HubNodesResponse {
        nodes: vec![HubNodeEntry {
            node_id: capabilities.node_id.to_string(),
            capabilities: serde_json::to_value(capabilities)
                .unwrap_or_else(|_| serde_json::json!({})),
            metrics: metrics_json,
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
    let metrics_json = serde_json::to_value(metrics.lock().await.clone())
        .unwrap_or_else(|_| serde_json::json!({}));
    if capabilities.node_id.to_string() == id {
        return Json(HubNodeEntry {
            node_id: capabilities.node_id.to_string(),
            capabilities: serde_json::to_value(capabilities)
                .unwrap_or_else(|_| serde_json::json!({})),
            metrics: metrics_json,
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
        .unwrap_or_else(|err| AnalyticsSummaryResponse { rows: vec![] });
    Json(aggregated)
}

/// Federated eval (single-node stub) requiring `dataset` query parameter.
pub async fn federated_eval(
    state: axum::extract::State<ApiState>,
    axum::extract::Query(query): axum::extract::Query<EvalQuery>,
) -> impl axum::response::IntoResponse {
    let queue = local_queue(&state);
    let aggregated = hub_federated_eval(&queue, &query.dataset)
        .await
        .unwrap_or_else(|_| FederatedEvalSummary::default());
    Json(aggregated)
}

#[derive(Debug, Deserialize)]
pub struct EvalQuery {
    pub dataset: String,
}

fn local_queue(state: &ApiState) -> JobQueue {
    let capabilities = state.plane.capabilities("");
    let url = std::env::var("refractive_swan_API_BASE_URL")
        .ok()
        .and_then(|raw| Url::parse(&raw).ok())
        .unwrap_or_else(|| Url::parse("http://127.0.0.1:8080").unwrap());
    let meta = NodeMetadata {
        node_id: capabilities.node_id.clone(),
        url,
        capabilities,
        last_seen_ms: None,
        status: NodeStatus::Unknown,
    };
    let mut registry = NodeRegistry::default();
    registry.upsert(meta);
    let hub_cfg = HubConfig::from_env();
    info!(
        target: "refractive_swan_api",
        "hub queue built with hub_id={} timeout={}s",
        hub_cfg.hub_id,
        hub_cfg.timeout_secs
    );
    JobQueue::new(registry, hub_cfg.timeout_secs)
}

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
