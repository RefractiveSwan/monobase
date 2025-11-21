use refractive_swan_validation_port::ExternalValidator;
use serde::{Deserialize, Serialize};

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

/// Aggregated validation mode for bundle ingestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ValidationMode {
    Strict,
    #[default]
    Lenient,
    ExternalPreferred,
    ExternalStrict,
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
