//! Mapping-layer domain types used by the NCIt/UMLS integration.
//!
//! These structs back the flows described in
//! `docs/system-design/ncit/architecture/system-architecture.md`,
//! `docs/system-design/ncit/models/data-model-er.md`, and
//! `docs/system-design/ncit/behavior/sequence-servicerequest.md`.
//! They bridge flattened staging codes into mapping candidates/results
//! with NCIt concept metadata for downstream analytics.
//!
//! Card: REFR-04.

pub mod bridge;
pub mod concept;
pub mod element;
pub mod result;
pub mod state;
pub mod strategy;
pub mod thresholds;
pub mod version;

#[cfg(test)]
mod tests;

pub use concept::{DimNCITConcept, NCItConcept};
pub use element::CodeElement;
pub use result::{MappingCandidate, MappingResult};
pub use state::MappingState;
pub use strategy::MappingStrategy;
pub use thresholds::MappingThresholds;
pub use version::MappingSourceVersion;
