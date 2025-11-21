//! Mesh DTO veneer for node/hub control-plane payloads.
//!
//! Re-exports mesh contracts from `refractive_swan_contracts` so mesh runtimes
//! (`refractive_swan_mesh_node`, `refractive_swan_mesh_hub`, governance services) depend on a
//! curated surface rather than the full contracts crate.
//!
//! See:
//! - docs/system-design/base/directory-architecture.md
//! - docs/kanban/feature/mvp/040-infra-and-docs/027-architecture-refinement.md

pub use refractive_swan_contracts::{
    MeshError, MeshErrorCode, MeshErrorKind, MeshJobDescriptor, MeshJobResult, MeshJobStatus,
    MeshJobType, MeshNodeId, NodeCapabilities,
};
