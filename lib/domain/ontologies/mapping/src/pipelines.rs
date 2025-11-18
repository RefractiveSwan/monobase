use std::collections::HashSet;
use std::sync::Arc;

use dfps_compliance::ComplianceAction;
use dfps_core::mapping::{
    CodeElement, DimNCITConcept, MappingResult, MappingState, MappingStrategy,
};
use dfps_core::staging::StgSrCodeExploded;
use dfps_terminology::{CodeKind, EnrichedCode, TerminologyClient, TerminologyResult};
use dfps_vector_store::{EmbeddingProvider, VectorStore, VectorStoreConfig};

use crate::config::MappingConfig;
use crate::data::{load_ncit_concepts, load_umls_xrefs};
use crate::engine::{MappingEngine, RuleReranker};
use crate::rankers::{LexicalRanker, VectorRankerBackend, VectorRankerError, VectorRankerMock};
use crate::traits::CandidateRanker;
use crate::types::MappingSummary;

pub fn map_staging_codes<I>(codes: I) -> (Vec<MappingResult>, Vec<DimNCITConcept>)
where
    I: IntoIterator<Item = StgSrCodeExploded>,
{
    let (results, dims, _) = map_staging_codes_with_summary(codes);
    (results, dims)
}

pub fn map_staging_codes_with_summary<I>(
    codes: I,
) -> (Vec<MappingResult>, Vec<DimNCITConcept>, MappingSummary)
where
    I: IntoIterator<Item = StgSrCodeExploded>,
{
    map_staging_codes_with_summary_with_config(codes, None, &MappingConfig::default())
}

pub fn map_staging_codes_with_summary_and_policy<I>(
    codes: I,
    policy: &dfps_compliance::Policy,
) -> (Vec<MappingResult>, Vec<DimNCITConcept>, MappingSummary)
where
    I: IntoIterator<Item = StgSrCodeExploded>,
{
    let config = MappingConfig::with_policy(policy.clone());
    map_staging_codes_with_summary_with_config(codes, None, &config)
}

pub fn map_staging_codes_with_summary_with_policy<I>(
    codes: I,
    client: Option<&dyn TerminologyClient>,
    policy: &dfps_compliance::Policy,
) -> (Vec<MappingResult>, Vec<DimNCITConcept>, MappingSummary)
where
    I: IntoIterator<Item = StgSrCodeExploded>,
{
    let config = MappingConfig::with_policy(policy.clone());
    map_staging_codes_with_summary_with_config(codes, client, &config)
}

pub fn map_staging_codes_with_summary_with_client<I>(
    codes: I,
    client: Option<&dyn TerminologyClient>,
) -> (Vec<MappingResult>, Vec<DimNCITConcept>, MappingSummary)
where
    I: IntoIterator<Item = StgSrCodeExploded>,
{
    map_staging_codes_with_summary_with_config(codes, client, &MappingConfig::default())
}

pub fn map_staging_codes_with_summary_with_config<I>(
    codes: I,
    client: Option<&dyn TerminologyClient>,
    config: &MappingConfig,
) -> (Vec<MappingResult>, Vec<DimNCITConcept>, MappingSummary)
where
    I: IntoIterator<Item = StgSrCodeExploded>,
{
    let dim_concepts = dim_concepts();
    let xrefs = load_umls_xrefs();
    let engine = MappingEngine::new(LexicalRanker, VectorRankerMock, RuleReranker);
    let (results, summary) = map_with_engine(codes.into_iter(), &engine, &xrefs, client, config);
    (results, dim_concepts, summary)
}

pub fn map_staging_codes_with_vector<I, S, E>(
    codes: I,
    store: Arc<S>,
    vector_config: VectorStoreConfig,
    embedding: E,
    top_k: usize,
) -> Result<
    (
        Vec<MappingResult>,
        Vec<DimNCITConcept>,
        MappingSummary,
        dfps_vector_store::VectorUsageSnapshot,
    ),
    VectorRankerError,
>
where
    I: IntoIterator<Item = StgSrCodeExploded>,
    S: VectorStore,
    E: EmbeddingProvider<CodeElement>,
{
    map_staging_codes_with_vector_and_config(
        codes,
        store,
        vector_config,
        embedding,
        top_k,
        &MappingConfig::default(),
    )
}

