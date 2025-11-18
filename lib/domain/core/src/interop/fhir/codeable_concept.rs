use serde::{Deserialize, Serialize};

use super::Coding;

/// Text + list of codings per FHIR `CodeableConcept`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeableConcept {
    #[serde(default)]
    pub coding: Vec<Coding>,
    pub text: Option<String>,
}
