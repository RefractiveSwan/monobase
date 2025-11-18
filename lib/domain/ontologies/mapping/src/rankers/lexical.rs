use dfps_core::mapping::{CodeElement, MappingCandidate};
#[cfg(feature = "obo-graph")]
use once_cell::sync::Lazy;

use crate::traits::CandidateRanker;

#[derive(Debug, Default)]
pub struct LexicalRanker;

#[cfg(feature = "obo-graph")]
const PET_PRIMARY_NCIT_ID: &str = "C19951";

#[cfg(feature = "obo-graph")]
fn contains_any(haystack: &str, needles: &[String]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

#[cfg(feature = "obo-graph")]
fn pet_synonyms() -> &'static Vec<String> {
    static SYNONYMS: Lazy<Vec<String>> =
        Lazy::new(|| dfps_terminology::synonym_set(PET_PRIMARY_NCIT_ID).unwrap_or_default());
    &SYNONYMS
}

#[cfg(feature = "obo-graph")]
fn pet_related_synonyms() -> &'static Vec<String> {
    use std::collections::BTreeSet;

    static RELATED: Lazy<Vec<String>> = Lazy::new(|| {
        let mut set = BTreeSet::new();
        for related in dfps_terminology::related_concepts(PET_PRIMARY_NCIT_ID, 2) {
            if let Some(syns) = dfps_terminology::synonym_set(&related) {
                for syn in syns {
                    set.insert(syn);
                }
            }
        }
        set.into_iter().collect()
    });
    &RELATED
}

impl CandidateRanker for LexicalRanker {
    fn rank(&self, code: &CodeElement) -> Vec<MappingCandidate> {
        let mut candidates = Vec::new();

        if let Some(display) = &code.display {
            let display_lower = display.to_ascii_lowercase();
            let mut direct_match = display_lower.contains("pet") || display_lower.contains("ct");
            #[cfg(feature = "obo-graph")]
            let mut related_match = false;

            #[cfg(feature = "obo-graph")]
            {
                if !direct_match && contains_any(&display_lower, pet_synonyms()) {
                    direct_match = true;
                }
                if !direct_match && contains_any(&display_lower, pet_related_synonyms()) {
                    related_match = true;
                }
            }

            if direct_match {
                candidates.push(MappingCandidate {
                    target_system: "NCIT".into(),
                    target_code: "C19951".into(),
                    cui: Some("C19951".into()),
                    score: 0.97,
                });
            }

            #[cfg(feature = "obo-graph")]
            if !direct_match && related_match {
                candidates.push(MappingCandidate {
                    target_system: "NCIT".into(),
                    target_code: "C19951".into(),
                    cui: Some("C19951".into()),
                    score: 0.95,
                });
            }

            if display_lower.contains("loinc") {
                candidates.push(MappingCandidate {
                    target_system: "LOINC".into(),
                    target_code: code.code.clone().unwrap_or_default(),
                    cui: None,
                    score: 0.6,
                });
            }
        }

        if candidates.is_empty() {
            candidates.push(MappingCandidate {
                target_system: code.system.clone().unwrap_or_else(|| "local-system".into()),
                target_code: code.code.clone().unwrap_or_else(|| "local-code".into()),
                cui: None,
                score: 0.4,
            });
        }

        candidates
    }
}
