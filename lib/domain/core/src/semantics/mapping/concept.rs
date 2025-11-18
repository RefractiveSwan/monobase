use serde::{Deserialize, Serialize};

/// NCIt concept metadata required for analytics + downstream linking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NCItConcept {
    pub ncit_id: String,
    pub preferred_name: String,
    #[serde(default)]
    pub synonyms: Vec<String>,
}

/// Dimensional NCIt concept view for warehouse/analytics consumption.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DimNCITConcept {
    pub ncit_id: String,
    pub preferred_name: String,
    pub semantic_group: String,
}
