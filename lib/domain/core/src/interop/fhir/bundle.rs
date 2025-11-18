use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::ServiceRequest;

/// Bundle entry that stores passthrough JSON resources.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleEntry {
    #[serde(default)]
    pub full_url: Option<String>,
    #[serde(default)]
    pub resource: Option<Value>,
}

/// Minimal Bundle representation containing arbitrary entries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bundle {
    #[serde(rename = "resourceType", default = "bundle_resource_type")]
    pub resource_type: String,
    #[serde(rename = "type")]
    pub bundle_type: Option<String>,
    #[serde(default)]
    pub entry: Vec<BundleEntry>,
}

fn bundle_resource_type() -> String {
    "Bundle".to_string()
}

impl Bundle {
    /// Iterate over ServiceRequest resources within the bundle.
    pub fn iter_servicerequests(
        &self,
    ) -> impl Iterator<Item = Result<ServiceRequest, serde_json::Error>> + '_ {
        self.entry.iter().filter_map(|entry| {
            entry.resource.as_ref().and_then(|resource| {
                let resource_type = resource
                    .get("resourceType")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                if resource_type == "ServiceRequest" {
                    Some(serde_json::from_value(resource.clone()))
                } else {
                    None
                }
            })
        })
    }
}
