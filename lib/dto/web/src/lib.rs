//! Shared DTOs for the refractive_swan web surfaces (frontend + API).
//! Re-exports the canonical contracts used by HTTP handlers and clients so
//! both sides stay in sync.

pub use refractive_swan_contracts::{
    AdminEvent, AdminEventKind, AnalyticsSummaryResponse, AnalyticsSummaryRow, CohortResponse,
    CohortRow, DatasetListEntry, DatasetManifest, EvalRunResponse, EvalSummary, PipelineMetrics,
    PipelineOutput,
};

pub use refractive_swan_contracts::errors::{ErrorCode, ErrorKind};

use refractive_swan_contracts::MeshNodeId;
use serde::{Deserialize, Serialize};

/// Subset of node metadata + last metrics for mesh dashboards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeView {
    pub node_id: MeshNodeId,
    pub status: Option<String>,
    pub last_seen_ms: Option<u128>,
    pub vector_backend: String,
    pub warehouse_backend: String,
    pub cache_backend: Option<String>,
    pub compliance_mode: String,
    pub tags: Vec<String>,
    pub metrics: Option<PipelineMetrics>,
}

/// Federated eval view for frontend consumption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedEvalView {
    pub dataset: String,
    pub aggregated: EvalSummary,
    pub per_node: Vec<NodeEvalView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEvalView {
    pub node_id: MeshNodeId,
    pub summary: EvalSummary,
}

/// Mesh toggles surface that extends `/admin/toggles` with mesh/hub state.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MeshTogglesView {
    pub mesh_enabled: bool,
    pub hub_enabled: bool,
    pub hub_url: Option<String>,
    pub node_id: Option<String>,
    pub cache_backend: Option<String>,
    pub warehouse_backend: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_view_serializes() {
        let view = NodeView {
            node_id: MeshNodeId("node-a".into()),
            status: Some("online".into()),
            last_seen_ms: Some(1234),
            vector_backend: "qdrant".into(),
            warehouse_backend: "sqlite".into(),
            cache_backend: Some("redis".into()),
            compliance_mode: "internal".into(),
            tags: vec!["dev".into()],
            metrics: None,
        };
        let json = serde_json::to_string(&view).unwrap();
        let roundtrip: NodeView = serde_json::from_str(&json).unwrap();
        assert_eq!(view.node_id, roundtrip.node_id);
        assert_eq!(view.cache_backend, roundtrip.cache_backend);
        assert_eq!(view.status, roundtrip.status);
    }
}
