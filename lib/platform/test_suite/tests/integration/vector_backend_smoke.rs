use std::sync::Arc;

use dfps_vector_store::{MockVectorStore, VectorBackend, VectorStore, VectorStoreConfig};

#[test]
fn backend_feature_flag_skips_network_when_disabled() {
    let store = Arc::new(MockVectorStore::new("ncit_dev"));
    let config = VectorStoreConfig {
        backend: VectorBackend::Qdrant,
        url: Some("http://localhost:6333".into()),
        namespace: "ncit_dev".into(),
        pool_max: 2,
        health_timeout_ms: 200,
        enabled: false,
    };
    // Health should not be called when disabled; we just ensure config parses and store is reachable.
    assert!(config.validate().is_ok());
    let status = store.health("ncit_dev");
    assert!(status.is_ok());
}

#[test]
fn index_items_rejects_dimension_mismatch() {
    let store = MockVectorStore::new("ncit_dev");
    let items = vec![
        dfps_vector_store::VectorItem {
            ref_id: "C1".into(),
            embedding: dfps_vector_store::Embedding {
                vector: vec![0.1, 0.2],
                metadata: dfps_vector_store::EmbeddingMetadata {
                    embedding_version: "test".into(),
                    dim: 2,
                },
            },
        },
        dfps_vector_store::VectorItem {
            ref_id: "C2".into(),
            embedding: dfps_vector_store::Embedding {
                vector: vec![0.1, 0.2, 0.3],
                metadata: dfps_vector_store::EmbeddingMetadata {
                    embedding_version: "test".into(),
                    dim: 3,
                },
            },
        },
    ];
    let result = store.index_items("ncit_dev", &items);
    assert!(result.is_err());
}

#[test]
fn pgvector_backend_is_constructible_with_feature() {
    #[cfg(feature = "backend-pgvector")]
    {
        use dfps_vector_store::PgVectorStore;
        let config = VectorStoreConfig {
            backend: VectorBackend::PgVector,
            url: Some("postgres://vector:vector@localhost:5432/vector".into()),
            namespace: "ncit_dev".into(),
            pool_max: 2,
            health_timeout_ms: 200,
            enabled: true,
        };
        let store = PgVectorStore::from_config(&config);
        assert!(store.is_ok());
    }
}
