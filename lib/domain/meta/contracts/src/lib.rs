//! Canonical contracts shared across CLI, HTTP, datamart, and eval surfaces.
//! Tracks REFR-03.

pub mod admin;
pub mod analytics;
pub mod errors;
pub mod eval;
pub mod federated;
pub mod mesh;
pub mod metrics;
pub mod pipeline;
pub mod warehouse;

pub use admin::*;
pub use analytics::*;
pub use errors::*;
pub use eval::*;
pub use federated::*;
pub use mesh::*;
pub use metrics::*;
pub use pipeline::*;
pub use warehouse::*;
