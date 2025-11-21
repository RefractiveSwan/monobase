//! Ingestion utilities for converting raw FHIR payloads into staging rows and
//! core domain aggregates, plus optional profile-aware validation.
//!
//! Flow (REFR-05):
//! - `interop::fhir::Bundle` → `staging::{StgServiceRequestFlat, StgSrCodeExploded}`
//!   via `sr_to_staging` and `bundle_to_staging[_with_validation]`
//! - `interop::fhir::ServiceRequest` → `clinical::order::ServiceRequest`
//!   via `sr_to_domain` and `bundle_to_domain[_with_validation]`
//! - Optional profile checks live in `profiles` (embedded StructureDefinitions);
//!   external validation is injected via the `ExternalValidator` port.
//!
//! See:
//! - docs/system-design/clinical/fhir/index.md
//! - docs/system-design/clinical/fhir/requirements/ingestion-requirements.md
//! - docs/runbook/fhir-profiles-quickstart.md
//! - docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md#refr-05--domain-ingestion--fhir-profiles

#[cfg(feature = "profile_validation")]
pub mod profiles;
mod reference;
pub mod transforms;
pub mod validation;

pub use reference::{reference_id, reference_id_from_str};
pub use transforms::{
    bundle::*, errors::IngestionError, service_request::sr_to_domain,
    service_request::sr_to_staging,
};

pub use refractive_swan_validation_port::{
    ExternalValidationError, ExternalValidator, OperationOutcome, OperationOutcomeIssue,
};
pub use validation::external::ExternalValidationReport;
#[cfg(feature = "profile_validation")]
pub use validation::profile::{profile_requirement_links, validate_sr_profile};
pub use validation::types::{
    ExternalValidationContext, RequirementRef, Validated, ValidationIssue, ValidationMode,
    ValidationReport, ValidationSeverity,
};
pub use validation::{
    validate_bundle, validate_bundle_with_external, validate_bundle_with_external_profile,
    validate_sr,
};
