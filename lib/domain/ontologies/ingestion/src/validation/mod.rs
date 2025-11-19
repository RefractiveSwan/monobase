//! Validation primitives for FHIR ServiceRequest ingestion requirements.
//!
//! Each [`RequirementRef`] corresponds to an ID defined in
//! `docs/system-design/clinical/fhir/requirements/ingestion-requirements.md`.
//! When the `profile_validation` feature is enabled, profile cardinalities from
//! `crate::profiles` are applied alongside the hand-written checks to keep
//! ingestion aligned with embedded StructureDefinitions.

use std::collections::HashSet;

use dfps_core::fhir;

use crate::reference::reference_id_from_str;
use crate::validation::external::ExternalValidationError;

pub mod external;
pub mod types;

#[cfg(feature = "profile_validation")]
pub mod profile;

pub use types::{
    ExternalValidationContext, RequirementRef, Validated, ValidationIssue, ValidationMode,
    ValidationReport, ValidationSeverity,
};

/// Validate a FHIR ServiceRequest against ingestion requirements.
pub fn validate_sr(sr: &fhir::ServiceRequest) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    validate_subject(sr, &mut issues);
    validate_status(sr, &mut issues);
    validate_traceability(sr, &mut issues);

    issues
}

/// Validate an entire FHIR Bundle by walking ServiceRequests and referenced resources.
pub fn validate_bundle(bundle: &fhir::Bundle) -> ValidationReport {
    validate_bundle_with_external_profile(
        bundle,
        ValidationMode::Lenient,
        ExternalValidationContext::default(),
    )
}

#[cfg(feature = "profile_validation")]
fn load_service_request_profile(profile_url: Option<&str>) -> Option<crate::profiles::FhirProfile> {
    let url = profile_url.unwrap_or(crate::profiles::SERVICE_REQUEST_PROFILE_URL);
    crate::profiles::load_profile(url)
}

/// Validate a bundle and optionally merge external validator feedback.
pub fn validate_bundle_with_external(
    bundle: &fhir::Bundle,
    mode: ValidationMode,
    ctx: ExternalValidationContext<'_>,
) -> ValidationReport {
    validate_bundle_with_external_profile(bundle, mode, ctx)
}

/// Validate a bundle and optionally merge external validator feedback with an explicit profile URL.
pub fn validate_bundle_with_external_profile(
    bundle: &fhir::Bundle,
    mode: ValidationMode,
    ctx: ExternalValidationContext<'_>,
) -> ValidationReport {
    #[cfg(feature = "profile_validation")]
    let profile_url = ctx.profile_url;
    #[cfg(feature = "profile_validation")]
    let sr_profile = load_service_request_profile(profile_url);

    let patient_ids = collect_resource_ids(bundle, "Patient");
    let encounter_ids = collect_resource_ids(bundle, "Encounter");
    let mut issues = Vec::new();

    for entry in bundle.iter_servicerequests() {
        match entry {
            Ok(sr) => {
                issues.extend(validate_sr(&sr));
                #[cfg(feature = "profile_validation")]
                if let Some(profile) = sr_profile.as_ref() {
                    issues.extend(crate::validation::profile::validate_sr_profile(
                        &sr, profile,
                    ));
                }
                validate_bundle_relationships(&sr, &patient_ids, &encounter_ids, &mut issues);
            }
            Err(err) => {
                issues.push(ValidationIssue::new(
                    "VAL_BUNDLE_SR_DECODE",
                    ValidationSeverity::Error,
                    format!("Failed to decode ServiceRequest: {err}"),
                    RequirementRef::RTrace,
                ));
            }
        }
    }

    let mut report = ValidationReport::new(issues);

    if matches!(
        mode,
        ValidationMode::ExternalPreferred | ValidationMode::ExternalStrict
    ) {
        let external_result = match ctx.validator {
            Some(validator) => validator.validate_bundle(bundle, ctx.profile_url),
            None => Err(ExternalValidationError::Unavailable(
                "no external validator provided".into(),
            )),
        };
        report = merge_external_report(mode, report, external_result);
    }

    report
}

