use std::time::Duration;

use dfps_core::fhir::Bundle;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{RequirementRef, ValidationIssue, ValidationSeverity};
use crate::validation::types::ExternalValidationContext;

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
                if let Some(exprs) = &issue.expression {
                    if !exprs.is_empty() {
                        msg.push_str(&format!(" (expression: {})", exprs.join(", ")));
                    }
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

#[derive(Debug, Clone)]
pub struct ExternalValidatorConfig {
    pub base_url: String,
    pub timeout: Duration,
    pub default_profile: Option<String>,
}

impl ExternalValidatorConfig {
    pub fn from_env() -> Result<Self, ExternalValidationError> {
        let base_url = std::env::var("DFPS_FHIR_VALIDATOR_BASE_URL").map_err(|_| {
            ExternalValidationError::Unavailable("DFPS_FHIR_VALIDATOR_BASE_URL not set".into())
        })?;
        let timeout_secs = std::env::var("DFPS_FHIR_VALIDATOR_TIMEOUT_SECS")
            .ok()
            .and_then(|raw| raw.parse::<u64>().ok())
            .unwrap_or(10);
        let default_profile = std::env::var("DFPS_FHIR_VALIDATOR_PROFILE").ok();
        Ok(Self {
            base_url,
            timeout: Duration::from_secs(timeout_secs),
            default_profile,
        })
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

pub fn validate_bundle_external(
    bundle: &Bundle,
    ctx: &ExternalValidationContext<'_>,
) -> Result<ExternalValidationReport, ExternalValidationError> {
    if let Ok(mode) = std::env::var("DFPS_FHIR_VALIDATOR_MOCK") {
        return Ok(match mode.as_str() {
            "error" => ExternalValidationReport {
                operation_outcome: None,
                issues: vec![ValidationIssue::new(
                    "VAL_EXTERNAL_MOCK",
                    ValidationSeverity::Error,
                    "Mock external validator reported an error",
                    RequirementRef::RExternal,
                )],
            },
            "ok" => ExternalValidationReport::default(),
            _ => ExternalValidationReport::default(),
        });
    }
    let cfg = ExternalValidatorConfig::from_env()?;
    let client = Client::builder()
        .timeout(cfg.timeout)
        .build()
        .map_err(|err| ExternalValidationError::Failed(err.to_string()))?;

    let mut url = cfg.base_url.trim_end_matches('/').to_string();
    if !url.ends_with("/$validate") {
        url.push_str("/$validate");
    }

    let mut request = client.post(url).json(bundle);
    if let Some(profile) = ctx.profile_url.or_else(|| cfg.default_profile.as_deref()) {
        request = request.query(&[("profile", profile)]);
    }

    let response = request
        .send()
        .map_err(|err| ExternalValidationError::Failed(err.to_string()))?;
    let outcome: OperationOutcome = response
        .json()
        .map_err(|err| ExternalValidationError::Parse(err.to_string()))?;
    Ok(ExternalValidationReport::from_operation_outcome(Some(
        outcome,
    )))
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
