//! Ingestion utilities for converting raw FHIR payloads into staging rows and
//! core domain aggregates.
//!
//! The helpers here are intentionally lightweight and align with the minimal
//! scope documented in `docs\kanban\feature\002-fhir-pipeline-mvp.md`.

mod reference;
mod transforms;
pub mod validation;

pub use reference::{reference_id, reference_id_from_str};
pub use transforms::{
    IngestionError, bundle_to_domain, bundle_to_domain_with_validation, bundle_to_staging,
    bundle_to_staging_with_validation, sr_to_domain, sr_to_staging,
};

pub use validation::{
    RequirementRef, Validated, ValidationIssue, ValidationMode, ValidationReport,
    ValidationSeverity,
    external::{
        ExternalValidationError, ExternalValidationReport, ExternalValidatorConfig,
        OperationOutcome, OperationOutcomeIssue, validate_bundle_external,
    },
    validate_bundle, validate_bundle_with_external, validate_bundle_with_external_profile,
    validate_sr,
};
#[cfg(feature = "profile_validation")]
pub use validation::{profile_requirement_links, validate_sr_profile};
