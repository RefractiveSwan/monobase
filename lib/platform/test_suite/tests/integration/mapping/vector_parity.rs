//! Ensures mock vector + baseline lexical parity (VEC-013).

use std::sync::Arc;

use dfps_core::mapping::MappingState;
use dfps_eval::{self, EvalCase, EvalSummary};
use dfps_mapping::{
    DeterministicEmbeddingProvider, map_staging_codes_with_summary, map_staging_codes_with_vector,
};
use dfps_test_suite::fixtures;
use dfps_vector_store::{MockVectorStore, VectorBackend, VectorStoreConfig};

fn build_codes_from_cases(cases: &[EvalCase]) -> Vec<dfps_core::staging::StgSrCodeExploded> {
    cases
        .iter()
        .map(|case| dfps_core::staging::StgSrCodeExploded {
            sr_id: format!("SR-{}", case.code),
            system: Some(case.system.clone()),
            code: Some(case.code.clone()),
            display: Some(case.display.clone()),
        })
        .collect()
}

fn summary_states(summary: &EvalSummary) -> (usize, usize, usize) {
    let auto = summary
        .state_counts
        .get("auto_mapped")
        .copied()
        .unwrap_or(0);
    let review = summary
        .state_counts
        .get("needs_review")
        .copied()
        .unwrap_or(0);
    let no_match = summary.state_counts.get("no_match").copied().unwrap_or(0);
    (auto, review, no_match)
}

#[test]
fn offline_and_vector_parity_on_pet_ct_small() {
    dfps_test_suite::init_environment().expect("load test env");
    let cases = fixtures::eval_pet_ct_small_cases();
    let codes = build_codes_from_cases(&cases);

    let (baseline_results, _, _baseline_summary) = map_staging_codes_with_summary(codes.clone());
    let mut baseline_eval = EvalSummary::default();
    baseline_eval.total_cases = baseline_results.len();
    for result in &baseline_results {
        match result.state {
            MappingState::AutoMapped => baseline_eval
                .state_counts
                .entry("auto_mapped".into())
                .and_modify(|v| *v += 1)
                .or_insert(1),
            MappingState::NeedsReview => baseline_eval
                .state_counts
                .entry("needs_review".into())
                .and_modify(|v| *v += 1)
                .or_insert(1),
            MappingState::NoMatch => baseline_eval
                .state_counts
                .entry("no_match".into())
                .and_modify(|v| *v += 1)
                .or_insert(1),
        };
    }

    let store = Arc::new(MockVectorStore::new("ncit_dev"));
    store.set_response(
        "ncit_dev",
        baseline_results
            .iter()
            .filter_map(|r| r.ncit_id.clone())
            .enumerate()
            .map(|(idx, id)| dfps_vector_store::VectorSearchHit {
                ref_id: id.replace("NCIT:", ""),
                score: 0.9 - (idx as f32) * 0.001,
            })
            .collect(),
    );
    let config = VectorStoreConfig {
        backend: VectorBackend::Mock,
        url: None,
        namespace: "ncit_dev".into(),
        pool_max: 2,
        health_timeout_ms: 250,
        enabled: true,
    };

    let (vector_results, _, _, usage) = map_staging_codes_with_vector(
        codes,
        store,
        config,
        DeterministicEmbeddingProvider::new(),
        3,
    )
    .expect("vector mapping");

    let mut vector_eval = EvalSummary::default();
    vector_eval.total_cases = vector_results.len();
    for result in &vector_results {
        match result.state {
            MappingState::AutoMapped => vector_eval
                .state_counts
                .entry("auto_mapped".into())
                .and_modify(|v| *v += 1)
                .or_insert(1),
            MappingState::NeedsReview => vector_eval
                .state_counts
                .entry("needs_review".into())
                .and_modify(|v| *v += 1)
                .or_insert(1),
            MappingState::NoMatch => vector_eval
                .state_counts
                .entry("no_match".into())
                .and_modify(|v| *v += 1)
                .or_insert(1),
        };
    }

    assert_eq!(baseline_results.len(), vector_results.len());
    assert_eq!(summary_states(&baseline_eval), summary_states(&vector_eval));
    assert_eq!(usage.fallbacks, 0);
}
