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
