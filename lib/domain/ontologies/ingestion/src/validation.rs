//! Validation primitives for FHIR ServiceRequest ingestion requirements.
//!
//! Each [`RequirementRef`] corresponds to an ID defined in
//! `docs/system-design/clinical/fhir/requirements/ingestion-requirements.md`.
//! When the `profile_validation` feature is enabled, profile cardinalities from
//! `crate::profiles` are applied alongside the hand-written checks to keep
//! ingestion aligned with embedded StructureDefinitions.

use std::collections::{HashMap, HashSet};

#[cfg(feature = "profile_validation")]
use crate::profiles::{self, ElementDefinition as ProfileElement, FhirProfile};
use dfps_core::fhir;
use serde::{Deserialize, Serialize};

use crate::reference::reference_id_from_str;
use crate::validation::external::{ExternalValidationError, ExternalValidator};

pub mod external;

/// Requirement identifiers mirrored from the ingestion requirements doc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementRef {
    /// Requirement ensuring every ServiceRequest references a Patient.
    RSubject,
    /// Requirement covering acceptable/normalizable status values.
    RStatus,
    /// Requirement ensuring provenance/trace identifiers are present.
    RTrace,
    /// Requirement representing external validator findings.
    RExternal,
}

impl RequirementRef {
    /// Return the canonical string code used in documentation.
    pub fn as_code(&self) -> &'static str {
        match self {
            RequirementRef::RSubject => "R_Subject",
            RequirementRef::RStatus => "R_Status",
            RequirementRef::RTrace => "R_Trace",
            RequirementRef::RExternal => "R_External",
        }
    }
}

/// Severity of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

/// Describes a requirement-linked validation issue discovered during ingestion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Stable issue identifier (e.g., `VAL_SR_SUBJECT_MISSING`).
    pub id: String,
    pub severity: ValidationSeverity,
    pub message: String,
    pub requirement: RequirementRef,
}

impl ValidationIssue {
    /// Convenience constructor for building a requirement-linked issue.
    pub fn new(
        id: impl Into<String>,
        severity: ValidationSeverity,
        message: impl Into<String>,
        requirement: RequirementRef,
    ) -> Self {
        Self {
            id: id.into(),
            severity,
            message: message.into(),
            requirement,
        }
    }

    /// Return the canonical requirement code (e.g., `R_Subject`).
    pub fn requirement_ref(&self) -> &'static str {
        self.requirement.as_code()
    }
}

/// Map RequirementRefs to specific profile element definitions for traceability.
#[cfg(feature = "profile_validation")]
pub fn profile_requirement_links(profile: &FhirProfile) -> Vec<(RequirementRef, ProfileElement)> {
    [
        (RequirementRef::RSubject, "ServiceRequest.subject"),
        (RequirementRef::RStatus, "ServiceRequest.status"),
        (RequirementRef::RTrace, "ServiceRequest.id"),
    ]
    .iter()
    .filter_map(|(req, path)| profile.element_by_path(path).cloned().map(|el| (*req, el)))
    .collect()
}

/// Profile-driven checks for ServiceRequest cardinalities and required elements.
#[cfg(feature = "profile_validation")]
pub fn validate_sr_profile(
    sr: &fhir::ServiceRequest,
    profile: &FhirProfile,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    let requirement_lookup = build_requirement_lookup(profile);

    for element in &profile.elements {
        if !element.path.starts_with("ServiceRequest.") {
            continue;
        }
        let min = element.min.unwrap_or(0);
        if min > 0 && !sr_field_present(sr, &element.path) {
            let requirement = requirement_lookup
                .get(element.path.as_str())
                .copied()
                .unwrap_or(RequirementRef::RTrace);
            let id = profile_issue_id(&element.path, "MISSING");
            let message = format!(
                "{} required by profile {} is missing.",
                element.path, profile.meta.url
            );
            issues.push(ValidationIssue::new(
                id,
                ValidationSeverity::Error,
                message,
                requirement,
            ));
        }
        if let Some(max) = element.max.as_deref() {
            if max == "0" && sr_field_present(sr, &element.path) {
                let requirement = requirement_lookup
                    .get(element.path.as_str())
                    .copied()
                    .unwrap_or(RequirementRef::RTrace);
                let id = profile_issue_id(&element.path, "NOT_ALLOWED");
                let message = format!(
                    "{} is not permitted by profile {}.",
                    element.path, profile.meta.url
                );
                issues.push(ValidationIssue::new(
                    id,
                    ValidationSeverity::Error,
                    message,
                    requirement,
                ));
            }
        }
    }

    issues
}

