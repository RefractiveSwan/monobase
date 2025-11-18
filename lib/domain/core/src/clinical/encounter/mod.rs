//! Encounter entity connecting patients to care contexts.
//!
//! Mirrors the Encounter nodes described in
//! `docs/system-design/fhir/models/class-model.md` and the PET/CT journey in
//! `docs/system-design/fhir/experience/user-journey-pet-ct.md`.
//!
//! Card: REFR-04.

pub mod ops;
pub mod types;

#[cfg(test)]
mod tests;

pub use types::Encounter;
