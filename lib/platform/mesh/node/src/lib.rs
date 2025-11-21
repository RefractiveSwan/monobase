//! Mesh node runtime crate.
//!
//! Hosts the reusable `NodeDataPlane` orchestration surface so that HTTP
//! adapters (today `refractive_swan_api`, future `refractive_swan_mesh_node` binaries) can share the
//! same wiring from domain ports → platform stores.

mod config;
pub mod plane;
mod vector;

pub use config::{NodePlaneConfig, NodePlaneConfigError};
pub use plane::NodeDataPlane;
