//! External validation port for FHIR bundles.

use dfps_core::fhir::Bundle;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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

pub type ExternalValidationOutcome = Option<OperationOutcome>;

pub trait ExternalValidator: Send + Sync {
    fn validate_bundle(
        &self,
        bundle: &Bundle,
        profile_url: Option<&str>,
    ) -> Result<ExternalValidationOutcome, ExternalValidationError>;
}
