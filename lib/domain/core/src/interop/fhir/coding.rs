use serde::{Deserialize, Serialize};

/// Code representation following FHIR `Coding`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coding {
    pub system: Option<String>,
    pub code: Option<String>,
    pub display: Option<String>,
}