pub fn map_staging_codes_with_vector_and_policy<I, S, E>(
    codes: I,
    store: Arc<S>,
    vector_config: VectorStoreConfig,
    embedding: E,
    top_k: usize,
    policy: &dfps_compliance::Policy,
) -> Result<
    (
        Vec<MappingResult>,
        Vec<DimNCITConcept>,
        MappingSummary,
        dfps_vector_store::VectorUsageSnapshot,
    ),
    VectorRankerError,
>
where
    I: IntoIterator<Item = StgSrCodeExploded>,
    S: VectorStore,
    E: EmbeddingProvider<CodeElement>,
{
    let mapping_config = MappingConfig::with_policy(policy.clone());
    map_staging_codes_with_vector_and_config(
        codes,
        store,
        vector_config,
        embedding,
        top_k,
        &mapping_config,
    )
}

pub fn map_staging_codes_with_vector_and_config<I, S, E>(
    codes: I,
    store: Arc<S>,
    vector_config: VectorStoreConfig,
    embedding: E,
    top_k: usize,
    mapping_config: &MappingConfig,
) -> Result<
    (
        Vec<MappingResult>,
        Vec<DimNCITConcept>,
        MappingSummary,
        dfps_vector_store::VectorUsageSnapshot,
    ),
    VectorRankerError,
>
where
    I: IntoIterator<Item = StgSrCodeExploded>,
    S: VectorStore,
    E: EmbeddingProvider<CodeElement>,
{
    let dim_concepts = dim_concepts();
    let xrefs = load_umls_xrefs();
    let vector_ranker = VectorRankerBackend::from_config(store, embedding, vector_config, top_k)?;
    let usage_handle = vector_ranker.usage_handle();
    let engine = MappingEngine::new(LexicalRanker, vector_ranker, RuleReranker);
    let (results, summary) = map_with_engine(
        codes.into_iter(),
        &engine,
        &xrefs,
        None as Option<&dyn TerminologyClient>,
        mapping_config,
    );
    let snapshot = usage_handle.snapshot();
    Ok((results, dim_concepts, summary, snapshot))
}

fn map_with_engine<I, L, V>(
    codes: I,
    engine: &MappingEngine<L, V>,
    xrefs: &std::collections::HashMap<(String, String), crate::data::UmlsXref>,
    client: Option<&dyn TerminologyClient>,
    config: &MappingConfig,
) -> (Vec<MappingResult>, MappingSummary)
where
    I: IntoIterator<Item = StgSrCodeExploded>,
    L: CandidateRanker,
    V: CandidateRanker,
{
    let mut results = Vec::new();
    let mut summary = MappingSummary::default();

    for staging in codes {
        let enriched = EnrichedCode::from_staging(staging.clone());
        let code_kind = enriched.code_kind();
        let element = CodeElement::from(staging);
        let system_value = enriched.staging.system.clone().unwrap_or_default();
        let code_value = enriched.staging.code.clone().unwrap_or_default();
        let key = (system_value.clone(), code_value.clone());

        summary.record(code_kind, enriched.license_label());

        if let Some(tier) = enriched.license_tier {
            if !config.policy.is_allowed(ComplianceAction::Map, tier) {
                let mut blocked = build_result_with_score(
                    &element,
                    None,
                    None,
                    0.0,
                    MappingStrategy::Unmapped,
                    Some("license_blocked".into()),
                    config,
                );
                attach_license_metadata(&mut blocked, &enriched);
                results.push(blocked);
                continue;
            }
        }

        let mut result = match code_kind {
            CodeKind::MissingSystemOrCode => build_result_with_score(
                &element,
                None,
                None,
                0.0,
                MappingStrategy::Unmapped,
                Some("missing_system_or_code".into()),
                config,
            ),
            CodeKind::UnknownSystem => {
                if matches!(enriched.license_label(), Some(label) if label == "forbidden") {
                    summary.record_external_miss();
                    build_result_with_score(
                        &element,
                        None,
                        None,
                        0.0,
                        MappingStrategy::Unmapped,
                        Some("license_forbidden".into()),
                        config,
                    )
                } else {
                    let base = build_result_with_score(
                        &element,
                        None,
                        None,
                        0.0,
                        MappingStrategy::Unmapped,
                        Some("unknown_code_system".into()),
                        config,
                    );
                    if let Some(client) = client {
                        match external_lookup(client, &system_value, &code_value) {
                            Ok(Some((cui, ncit_id))) => {
                                summary.record_external_success();
                                build_result_with_score(
                                    &element,
                                    cui,
                                    ncit_id,
                                    0.95,
                                    MappingStrategy::Rule,
                                    Some("external_terminology_lookup".into()),
                                    config,
                                )
                            }
                            Ok(None) => {
                                summary.record_external_miss();
                                base
                            }
                            Err(_) => {
                                summary.record_external_error();
                                base
                            }
                        }
                    } else {
                        base
                    }
                }
            }
            _ => {
                if let Some(xref) = xrefs.get(&key) {
                    build_result_with_score(
                        &element,
                        Some(xref.cui.clone()),
                        Some(xref.ncit_id.clone()),
                        0.99,
                        MappingStrategy::Rule,
                        Some("umls_direct_xref".into()),
                        config,
                    )
                } else {
                    engine.map_with_config(&element, config)
                }
            }
        };

        attach_license_metadata(&mut result, &enriched);
        results.push(result);
    }

    (results, summary)
}

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

