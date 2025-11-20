//! Canonical contracts shared across CLI, HTTP, datamart, and eval surfaces.
//! Tracks REFR-03.

pub mod analytics;
pub mod errors;
pub mod eval;
pub mod metrics;
pub mod pipeline;
pub mod warehouse;

pub use analytics::*;
pub use errors::*;
pub use eval::*;
pub use metrics::*;
pub use pipeline::*;
pub use warehouse::*;
