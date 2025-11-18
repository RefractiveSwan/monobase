use serde::{Deserialize, Serialize};

/// Versions of the vocabularies involved in mapping decisions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MappingSourceVersion {
    pub ncit: String,
    pub umls: String,
}

impl MappingSourceVersion {
    pub fn new(ncit: impl Into<String>, umls: impl Into<String>) -> Self {
        Self {
            ncit: ncit.into(),
            umls: umls.into(),
        }
    }
}
