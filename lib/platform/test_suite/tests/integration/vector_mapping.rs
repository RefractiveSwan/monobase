use std::sync::Arc;

use dfps_core::{mapping::MappingState, staging::StgSrCodeExploded};
use dfps_mapping::{
    DeterministicEmbeddingProvider, map_staging_codes_with_summary, map_staging_codes_with_vector,
};
use dfps_vector_store::{
    CapacityProxies, MockVectorStore, VectorBackend, VectorSearchHit, VectorStoreConfig,
};

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

#[test]
fn vector_backend_shows_uplift_vs_baseline() {
    let codes = vec![
        StgSrCodeExploded {
            sr_id: "SR-uplift-1".into(),
            system: Some("http://loinc.org".into()),
            code: Some("1111-1".into()),
            display: Some("uplift code 1".into()),
        },
        StgSrCodeExploded {
            sr_id: "SR-uplift-2".into(),
            system: Some("http://loinc.org".into()),
            code: Some("2222-2".into()),
            display: Some("uplift code 2".into()),
        },
    ];

    let (baseline_results, _, _) = map_staging_codes_with_summary(codes.clone());
    let baseline_auto = baseline_results
        .iter()
        .filter(|r| matches!(r.state, MappingState::AutoMapped))
        .count();

    let store = Arc::new(MockVectorStore::new("ncit_dev"));
    store.set_response(
        "ncit_dev",
        vec![
            VectorSearchHit {
                ref_id: "C12345".into(),
                score: 0.99,
            },
            VectorSearchHit {
                ref_id: "C54321".into(),
                score: 0.97,
            },
        ],
    );
    let codes_len = codes.len();
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
        store.clone(),
        config,
        DeterministicEmbeddingProvider::new(),
        3,
    )
    .expect("vector mapping");
    assert_eq!(vector_results.len(), codes_len);
    let vector_auto = vector_results
        .iter()
        .filter(|r| matches!(r.state, MappingState::AutoMapped))
        .count();

    assert_eq!(baseline_auto, 0);
    assert!(
        vector_auto > baseline_auto,
        "vector mode should uplift automapped count"
    );
    assert_eq!(usage.fallbacks, 0);
    assert!(usage.hits >= vector_auto);
}

#[test]
fn capacity_proxy_is_carried_from_vector_store() {
    let codes = vec![StgSrCodeExploded {
        sr_id: "SR-cap".into(),
        system: Some("http://loinc.org".into()),
        code: Some("3333-3".into()),
        display: Some("cap test".into()),
    }];

    let store = Arc::new(
        MockVectorStore::new("ncit_dev").with_capacity(CapacityProxies {
            geom_rm: Some(0.12),
            geom_dm: Some(3.0),
            geom_rm_sqrt_dm: Some(0.2),
            cap_alpha_sim: Some(0.9),
        }),
    );
    let config = VectorStoreConfig {
        backend: VectorBackend::Mock,
        url: None,
        namespace: "ncit_dev".into(),
        pool_max: 2,
        health_timeout_ms: 250,
        enabled: true,
    };

    let (_, _, _, usage) = map_staging_codes_with_vector(
        codes,
        store.clone(),
        config,
        DeterministicEmbeddingProvider::new(),
        3,
    )
    .expect("vector mapping");

    assert!(usage.capacity.is_some());
    let cap = usage.capacity.unwrap();
    assert_eq!(cap.geom_rm_sqrt_dm, Some(0.2));
    assert_eq!(cap.cap_alpha_sim, Some(0.9));
}

#[test]
fn vector_ci_guard_recall_no_regressions_against_baseline() {
    let codes = vec![
        StgSrCodeExploded {
            sr_id: "SR-ci-1".into(),
            system: Some("http://loinc.org".into()),
            code: Some("4444-4".into()),
            display: Some("ci guard code 1".into()),
        },
        StgSrCodeExploded {
            sr_id: "SR-ci-2".into(),
            system: Some("http://loinc.org".into()),
            code: Some("5555-5".into()),
            display: Some("ci guard code 2".into()),
        },
    ];

    let (baseline_results, _, _) = map_staging_codes_with_summary(codes.clone());
    let baseline_auto = baseline_results
        .iter()
        .filter(|r| matches!(r.state, MappingState::AutoMapped))
        .count();

    // Seed mock store with baseline NCIT ids so vector path matches or improves recall.
    let store = Arc::new(MockVectorStore::new("ncit_dev"));
    let hits: Vec<VectorSearchHit> = baseline_results
        .iter()
        .filter_map(|r| r.ncit_id.clone())
        .enumerate()
        .map(|(idx, id)| VectorSearchHit {
            ref_id: id.replace("NCIT:", ""),
            score: 0.9 - (idx as f32) * 0.001,
        })
        .collect();
    store.set_response("ncit_dev", hits);
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
    let vector_auto = vector_results
        .iter()
        .filter(|r| matches!(r.state, MappingState::AutoMapped))
        .count();

    assert!(
        vector_auto >= baseline_auto,
        "vector recall should not regress (baseline {} vs vector {})",
        baseline_auto,
        vector_auto
    );
    assert_eq!(usage.fallbacks, 0);
}
