use serde::{Deserialize, Serialize};

/// Minimal FHIR Patient resource for linking IDs inside Bundles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patient {
    #[serde(rename = "resourceType", default = "patient_resource_type")]
    pub resource_type: String,
    pub id: Option<String>,
}

fn patient_resource_type() -> String {
    "Patient".to_string()
}
