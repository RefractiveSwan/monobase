use serde::{Deserialize, Serialize};

use super::{CodeableConcept, Reference};

/// Minimal FHIR ServiceRequest subset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceRequest {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    pub id: Option<String>,

    pub status: Option<String>,
    pub intent: Option<String>,

    pub subject: Option<Reference>,
    pub encounter: Option<Reference>,
    pub requester: Option<Reference>,
    #[serde(default)]
    pub supporting_info: Vec<Reference>,

    pub code: Option<CodeableConcept>,
    #[serde(default)]
    pub category: Vec<CodeableConcept>,
    pub description: Option<String>,
    pub authored_on: Option<String>,
}
