//! Strongly typed value objects (IDs) for the refractive_swan domain model.
//!
//! These types enforce invariants discussed in
//! `docs/system-design/fhir/models/data-model-er.md` and provide the anchors
//! for ServiceRequest/Encounter relationships.
//!
//! Card: REFR-04.

pub mod ids;

#[cfg(test)]
mod tests;

pub use ids::{EncounterId, PatientId, ServiceRequestId};
