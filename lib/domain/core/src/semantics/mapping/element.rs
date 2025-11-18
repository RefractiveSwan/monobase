use serde::{Deserialize, Serialize};

/// Atomic code extracted from staging and ready for mapping.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeElement {
    pub id: String,
    pub system: Option<String>,
    pub code: Option<String>,
    pub display: Option<String>,
}

impl CodeElement {
    pub fn new(
        id: impl Into<String>,
        system: Option<String>,
        code: Option<String>,
        display: Option<String>,
    ) -> Self {
        Self {
            id: id.into(),
            system,
            code,
            display,
        }
    }

    /// Construct a stable identifier for a code element, falling back to
    /// `"unknown-system"` and `"unknown-code"` when the FHIR coding is missing.
    pub fn id_for(
        sr_id: &str,
        system: Option<&str>,
        code: Option<&str>,
        display: Option<&str>,
    ) -> String {
        format!(
            "{}::{}::{}",
            sr_id,
            system.unwrap_or("unknown-system"),
            code.or(display).unwrap_or("unknown-code")
        )
    }
}
