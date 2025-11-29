use std::collections::BTreeMap;

use axum::Json;
use log::info;
use refractive_swan_compliance::ComplianceAction;
use refractive_swan_contracts::{MeshJobDescriptor, MeshJobResult};
use refractive_swan_mesh_dto::NodeCapabilities;
use refractive_swan_observability::metrics_snapshot_json;
use serde::Serialize;
use uuid::Uuid;

use crate::utils::ApiState;

#[derive(Debug, Serialize)]
pub struct MeshHealthResponse {
    pub status: &'static str,
    pub node_id: String,
    pub metrics: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct MeshGovernanceResponse {
    pub mode: String,
    pub allowed_actions: Vec<String>,
    pub allowed_tiers: BTreeMap<String, Vec<String>>,
    pub dp_budget_remaining: Option<f32>,
}

/// Health endpoint for mesh-aware nodes that includes metrics snapshot.
pub async fn health(state: axum::extract::State<ApiState>) -> impl axum::response::IntoResponse {
    let request_id = Uuid::new_v4();
    let metrics_handle = state.plane.metrics();
    let metrics = metrics_handle.lock().await.clone();
    let metrics_snapshot = metrics_snapshot_json(&metrics);
    let node_id = state.plane.node_id().to_string();
    info!(
        target: "refractive_swan_api",
        "request_id={request_id} mesh_health node_id={node_id} bundles={} mappings={}",
        metrics.bundle_count,
        metrics.mapping_count
    );
    Json(MeshHealthResponse {
        status: "ok",
        node_id,
        metrics: metrics_snapshot,
    })
}

/// Advertise node capabilities to hubs/frontends.
pub async fn capabilities(
    state: axum::extract::State<ApiState>,
) -> impl axum::response::IntoResponse {
    let request_id = Uuid::new_v4();
    let capabilities: NodeCapabilities = state.plane.capabilities("");
    info!(
        target: "refractive_swan_api",
        "request_id={request_id} mesh_capabilities node_id={} vector={} warehouse={} compliance_mode={}",
        capabilities.node_id,
        capabilities.vector_backend,
        capabilities.warehouse_backend,
        capabilities.compliance_mode
    );
    Json(capabilities)
}

/// Governance surface for mesh nodes.
pub async fn governance(
    state: axum::extract::State<ApiState>,
) -> impl axum::response::IntoResponse {
    let request_id = Uuid::new_v4();
    let policy = state.plane.policy().clone();
    let response = MeshGovernanceResponse::from_policy(&policy);
    info!(
        target: "refractive_swan_api",
        "request_id={request_id} mesh_governance node_id={} mode={}",
        state.plane.node_id(),
        policy.mode.as_str()
    );
    Json(response)
}

/// Execute a mesh job on the node.
pub async fn mesh_job(
    state: axum::extract::State<ApiState>,
    axum::Json(job): axum::Json<MeshJobDescriptor>,
) -> impl axum::response::IntoResponse {
    let request_id = Uuid::new_v4();
    let result: MeshJobResult = state.plane.run_mesh_job(&job).await;
    info!(
        target: "refractive_swan_api",
        "request_id={request_id} mesh_job job_id={} status={:?}",
        job.job_id,
        result.status
    );
    Json(result)
}

impl MeshGovernanceResponse {
    fn from_policy(policy: &refractive_swan_compliance::Policy) -> Self {
        let allowed_actions: Vec<String> = policy
            .allowed_actions
            .iter()
            .map(compliance_action_label)
            .map(String::from)
            .collect();
        let mut allowed_tiers: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (action, tiers) in &policy.allowed_tiers {
            let label = compliance_action_label(action).to_string();
            let tier_labels = tiers.iter().map(|tier| tier.as_str().to_string()).collect();
            allowed_tiers.insert(label, tier_labels);
        }
        Self {
            mode: policy.mode.as_str().to_string(),
            allowed_actions,
            allowed_tiers,
            dp_budget_remaining: None,
        }
    }
}

fn compliance_action_label(action: &ComplianceAction) -> &'static str {
    match action {
        ComplianceAction::Ingest => "ingest",
        ComplianceAction::Map => "map",
        ComplianceAction::Export => "export",
    }
}
