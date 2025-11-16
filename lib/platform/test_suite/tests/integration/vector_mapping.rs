use std::sync::Arc;

use dfps_core::{mapping::MappingState, staging::StgSrCodeExploded};
use dfps_mapping::{
    DeterministicEmbeddingProvider, map_staging_codes_with_summary, map_staging_codes_with_vector,
};
use dfps_vector_store::{MockVectorStore, VectorBackend, VectorSearchHit, VectorStoreConfig};

#[test]
fn vector_backend_improves_recall_over_baseline_mock() {
    let codes = vec![StgSrCodeExploded {
        sr_id: "SR-vec-1".into(),
        system: Some("http://loinc.org".into()),
        code: Some("X1".into()),
        display: Some("Unmapped example display".into()),
    }];

    let (baseline_results, _, _baseline_summary) = map_staging_codes_with_summary(codes.clone());
    let baseline_automapped = baseline_results
        .iter()
        .filter(|result| matches!(result.state, MappingState::AutoMapped))
        .count();
    assert_eq!(baseline_automapped, 0);

    let store = Arc::new(MockVectorStore::new("ncit_dev"));
    store.set_response(
        "ncit_dev",
        vec![VectorSearchHit {
            ref_id: "C99999".into(),
            score: 0.97,
        }],
    );
    let config = VectorStoreConfig {
        backend: VectorBackend::Mock,
        url: None,
        namespace: "ncit_dev".into(),
        pool_max: 2,
        health_timeout_ms: 250,
        enabled: true,
    };

    let (results, _, _summary, snapshot) = map_staging_codes_with_vector(
        codes,
        store.clone(),
        config,
        DeterministicEmbeddingProvider::new(),
        3,
    )
    .expect("vector mapping");

    let automapped = results
        .iter()
        .filter(|result| matches!(result.state, MappingState::AutoMapped))
        .count();
    assert_eq!(automapped, 1);
    assert_eq!(snapshot.hits, 1);
    assert_eq!(snapshot.fallbacks, 0);
    assert!(
        results
            .iter()
            .any(|r| r.ncit_id.as_deref() == Some("NCIT:C99999"))
    );
}

#[test]
fn vector_toggle_false_increments_fallbacks() {
    let codes = vec![StgSrCodeExploded {
        sr_id: "SR-toggle".into(),
        system: Some("http://loinc.org".into()),
        code: Some("1111-1".into()),
        display: Some("Toggle test".into()),
    }];

    let store = Arc::new(MockVectorStore::new("ncit_dev"));
    let config = VectorStoreConfig {
        backend: VectorBackend::Mock,
        url: None,
        namespace: "ncit_dev".into(),
        pool_max: 2,
        health_timeout_ms: 250,
        enabled: false,
    };

    let (results, _, _summary, snapshot) = map_staging_codes_with_vector(
        codes,
        store.clone(),
        config,
        DeterministicEmbeddingProvider::new(),
        3,
    )
    .expect("vector mapping");

    assert_eq!(snapshot.fallbacks, 1);
    assert!(!results.is_empty());
}
