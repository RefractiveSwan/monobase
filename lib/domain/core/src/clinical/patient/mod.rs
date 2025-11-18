//! Patient aggregate mirrored in the system design diagrams.
//!
//! The structure matches the Patient nodes in
//! `docs/system-design/fhir/models/class-model.md` and is referenced by the
//! ServiceRequest lifecycle diagrams in
//! `docs/system-design/fhir/behavior/sequence-servicerequest.md`.
//!
//! Card: REFR-04.

pub mod ops;
pub mod types;

#[cfg(test)]
mod tests;

pub use types::Patient;
