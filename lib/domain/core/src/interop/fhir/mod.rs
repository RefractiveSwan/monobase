//! Minimal FHIR R4/R5 resource representations for the ingestion MVP.
//!
//! The structs in this module directly align with the ServiceRequest pipeline
//! diagrams in `docs/system-design/fhir/architecture/system-architecture.md`,
//! `docs/system-design/fhir/models/data-model-er.md`, and the sequence flow
//! described in `docs/system-design/fhir/behavior/sequence-servicerequest.md`.
//! They keep only the subset of fields required to parse raw Bundles and feed
//! staging + domain models without committing to a full FHIR implementation.
//!
//! Card: REFR-04.

pub mod bundle;
pub mod codeable_concept;
pub mod coding;
pub mod encounter;
pub mod patient;
pub mod reference;
pub mod servicerequest;

#[cfg(test)]
mod tests;

pub use bundle::{Bundle, BundleEntry};
pub use codeable_concept::CodeableConcept;
pub use coding::Coding;
pub use encounter::Encounter;
pub use patient::Patient;
pub use reference::Reference;
pub use servicerequest::ServiceRequest;
