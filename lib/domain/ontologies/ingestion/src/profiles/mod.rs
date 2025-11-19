//! Embedded FHIR StructureDefinition slices used for profile-aware validation.
//!
//! Kept lightweight to avoid pulling in a full FHIR engine. Profiles stay
//! colocated with ingestion so callers enabling `profile_validation` can rely
//! on bundled snapshots without any IO or HTTP round-trips.
//!
//! See:
//! - docs/system-design/clinical/fhir/requirements/ingestion-requirements.md
//! - docs/system-design/clinical/fhir/overview.md
//! - docs/runbook/fhir-profiles-quickstart.md

pub mod embedded;
pub mod model;

#[cfg(test)]
mod tests;

pub use embedded::{
    ENCOUNTER_PROFILE_URL, PATIENT_PROFILE_URL, SERVICE_REQUEST_PROFILE_URL, known_profile_urls,
    load_profile,
};
pub use model::{
    ElementBinding, ElementDefinition, FhirProfile, ProfileError, ProfileMeta, RawElementBinding,
    RawElementDefinition, RawElementType, RawSnapshot, RawStructureDefinition,
};
