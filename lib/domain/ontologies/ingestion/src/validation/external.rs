use dfps_core::fhir::Bundle;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{RequirementRef, ValidationIssue, ValidationSeverity};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationOutcomeIssue {
    pub severity: Option<String>,
    pub code: Option<String>,
    pub diagnostics: Option<String>,
    pub expression: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OperationOutcome {
    #[serde(default, alias = "issue")]
    pub issues: Vec<OperationOutcomeIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExternalValidationReport {
    pub operation_outcome: Option<OperationOutcome>,
    pub issues: Vec<ValidationIssue>,
}

impl ExternalValidationReport {
    pub fn from_operation_outcome(outcome: Option<OperationOutcome>) -> Self {
        let mut issues = Vec::new();
        if let Some(out) = &outcome {
            for issue in &out.issues {
                let severity = match issue
                    .severity
                    .as_deref()
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .as_str()
                {
                    "fatal" | "error" => ValidationSeverity::Error,
                    "warning" => ValidationSeverity::Warning,
                    _ => ValidationSeverity::Info,
                };
                let message = issue
                    .diagnostics
                    .clone()
                    .unwrap_or_else(|| "External validation reported an issue".to_string());
                let id = format!(
                    "VAL_EXTERNAL_{}",
                    issue
                        .code
                        .as_deref()
                        .unwrap_or("UNKNOWN")
                        .to_ascii_uppercase()
                );
                let mut msg = message;
                if let Some(exprs) = &issue.expression
                    && !exprs.is_empty()
                {
                    msg.push_str(&format!(" (expression: {})", exprs.join(", ")));
                }
                issues.push(ValidationIssue::new(
                    id,
                    severity,
                    msg,
                    RequirementRef::RExternal,
                ));
            }
        }
        Self {
            operation_outcome: outcome,
            issues,
        }
    }
}

#[derive(Debug, Error)]
pub enum ExternalValidationError {
    #[error("external validator unavailable: {0}")]
    Unavailable(String),
    #[error("external validator failed: {0}")]
    Failed(String),
    #[error("serialize bundle: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("parse operation outcome: {0}")]
    Parse(String),
}

pub trait ExternalValidator {
    fn validate_bundle(
        &self,
        bundle: &Bundle,
        profile_url: Option<&str>,
    ) -> Result<ExternalValidationReport, ExternalValidationError>;
}

/// No-op validator for test/default contexts when no external service is configured.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopExternalValidator;

impl ExternalValidator for NoopExternalValidator {
    fn validate_bundle(
        &self,
        _bundle: &Bundle,
        _profile_url: Option<&str>,
    ) -> Result<ExternalValidationReport, ExternalValidationError> {
        Ok(ExternalValidationReport::default())
    }
}