/// Merge external validation output into an existing report.
pub(crate) fn merge_external_report(
    mode: ValidationMode,
    mut report: ValidationReport,
    external: Result<
        crate::validation::external::ExternalValidationReport,
        crate::validation::external::ExternalValidationError,
    >,
) -> ValidationReport {
    match external {
        Ok(ext) => {
            if ext.issues.is_empty() && matches!(mode, ValidationMode::ExternalStrict) {
                report.issues.push(ValidationIssue::new(
                    "VAL_EXTERNAL_EMPTY",
                    ValidationSeverity::Error,
                    "External validation returned no issues or diagnostics in ExternalStrict mode.",
                    RequirementRef::RExternal,
                ));
            } else {
                report.issues.extend(ext.issues);
            }
        }
        Err(err) => {
            report.issues.push(ValidationIssue::new(
                "VAL_EXTERNAL_UNAVAILABLE",
                if matches!(mode, ValidationMode::ExternalStrict) {
                    ValidationSeverity::Error
                } else {
                    ValidationSeverity::Warning
                },
                format!("External validation unavailable: {err}"),
                RequirementRef::RExternal,
            ));
        }
    }
    report
}

fn validate_subject(sr: &fhir::ServiceRequest, issues: &mut Vec<ValidationIssue>) {
    match sr
        .subject
        .as_ref()
        .and_then(|reference| reference.reference.as_deref())
    {
        Some(reference) if is_patient_reference(reference) => {}
        Some(_) => issues.push(ValidationIssue::new(
            "VAL_SR_SUBJECT_INVALID",
            ValidationSeverity::Error,
            "ServiceRequest.subject must reference a Patient (Patient/<id>).",
            RequirementRef::RSubject,
        )),
        None => issues.push(ValidationIssue::new(
            "VAL_SR_SUBJECT_MISSING",
            ValidationSeverity::Error,
            "ServiceRequest.subject is required.",
            RequirementRef::RSubject,
        )),
    }
}

fn validate_status(sr: &fhir::ServiceRequest, issues: &mut Vec<ValidationIssue>) {
    match sr.status.as_deref() {
        Some(value) if is_known_status(value) => {}
        Some(_) => issues.push(ValidationIssue::new(
            "VAL_SR_STATUS_INVALID",
            ValidationSeverity::Error,
            "ServiceRequest.status must be a recognized value (draft, active, on-hold, completed, cancelled, revoked, entered-in-error).",
            RequirementRef::RStatus,
        )),
        None => issues.push(ValidationIssue::new(
            "VAL_SR_STATUS_MISSING",
            ValidationSeverity::Error,
            "ServiceRequest.status is required.",
            RequirementRef::RStatus,
        )),
    }
}

fn validate_traceability(sr: &fhir::ServiceRequest, issues: &mut Vec<ValidationIssue>) {
    if sr.id.as_deref().unwrap_or("").is_empty() {
        issues.push(ValidationIssue::new(
            "VAL_SR_TRACE_ID_MISSING",
            ValidationSeverity::Error,
            "ServiceRequest.id is required to trace staging rows back to the Bundle.",
            RequirementRef::RTrace,
        ))
    }
}

fn validate_bundle_relationships(
    sr: &fhir::ServiceRequest,
    patient_ids: &HashSet<String>,
    encounter_ids: &HashSet<String>,
    issues: &mut Vec<ValidationIssue>,
) {
    if let Some(reference) = sr.subject.as_ref().and_then(|r| r.reference.as_deref()) {
        if let Some(id) = reference_id_from_str(reference) {
            if !patient_ids.contains(id) {
                issues.push(ValidationIssue::new(
                    "VAL_SR_SUBJECT_PATIENT_NOT_FOUND",
                    ValidationSeverity::Error,
                    format!("ServiceRequest.subject references Patient/{id}, which is not present in the Bundle."),
                    RequirementRef::RSubject,
                ));
            }
        }
    }

    if let Some(reference) = sr.encounter.as_ref().and_then(|r| r.reference.as_deref()) {
        if let Some(id) = reference_id_from_str(reference) {
            if !encounter_ids.contains(id) {
                issues.push(ValidationIssue::new(
                    "VAL_SR_ENCOUNTER_NOT_FOUND",
                    ValidationSeverity::Warning,
                    format!(
                        "ServiceRequest.encounter references Encounter/{id}, which is not present in the Bundle."
                    ),
                    RequirementRef::RTrace,
                ));
            }
        }
    }
}

fn is_patient_reference(reference: &str) -> bool {
    reference.starts_with("Patient/")
        && reference
            .split('/')
            .nth(1)
            .map(|id| !id.is_empty())
            .unwrap_or(false)
}