#[cfg(feature = "profile_validation")]
fn build_requirement_lookup(profile: &FhirProfile) -> HashMap<String, RequirementRef> {
    profile_requirement_links(profile)
        .into_iter()
        .map(|(req, el)| (el.path.clone(), req))
        .collect()
}

#[cfg(feature = "profile_validation")]
fn sr_field_present(sr: &fhir::ServiceRequest, path: &str) -> bool {
    match path {
        "ServiceRequest.id" => sr.id.as_deref().map(|v| !v.is_empty()).unwrap_or(false),
        "ServiceRequest.status" => sr.status.as_deref().map(|v| !v.is_empty()).unwrap_or(false),
        "ServiceRequest.intent" => sr.intent.as_deref().map(|v| !v.is_empty()).unwrap_or(false),
        "ServiceRequest.subject" => sr
            .subject
            .as_ref()
            .and_then(|r| r.reference.as_deref())
            .map(|v| !v.is_empty())
            .unwrap_or(false),
        "ServiceRequest.encounter" => sr
            .encounter
            .as_ref()
            .and_then(|r| r.reference.as_deref())
            .map(|v| !v.is_empty())
            .unwrap_or(false),
        _ => false,
    }
}

#[cfg(feature = "profile_validation")]
fn profile_issue_id(path: &str, suffix: &str) -> String {
    let key = path
        .trim_start_matches("ServiceRequest.")
        .replace('.', "_")
        .to_ascii_uppercase();
    format!("VAL_SR_PROFILE_{}_{}", key, suffix)
}

/// Validate a FHIR ServiceRequest against ingestion requirements.
pub fn validate_sr(sr: &fhir::ServiceRequest) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    validate_subject(sr, &mut issues);
    validate_status(sr, &mut issues);
    validate_traceability(sr, &mut issues);

    issues
}

/// Aggregated validation mode for bundle ingestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationMode {
    Strict,
    Lenient,
    ExternalPreferred,
    ExternalStrict,
}

impl Default for ValidationMode {
    fn default() -> Self {
        ValidationMode::Lenient
    }
}

/// Aggregated report returned by `validate_bundle`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReport {
    pub issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    pub fn new(issues: Vec<ValidationIssue>) -> Self {
        Self { issues }
    }

    pub fn has_errors(&self) -> bool {
        self.issues
            .iter()
            .any(|issue| issue.severity == ValidationSeverity::Error)
    }
}

/// Context for optional external validation (validator + profile URL).
#[derive(Clone, Copy, Default)]
pub struct ExternalValidationContext<'a> {
    pub validator: Option<&'a dyn ExternalValidator>,
    pub profile_url: Option<&'a str>,
}

impl std::fmt::Debug for ExternalValidationContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalValidationContext")
            .field("validator", &self.validator.is_some())
            .field("profile_url", &self.profile_url)
            .finish()
    }
}

/// Output wrapper for functions that combine ingestion + validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Validated<T> {
    pub value: T,
    pub report: ValidationReport,
}

