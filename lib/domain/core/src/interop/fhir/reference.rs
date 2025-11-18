use serde::{Deserialize, Serialize};

/// Simple `Reference` type: `"ResourceType/id"` string plus optional label.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    pub reference: Option<String>,
    pub display: Option<String>,
}
