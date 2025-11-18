use serde::{Deserialize, Serialize};

/// Minimal FHIR Encounter resource.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Encounter {
    #[serde(rename = "resourceType", default = "encounter_resource_type")]
    pub resource_type: String,
    pub id: Option<String>,
}

fn encounter_resource_type() -> String {
    "Encounter".to_string()
}
