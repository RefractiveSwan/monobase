//! Mesh-level contracts for node/hub coordination.
//!
//! This module defines the canonical DTOs for mesh operations: node identity,
//! capabilities, job descriptors, results, and errors. These contracts enable
//! cross-node communication while keeping the mesh runtime (`dfps_mesh_node`,
//! `dfps_mesh_hub`) decoupled from domain logic.
//!
//! **Usage constraint**: Only `dfps_mesh_node` and `dfps_mesh_hub` should use
//! these contracts directly. Apps/CLIs remain node-local and use existing
//! contracts (analytics, eval, pipeline).

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

// ============================================================================
// Node Identity & Capabilities
// ============================================================================

/// Opaque identifier for a mesh node.
///
/// Each node in a mesh deployment has a unique ID assigned at startup. The ID
/// is typically a UUID or a stable string derived from node configuration.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MeshNodeId(pub String);

impl MeshNodeId {
    /// Create a new random node ID (UUID v4).
    pub fn new_random() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create a node ID from a known string (e.g., from env or config).
    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    /// Get the underlying string representation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MeshNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Node capabilities advertised to the mesh hub or other nodes.
///
/// Describes the backends, compliance mode, and capacity constraints for a
/// node. The hub uses this metadata to route jobs appropriately.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeCapabilities {
    /// Node identifier.
    pub node_id: MeshNodeId,

    /// Vector backend (Qdrant, PGVector, Milvus, Mock).
    pub vector_backend: String,

    /// Relational warehouse backend (Sqlite, Postgres, Duckdb, External).
    pub warehouse_backend: String,

    /// Compliance mode (e.g., "strict", "permissive", "disabled").
    pub compliance_mode: String,

    /// Maximum dataset size (rows) this node can handle.
    pub max_dataset_size: u64,

    /// Optional tags for routing (e.g., "research", "production", "staging").
    #[serde(default)]
    pub tags: Vec<String>,
}

// ============================================================================
// Job Descriptors & Results
// ============================================================================

/// Mesh job type.
///
/// Defines the category of work being requested. Each type corresponds to a
/// specific set of parameters in `MeshJobDescriptor`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeshJobType {
    /// Run an eval dataset against the mapping pipeline.
    EvalDataset,

    /// Execute an analytics query (NCIt summary, cohort).
    AnalyticsQuery,

    /// Perform a mapping health check (verify pipeline/vector runtime).
    MappingHealthCheck,

    /// Export aggregated data (with DP noise, if required).
    ExportJob,

    /// Node introspection (metrics, capacity, status).
    NodeIntrospection,
}

/// Mesh job descriptor.
///
/// Encapsulates all parameters needed to execute a job on a node. The hub or
/// orchestrator sends this to a node via HTTP/gRPC, and the node's runtime
/// (NodeDataPlane) dispatches it to the appropriate handler.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeshJobDescriptor {
    /// Unique job ID (assigned by hub or caller).
    pub job_id: String,

    /// Job type.
    pub job_type: MeshJobType,

    /// Type-specific parameters (JSON blob).
    ///
    /// For `EvalDataset`: `{ "dataset_name": "...", "top_k": 10 }`
    /// For `AnalyticsQuery`: `{ "query_type": "ncit_summary", "filters": {...} }`
    /// For `MappingHealthCheck`: `{}`
    #[serde(default)]
    pub parameters: serde_json::Value,

    /// Governance context (optional): requesting node ID, reason, etc.
    #[serde(default)]
    pub governance_context: Option<serde_json::Value>,
}

/// Mesh job result.
///
/// Returned by a node after executing a job. Contains status, optional
/// metrics snapshot, and structured output or error.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeshJobResult {
    /// Job ID (echoes the descriptor).
    pub job_id: String,

    /// Execution status.
    pub status: MeshJobStatus,

    /// Optional metrics snapshot captured during execution.
    #[serde(default)]
    pub metrics: Option<serde_json::Value>,

    /// Structured output (job-type specific).
    ///
    /// For `EvalDataset`: `EvalRunResponse`
    /// For `AnalyticsQuery`: `AnalyticsSummaryResponse` or `CohortResponse`
    /// For `MappingHealthCheck`: `{ "ok": true, "latency_ms": 123 }`
    #[serde(default)]
    pub output: Option<serde_json::Value>,

    /// Error details if status is Failed or Denied.
    #[serde(default)]
    pub error: Option<MeshError>,
}

