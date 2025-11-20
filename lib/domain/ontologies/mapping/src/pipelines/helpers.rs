use std::collections::HashSet;

use dfps_core::mapping::{
    CodeElement, DimNCITConcept, MappingResult, MappingState, MappingStrategy,
};

use crate::config::MappingConfig;
use crate::data::load_ncit_concepts;
use crate::rankers::normalize_ncit_code;

pub fn build_result_with_score(
    code: &CodeElement,
    cui: Option<String>,
    ncit_id: Option<String>,
    score: f32,
    strategy: MappingStrategy,
    reason: Option<String>,
    config: &MappingConfig,
) -> MappingResult {
    let thresholds = config.thresholds;
    let state = classify(score, &thresholds);
    let mut final_ncit = ncit_id;
    let final_reason = if state == MappingState::NoMatch {
        final_ncit = None;
        Some(reason.unwrap_or_else(|| "score_below_threshold".into()))
    } else {
        reason
    };
    MappingResult {
        code_element_id: code.id.clone(),
        cui,
        ncit_id: final_ncit,
        score,
        strategy,
        state,
        thresholds,
        source_version: config.source_version.clone(),
        reason: final_reason,
        license_tier: None,
        source_kind: None,
    }
}

pub fn classify(score: f32, thresholds: &dfps_core::mapping::MappingThresholds) -> MappingState {
    if score >= thresholds.auto_map_min {
        MappingState::AutoMapped
    } else if score >= thresholds.needs_review_min {
        MappingState::NeedsReview
    } else {
        MappingState::NoMatch
    }
}

pub fn attach_license_metadata(
    result: &mut MappingResult,
    enriched: &dfps_terminology::EnrichedCode,
) {
    if let Some(label) = enriched.license_label() {
        result.license_tier = Some(label.to_string());
    }
    if let Some(label) = enriched.source_label() {
        result.source_kind = Some(label.to_string());
    }
}

pub fn dim_concepts() -> Vec<DimNCITConcept> {
    normalize_concepts_for_dim(lookup_dim_concepts())
}

fn lookup_dim_concepts() -> Vec<DimNCITConcept> {
    let concepts = load_ncit_concepts();
    let mut seen = HashSet::new();
    let mut dim_concepts = Vec::new();
    for (_, dim) in concepts {
        if seen.insert(dim.ncit_id.clone()) {
            dim_concepts.push(dim);
        }
    }
    dim_concepts
}

fn normalize_code_for_dim(concept: &DimNCITConcept) -> DimNCITConcept {
    DimNCITConcept {
        ncit_id: normalize_ncit_code(&concept.ncit_id),
        preferred_name: concept.preferred_name.clone(),
        semantic_group: concept.semantic_group.clone(),
    }
}

fn normalize_concepts_for_dim(mut concepts: Vec<DimNCITConcept>) -> Vec<DimNCITConcept> {
    concepts
        .iter_mut()
        .for_each(|concept| *concept = normalize_code_for_dim(concept));
    concepts
}

pub fn map_with_dim_concepts<I>(
    codes: I,
    engine: crate::engine::MappingEngine<
        impl crate::traits::CandidateRanker,
        impl crate::traits::CandidateRanker,
    >,
) -> (Vec<MappingResult>, Vec<DimNCITConcept>)
where
    I: IntoIterator<Item = dfps_core::staging::StgSrCodeExploded>,
{
    let xrefs = crate::data::load_umls_xrefs();
    let (results, _) = crate::pipelines::staging::map_with_engine(
        codes,
        &engine,
        &xrefs,
        None as Option<&dyn dfps_terminology::TerminologyClient>,
        &MappingConfig::default(),
    );
    let dim_concepts = extract_dim_concepts(&results);
    (results, dim_concepts)
}

fn extract_dim_concepts(results: &[MappingResult]) -> Vec<DimNCITConcept> {
    let mut concepts = Vec::new();
    for result in results {
        if let Some(ncit_id) = &result.ncit_id {
            concepts.push(DimNCITConcept {
                ncit_id: ncit_id.clone(),
                preferred_name: String::new(),
                semantic_group: String::new(),
            });
        }
    }
    normalize_concepts_for_dim(concepts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dfps_core::mapping::{MappingStrategy, MappingThresholds};
    use dfps_core::staging::StgSrCodeExploded;

    fn sample_code() -> CodeElement {
        CodeElement::from(StgSrCodeExploded {
            sr_id: "sr-1".into(),
            system: Some("system".into()),
            code: Some("code".into()),
            display: Some("display".into()),
        })
    }

    fn config_with_thresholds(auto: f32, review: f32) -> MappingConfig {
        MappingConfig {
            thresholds: MappingThresholds {
                auto_map_min: auto,
                needs_review_min: review,
            },
            ..MappingConfig::default()
        }
    }

    #[test]
    fn classify_respects_threshold_ordering() {
        let thresholds = MappingThresholds {
            auto_map_min: 0.8,
            needs_review_min: 0.5,
        };
        assert_eq!(classify(0.82, &thresholds), MappingState::AutoMapped);
        assert_eq!(classify(0.6, &thresholds), MappingState::NeedsReview);
        assert_eq!(classify(0.2, &thresholds), MappingState::NoMatch);
    }

    #[test]
    fn build_result_flags_no_match_reason() {
        let code = sample_code();
        let config = config_with_thresholds(0.9, 0.7);
        let result = build_result_with_score(
            &code,
            None,
            Some("C0001".into()),
            0.3,
            MappingStrategy::Composite,
            None,
            &config,
        );
        assert_eq!(result.state, MappingState::NoMatch);
        assert_eq!(result.ncit_id, None);
        assert_eq!(result.reason.as_deref(), Some("score_below_threshold"));
    }

    #[test]
    fn build_result_keeps_reason_for_matches() {
        let code = sample_code();
        let config = config_with_thresholds(0.8, 0.6);
        let result = build_result_with_score(
            &code,
            Some("CUI".into()),
            Some("C9999".into()),
            0.85,
            MappingStrategy::Composite,
            Some("umls_direct_xref".into()),
            &config,
        );
        assert_eq!(result.state, MappingState::AutoMapped);
        assert_eq!(result.ncit_id.as_deref(), Some("C9999"));
        assert_eq!(result.cui.as_deref(), Some("CUI"));
        assert_eq!(result.reason.as_deref(), Some("umls_direct_xref"));
    }
}
