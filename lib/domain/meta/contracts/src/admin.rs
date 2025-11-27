use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Structured admin/audit events exposed by backend surfaces so UIs can show a
/// lightweight activity log without parsing raw logs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdminEventKind {
    DatasetUpload,
    DatasetDelete,
    DatasetRefresh,
    DatasetEnable,
    DatasetDisable,
    Compliance,
    Vector,
    Datamart,
    Ingestion,
    Toggle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub kind: AdminEventKind,
    pub message: String,
    /// Optional mesh node identifier when the event originates from a mesh-aware node.
    #[serde(default)]
    pub mesh_node_id: Option<String>,
}