/// Mesh job execution status.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeshJobStatus {
    /// Job completed successfully.
    Success,

    /// Job failed due to internal error.
    Failed,

    /// Job was rejected by governance policy.
    Denied,

    /// Job is still running (used for async/polling workflows).
    InProgress,

    /// Job was cancelled.
    Cancelled,
}

// ============================================================================
// Errors
// ============================================================================

/// Mesh error kind (category).
///
/// Helps the hub or orchestrator understand whether a failure is retryable,
/// a policy denial, or a node-unavailable situation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeshErrorKind {
    /// Node is unavailable or unreachable.
    NodeUnavailable,

    /// Job was rejected by the node (e.g., queue full, unsupported job type).
    JobRejected,

    /// Governance policy denied the request.
    PolicyDenied,

    /// Internal error during job execution.
    Internal,

    /// Invalid job parameters.
    InvalidRequest,
}

/// Mesh error code (string).
///
/// Machine-readable error code for fine-grained diagnostics. Examples:
/// - `"node_unavailable:timeout"`
/// - `"policy_denied:dp_budget_exceeded"`
/// - `"internal:vector_store_down"`
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MeshErrorCode(pub String);

impl MeshErrorCode {
    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MeshErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Mesh error details.
///
/// Structured error information returned in `MeshJobResult::error`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeshError {
    /// Error kind (category).
    pub kind: MeshErrorKind,

    /// Machine-readable error code.
    pub code: MeshErrorCode,

    /// Human-readable error message.
    pub message: String,

    /// Optional additional context (e.g., stack trace, node logs).
    #[serde(default)]
    pub context: Option<serde_json::Value>,
}

impl MeshError {
    /// Create a new mesh error.
    pub fn new(kind: MeshErrorKind, code: MeshErrorCode, message: String) -> Self {
        Self {
            kind,
            code,
            message,
            context: None,
        }
    }

    /// Add context to the error.
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }
}

impl fmt::Display for MeshError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.kind_str(), self.code, self.message)
    }
}

impl MeshError {
    fn kind_str(&self) -> &str {
        match self.kind {
            MeshErrorKind::NodeUnavailable => "node_unavailable",
            MeshErrorKind::JobRejected => "job_rejected",
            MeshErrorKind::PolicyDenied => "policy_denied",
            MeshErrorKind::Internal => "internal",
            MeshErrorKind::InvalidRequest => "invalid_request",
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_node_id_roundtrip() {
        let id = MeshNodeId::new_random();
        let serialized = serde_json::to_string(&id).unwrap();
        let deserialized: MeshNodeId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_node_capabilities_serde() {
        let caps = NodeCapabilities {
            node_id: MeshNodeId::from_string("node-123".to_string()),
            vector_backend: "Qdrant".to_string(),
            warehouse_backend: "Postgres".to_string(),
            compliance_mode: "strict".to_string(),
            max_dataset_size: 1_000_000,
            tags: vec!["research".to_string(), "staging".to_string()],
        };

        let json = serde_json::to_string_pretty(&caps).unwrap();
        let parsed: NodeCapabilities = serde_json::from_str(&json).unwrap();
        assert_eq!(caps, parsed);
    }

    #[test]
    fn test_mesh_job_descriptor_serde() {
        let descriptor = MeshJobDescriptor {
            job_id: "job-456".to_string(),
            job_type: MeshJobType::EvalDataset,
            parameters: serde_json::json!({ "dataset_name": "test_dataset", "top_k": 10 }),
            governance_context: None,
        };

        let json = serde_json::to_string_pretty(&descriptor).unwrap();
        let parsed: MeshJobDescriptor = serde_json::from_str(&json).unwrap();
        assert_eq!(descriptor, parsed);
    }

    #[test]
    fn test_mesh_job_result_success() {
        let result = MeshJobResult {
            job_id: "job-789".to_string(),
            status: MeshJobStatus::Success,
            metrics: Some(serde_json::json!({ "duration_ms": 250 })),
            output: Some(serde_json::json!({ "result": "ok" })),
            error: None,
        };

        let json = serde_json::to_string_pretty(&result).unwrap();
        let parsed: MeshJobResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, parsed);
    }

    #[test]
    fn test_mesh_error() {
        let error = MeshError::new(
            MeshErrorKind::PolicyDenied,
            MeshErrorCode::new("policy_denied:dp_budget_exceeded"),
            "Differential privacy budget exceeded".to_string(),
        )
        .with_context(serde_json::json!({ "remaining_budget": 0.0 }));

        assert_eq!(error.kind, MeshErrorKind::PolicyDenied);
        assert_eq!(error.code.as_str(), "policy_denied:dp_budget_exceeded");
        assert!(error.context.is_some());
    }
}
