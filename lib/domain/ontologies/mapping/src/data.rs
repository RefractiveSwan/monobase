use std::collections::HashMap;

use dfps_core::mapping::{DimNCITConcept, NCItConcept};
use serde::Deserialize;

pub const NCIT_DATA_VERSION: &str = "mock-ncit-2024-01";
pub const UMLS_DATA_VERSION: &str = "mock-umls-2024-01";

#[derive(Debug, Deserialize)]
struct RawConcept {
    ncit_id: String,
    preferred_name: String,
    #[serde(default)]
    synonyms: Vec<String>,
    semantic_group: String,
}

pub fn load_ncit_concepts() -> Vec<(NCItConcept, DimNCITConcept)> {
    static RAW: &str = include_str!("../data/ncit_concepts.json");
    serde_json::from_str::<Vec<RawConcept>>(RAW)
        .expect("ncit_concepts.json should parse")
        .into_iter()
        .map(|raw| {
            (
                NCItConcept {
                    ncit_id: raw.ncit_id.clone(),
                    preferred_name: raw.preferred_name.clone(),
                    synonyms: raw.synonyms.clone(),
                },
                DimNCITConcept {
                    ncit_id: raw.ncit_id,
                    preferred_name: raw.preferred_name,
                    semantic_group: raw.semantic_group,
                },
            )
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UmlsXref {
    pub system: String,
    pub code: String,
    pub cui: String,
    pub ncit_id: String,
}

pub fn load_umls_xrefs() -> HashMap<(String, String), UmlsXref> {
    static RAW: &str = include_str!("../data/umls_xrefs.json");
    serde_json::from_str::<Vec<UmlsXref>>(RAW)
        .expect("umls_xrefs.json should parse")
        .into_iter()
        .map(|xref| ((xref.system.clone(), xref.code.clone()), xref))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_mock_concept_data() {
        let concepts = load_ncit_concepts();
        assert!(!concepts.is_empty());
        assert!(
            concepts
                .iter()
                .any(|(c, _)| c.ncit_id == "NCIT:C19951" && c.synonyms.len() >= 1)
        );
    }

    #[test]
    fn loads_mock_xrefs() {
        let xrefs = load_umls_xrefs();
        let key = (
            "http://www.ama-assn.org/go/cpt".to_string(),
            "78815".to_string(),
        );
        assert!(xrefs.contains_key(&key));
    }
}
