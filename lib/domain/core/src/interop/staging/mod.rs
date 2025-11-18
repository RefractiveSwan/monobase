//! Staging-layer row structs for the ingestion MVP.
//!
//! These types correspond to the landing tables shown in
//! `docs/system-design/fhir/architecture/system-architecture.md`,
//! `docs/system-design/fhir/models/data-model-er.md`, and the step-by-step
//! interactions in `docs/system-design/fhir/behavior/sequence-servicerequest.md`.
//! They capture flattened ServiceRequest fields before NCIt/UMLS enrichment.
//!
//! Card: REFR-04.

pub mod rows;

#[cfg(test)]
mod tests;

pub use rows::{StgServiceRequestFlat, StgSrCodeExploded};