impl<T> Validated<T> {
    pub fn new(value: T, report: ValidationReport) -> Self {
        Self { value, report }
    }
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
fn load_service_request_profile(profile_url: Option<&str>) -> Option<FhirProfile> {
    let url = profile_url.unwrap_or(profiles::SERVICE_REQUEST_PROFILE_URL);
    profiles::load_profile(url)
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
                    issues.extend(validate_sr_profile(&sr, profile));
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

    #[test]
    fn profile_requirement_links_return_expected_paths() {
        let profile =
            crate::profiles::load_profile(crate::profiles::SERVICE_REQUEST_PROFILE_URL).unwrap();
        let mappings = profile_requirement_links(&profile);
        assert!(mappings.iter().any(|(req, el)| {
            *req == RequirementRef::RSubject && el.path == "ServiceRequest.subject"
        }));
        assert!(
            mappings.iter().any(|(req, el)| {
                *req == RequirementRef::RTrace && el.path == "ServiceRequest.id"
            })
        );
    }

    #[test]
    fn validate_sr_profile_flags_missing_intent() {
        let profile =
            crate::profiles::load_profile(crate::profiles::SERVICE_REQUEST_PROFILE_URL).unwrap();
        let sr = fhir::ServiceRequest {
            resource_type: "ServiceRequest".into(),
            id: Some("SR-123".into()),
            status: Some("active".into()),
            intent: None,
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

        let issues = validate_sr_profile(&sr, &profile);
        assert!(
            issues
                .iter()
                .any(|issue| issue.id == "VAL_SR_PROFILE_INTENT_MISSING"),
            "missing intent should surface a profile issue"
        );
        assert_eq!(
            issues
                .iter()
                .filter(|i| i.requirement == RequirementRef::RTrace)
                .count(),
            1
        );
    }

    #[test]
    fn bundle_validation_aggregates_service_request_issues() {
        let bundle = fhir::Bundle {
            resource_type: "Bundle".into(),
            bundle_type: Some("collection".into()),
            entry: vec![fhir::BundleEntry {
                full_url: None,
                resource: Some(serde_json::json!({
                    "resourceType": "ServiceRequest",
                    "id": "SR-2",
                    "status": "unknown",
                    "intent": "order",
                    "subject": { "reference": "Observation/123" },
                })),
            }],
        };

        let report = validate_bundle(&bundle);
        assert!(report.has_errors());
        assert_eq!(report.issues.len(), 3);
    }

    #[test]
    fn bundle_validation_flags_missing_patient_resource() {
        let bundle = fhir::Bundle {
            resource_type: "Bundle".into(),
            bundle_type: Some("collection".into()),
            entry: vec![fhir::BundleEntry {
                full_url: None,
                resource: Some(serde_json::json!({
                    "resourceType": "ServiceRequest",
                    "id": "SR-3",
                    "status": "active",
                    "intent": "order",
                    "subject": { "reference": "Patient/P-MISSING" }
                })),
            }],
        };

        let report = validate_bundle(&bundle);
        assert!(report.has_errors());
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.id == "VAL_SR_SUBJECT_PATIENT_NOT_FOUND")
        );
    }

    #[test]
    fn bundle_validation_flags_missing_encounter_resource() {
        let bundle = fhir::Bundle {
            resource_type: "Bundle".into(),
            bundle_type: Some("collection".into()),
            entry: vec![
                fhir::BundleEntry {
                    full_url: None,
                    resource: Some(serde_json::json!({
                        "resourceType": "Patient",
                        "id": "PAT-1"
                    })),
                },
                fhir::BundleEntry {
                    full_url: None,
                    resource: Some(serde_json::json!({
                        "resourceType": "ServiceRequest",
                        "id": "SR-4",
                        "status": "active",
                        "intent": "order",
                        "subject": { "reference": "Patient/PAT-1" },
                        "encounter": { "reference": "Encounter/ENC-MISSING" }
                    })),
                },
            ],
        };

        let report = validate_bundle(&bundle);
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.id == "VAL_SR_ENCOUNTER_NOT_FOUND")
        );
    }
}