fn is_known_status(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "draft"
            | "active"
            | "on-hold"
            | "on_hold"
            | "completed"
            | "cancelled"
            | "revoked"
            | "entered-in-error"
            | "entered_in_error"
    )
}

fn collect_resource_ids(bundle: &fhir::Bundle, resource_type: &str) -> HashSet<String> {
    bundle
        .entry
        .iter()
        .filter_map(|entry| entry.resource.as_ref())
        .filter_map(|resource| {
            let ty = resource.get("resourceType")?.as_str()?;
            if ty.eq_ignore_ascii_case(resource_type) {
                resource.get("id").and_then(|v| v.as_str())
            } else {
                None
            }
        })
        .map(|id| id.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::external::{
        ExternalValidationError, ExternalValidationReport, OperationOutcome, OperationOutcomeIssue,
    };
    use dfps_core::fhir;

    #[test]
    fn requirement_codes_match_docs() {
        assert_eq!(RequirementRef::RSubject.as_code(), "R_Subject");
        assert_eq!(RequirementRef::RStatus.as_code(), "R_Status");
        assert_eq!(RequirementRef::RTrace.as_code(), "R_Trace");
    }

    #[test]
    fn issue_construction_exposes_requirement_ref() {
        let issue = ValidationIssue::new(
            "VAL_SR_SUBJECT_MISSING",
            ValidationSeverity::Error,
            "ServiceRequest.subject must reference a Patient",
            RequirementRef::RSubject,
        );
        assert_eq!(issue.requirement_ref(), "R_Subject");
        assert_eq!(issue.severity, ValidationSeverity::Error);
    }

    #[test]
    fn validate_sr_flags_missing_subject_and_status() {
        let sr = fhir::ServiceRequest {
            resource_type: "ServiceRequest".into(),
            id: None,
            status: None,
            intent: Some("order".into()),
            subject: None,
            encounter: None,
            requester: None,
            supporting_info: vec![],
            code: None,
            category: vec![],
            description: None,
            authored_on: None,
        };

        let issues = validate_sr(&sr);
        assert!(
            issues
                .iter()
                .any(|issue| issue.id == "VAL_SR_SUBJECT_MISSING")
        );
        assert!(
            issues
                .iter()
                .any(|issue| issue.id == "VAL_SR_STATUS_MISSING")
        );
        assert!(
            issues
                .iter()
                .any(|issue| issue.id == "VAL_SR_TRACE_ID_MISSING")
        );
    }

    #[test]
    fn merge_external_adds_warning_on_failure() {
        let report = ValidationReport::new(vec![]);
        let merged = merge_external_report(
            ValidationMode::ExternalPreferred,
            report,
            Err(ExternalValidationError::Unavailable(
                "DFPS_FHIR_VALIDATOR_BASE_URL not set".into(),
            )),
        );
        assert_eq!(merged.issues.len(), 1);
        let issue = &merged.issues[0];
        assert_eq!(issue.requirement, RequirementRef::RExternal);
        assert_eq!(issue.severity, ValidationSeverity::Warning);
    }

    #[test]
    fn merge_external_includes_operation_outcome_issues() {
        let ext = ExternalValidationReport::from_operation_outcome(Some(OperationOutcome {
            issues: vec![OperationOutcomeIssue {
                severity: Some("error".into()),
                code: Some("invalid".into()),
                diagnostics: Some("missing subject".into()),
                expression: None,
            }],
        }));
        let merged = merge_external_report(
            ValidationMode::ExternalStrict,
            ValidationReport::new(vec![]),
            Ok(ext),
        );
        assert_eq!(merged.issues.len(), 1);
        assert_eq!(merged.issues[0].requirement, RequirementRef::RExternal);
        assert_eq!(merged.issues[0].severity, ValidationSeverity::Error);
        assert!(merged.has_errors());
    }

    #[test]
    fn validate_sr_accepts_valid_status_and_subject() {
        let sr = fhir::ServiceRequest {
            resource_type: "ServiceRequest".into(),
            id: Some("SR-1".into()),
            status: Some("active".into()),
            intent: Some("order".into()),
            subject: Some(fhir::Reference {
                reference: Some("Patient/P1".into()),
                display: None,
            }),
            encounter: None,
            requester: None,
            supporting_info: vec![],
            code: None,
            category: vec![],
            description: None,
            authored_on: None,
        };

        let issues = validate_sr(&sr);
        assert!(issues.is_empty());
    }
}