fn classify(score: f32, thresholds: &dfps_core::mapping::MappingThresholds) -> MappingState {
    if score >= thresholds.auto_map_min {
        MappingState::AutoMapped
    } else if score >= thresholds.needs_review_min {
        MappingState::NeedsReview
    } else {
        MappingState::NoMatch
    }
}

fn attach_license_metadata(result: &mut MappingResult, enriched: &EnrichedCode) {
    if let Some(label) = enriched.license_label() {
        result.license_tier = Some(label.to_string());
    }
    if let Some(label) = enriched.source_label() {
        result.source_kind = Some(label.to_string());
    }
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

fn dim_concepts() -> Vec<DimNCITConcept> {
    normalize_concepts_for_dim(lookup_dim_concepts())
}

fn normalize_code_for_dim(concept: &DimNCITConcept) -> DimNCITConcept {
    DimNCITConcept {
        ncit_id: crate::rankers::normalize_ncit_code(&concept.ncit_id),
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

fn external_lookup(
    client: &dyn TerminologyClient,
    system: &str,
    code: &str,
) -> TerminologyResult<Option<(Option<String>, Option<String>)>> {
    let mut cui = None;
    if let Some(record) = client.lookup_cui(system, code)? {
        cui = Some(record.cui);
    }

    let ncit_id = client
        .lookup_ncit(cui.as_deref().unwrap_or(code))?
        .map(|record| record.ncit_id);

    if cui.is_some() || ncit_id.is_some() {
        Ok(Some((cui, ncit_id)))
    } else {
        Ok(None)
    }
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

pub fn map_with_dim_concepts<I>(
    codes: I,
    engine: MappingEngine<impl CandidateRanker, impl CandidateRanker>,
) -> (Vec<MappingResult>, Vec<DimNCITConcept>)
where
    I: IntoIterator<Item = StgSrCodeExploded>,
{
    let xrefs = load_umls_xrefs();
    let (results, _) = map_with_engine(
        codes,
        &engine,
        &xrefs,
        None as Option<&dyn TerminologyClient>,
        &MappingConfig::default(),
    );
    let dim_concepts = extract_dim_concepts(&results);
    (results, dim_concepts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{MappingEngine, RuleReranker};
    use crate::rankers::{DeterministicEmbeddingProvider, LexicalRanker, VectorRankerMock};
    use crate::traits::Mapper;
    use crate::types::FusionWeights;
    use dfps_compliance::{ComplianceMode, Policy};
    use dfps_core::mapping::{MappingCandidate, MappingState, MappingStrategy};
    use dfps_core::staging::StgSrCodeExploded;
    use dfps_terminology::MockTerminologyClient;
    use dfps_vector_store::{MockVectorStore, VectorBackend, VectorSearchHit, VectorStoreConfig};
    use std::sync::Arc;

    #[test]
    fn engine_returns_deterministic_result() {
        let staging = StgSrCodeExploded {
            sr_id: "SR-1".into(),
            system: Some("http://snomed.info/sct".into()),
            code: Some("123".into()),
            display: Some("PET CT staging".into()),
        };
        let code = CodeElement::from(staging);
        let engine = MappingEngine::new(LexicalRanker, VectorRankerMock, RuleReranker);

        let result = engine.map(&code);
        assert!(result.score > 0.5);
        assert!(result.ncit_id.unwrap().starts_with("NCIT:"));
        assert_ne!(result.state, MappingState::NoMatch);
    }

    #[test]
    fn summary_tracks_code_kind_and_license_counts() {
        let codes = vec![
            StgSrCodeExploded {
                sr_id: "SR-1".into(),
                system: Some("http://www.ama-assn.org/go/cpt".into()),
                code: Some("78815".into()),
                display: None,
            },
            StgSrCodeExploded {
                sr_id: "SR-2".into(),
                system: Some("http://example.org/custom".into()),
                code: Some("A1".into()),
                display: None,
            },
            StgSrCodeExploded {
                sr_id: "SR-3".into(),
                system: None,
                code: Some("B1".into()),
                display: None,
            },
        ];

        let (_, _, summary) = map_staging_codes_with_summary(codes);

        assert_eq!(summary.total, 3);
        assert_eq!(summary.by_code_kind.get("known_licensed_system"), Some(&1));
        assert_eq!(summary.by_code_kind.get("unknown_system"), Some(&1));
        assert_eq!(summary.by_code_kind.get("missing_system_or_code"), Some(&1));
        assert_eq!(summary.by_license_tier.get("licensed"), Some(&1));
        assert_eq!(summary.by_license_tier.get("unknown"), Some(&2));
    }

    #[test]
    fn unknown_system_uses_external_client() {
        let codes = vec![StgSrCodeExploded {
            sr_id: "SR-1".into(),
            system: Some("http://unknown.test/system".into()),
            code: Some("X1".into()),
            display: Some("Unknown".into()),
        }];
        let mut mock = MockTerminologyClient::default();
        mock = mock.with_cui(
            "http://unknown.test/system",
            "X1",
            dfps_terminology::CuiRecord {
                cui: "CEXTERNAL".into(),
                preferred_name: "External".into(),
            },
        );
        mock = mock.with_ncit(
            "C9999",
            dfps_terminology::NcitRecord {
                ncit_id: "C9999".into(),
                preferred_name: "External NCIt".into(),
                synonyms: vec![],
            },
        );
        let (results, _dims, summary) =
            map_staging_codes_with_summary_with_client(codes, Some(&mock));
        assert_eq!(summary.extern_lookup_success, 1);
        let result = &results[0];
        assert_eq!(result.strategy, MappingStrategy::Rule);
        assert_eq!(
            result.reason.as_deref(),
            Some("external_terminology_lookup")
        );
    }

    #[test]
    fn compliance_blocks_licensed_codes_in_open_source_mode() {
        let codes = vec![StgSrCodeExploded {
            sr_id: "SR-1".into(),
            system: Some("http://www.ama-assn.org/go/cpt".into()),
            code: Some("78815".into()),
            display: None,
        }];

        let policy = Policy::default_for_mode(ComplianceMode::OpenSource);
        let (results, _, summary) = map_staging_codes_with_summary_and_policy(codes, &policy);

        assert_eq!(summary.total, 1);
        assert_eq!(results.len(), 1);
        let result = &results[0];
        assert_eq!(result.state, MappingState::NoMatch);
        assert_eq!(result.reason.as_deref(), Some("license_blocked"));
        assert_eq!(result.ncit_id, None);
    }

    #[test]
    fn vector_ranker_uses_backend_hits_and_normalizes_codes() {
        let store = Arc::new(MockVectorStore::new("ncit_dev"));
        store.set_response(
            "ncit_dev",
            vec![
                VectorSearchHit {
                    ref_id: "C12345".into(),
                    score: 0.72,
                },
                VectorSearchHit {
                    ref_id: "NCIT:C00010".into(),
                    score: 0.81,
                },
            ],
        );
        let config = VectorStoreConfig {
            backend: VectorBackend::Mock,
            url: None,
            namespace: "ncit_dev".into(),
            pool_max: 4,
            health_timeout_ms: 250,
            enabled: true,
        };
        let ranker = VectorRankerBackend::from_config(
            store.clone(),
            DeterministicEmbeddingProvider::new(),
            config,
            3,
        )
        .expect("vector ranker");
        let usage = ranker.usage_handle();
        let engine = MappingEngine::new(LexicalRanker, ranker, RuleReranker);
        let staging = StgSrCodeExploded {
            sr_id: "SR-vec".into(),
            system: Some("http://ncit.nci.nih.gov".into()),
            code: Some("C19951".into()),
            display: Some("PET CT staging".into()),
        };
        let code = CodeElement::from(staging);
        let candidates = engine.ranked_candidates(&code);
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.target_code == "NCIT:C00010")
        );
        assert!(candidates[0].score >= candidates[1].score);
        let snapshot = usage.snapshot();
        assert_eq!(snapshot.capacity, None);
    }

    #[test]
    fn vector_ranker_captures_capacity_proxies_from_backend() {
        let store = Arc::new(MockVectorStore::new("ncit_dev").with_capacity(
            dfps_vector_store::CapacityProxies {
                geom_rm_sqrt_dm: Some(1.25),
                cap_alpha_sim: Some(0.92),
                ..dfps_vector_store::CapacityProxies::default()
            },
        ));
        let config = VectorStoreConfig {
            backend: VectorBackend::Mock,
            url: None,
            namespace: "ncit_dev".into(),
            pool_max: 2,
            health_timeout_ms: 200,
            enabled: true,
        };
        let ranker = VectorRankerBackend::from_config(
            store.clone(),
            DeterministicEmbeddingProvider::new(),
            config,
            2,
        )
        .expect("vector ranker");
        let usage = ranker.usage_handle();
        let engine = MappingEngine::new(LexicalRanker, ranker, RuleReranker);
        let code = CodeElement::new(
            "SR-capacity",
            Some("http://loinc.org".into()),
            Some("1111-1".into()),
            Some("Example display".into()),
        );
        let _ = engine.ranked_candidates(&code);
        let snapshot = usage.snapshot();
        assert!(snapshot.capacity.is_some());
        let capacity = snapshot.capacity.unwrap();
        assert_eq!(capacity.geom_rm_sqrt_dm, Some(1.25));
        assert_eq!(capacity.cap_alpha_sim, Some(0.92));
    }

    #[test]
    fn vector_ranker_disabled_falls_back_to_mock() {
        let store = Arc::new(MockVectorStore::new("ncit_dev"));
        let config = VectorStoreConfig {
            backend: VectorBackend::Mock,
            url: None,
            namespace: "ncit_dev".into(),
            pool_max: 1,
            health_timeout_ms: 100,
            enabled: false,
        };
        let ranker = VectorRankerBackend::from_config(
            store.clone(),
            DeterministicEmbeddingProvider::new(),
            config,
            0,
        )
        .expect("vector ranker");
        let usage = ranker.usage_handle();
        let engine = MappingEngine::new(LexicalRanker, ranker, RuleReranker);
        let code = CodeElement::new(
            "SR-raw",
            Some("http://example.org/system".into()),
            Some("A1".into()),
            Some("Example display".into()),
        );
        let candidates = engine.ranked_candidates(&code);
        assert!(!candidates.is_empty());
        let snapshot = usage.snapshot();
        assert_eq!(snapshot.fallbacks, 1);
    }

    #[derive(Clone)]
    struct FixedRanker {
        candidates: Vec<MappingCandidate>,
    }

    impl CandidateRanker for FixedRanker {
        fn rank(&self, _code: &CodeElement) -> Vec<MappingCandidate> {
            self.candidates.clone()
        }
    }

    #[test]
    fn fusion_weights_shift_scores_between_lexical_and_vector() {
        let lexical = FixedRanker {
            candidates: vec![MappingCandidate {
                target_system: "NCIT".into(),
                target_code: "LEX".into(),
                cui: None,
                score: 0.9,
            }],
        };
        let vector = FixedRanker {
            candidates: vec![MappingCandidate {
                target_system: "NCIT".into(),
                target_code: "VEC".into(),
                cui: None,
                score: 0.8,
            }],
        };
        let engine = MappingEngine::new_with_weights(
            lexical.clone(),
            vector.clone(),
            RuleReranker,
            FusionWeights::new(0.1, 1.0),
        );
        let code = CodeElement::new("id", Some("sys".into()), Some("code".into()), None);
        let ranked = engine.ranked_candidates(&code);
        assert_eq!(ranked[0].target_code, "VEC");
    }
}
