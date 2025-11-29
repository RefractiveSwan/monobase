use std::sync::Arc;

use refractive_swan_pipeline::VectorPipelineContext;
use refractive_swan_vector_port::{MockVectorStore, VectorBackend, VectorStore};
use refractive_swan_vector_store::{QdrantVectorStore, VectorStoreConfig};

/// Build a [`VectorPipelineContext`] from config (if enabled).
pub fn vector_context_from_config(config: &VectorStoreConfig) -> Option<VectorPipelineContext> {
    if !config.enabled {
        return None;
    }

    let store: Arc<dyn VectorStore> = match config.backend {
        VectorBackend::Qdrant => {
            let store = QdrantVectorStore::from_config(config).ok()?;
            Arc::new(store)
        }
        VectorBackend::Mock => Arc::new(MockVectorStore::new(config.namespace.clone())),
        _ => return None,
    };

    Some(VectorPipelineContext::new(store, config.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_vector_context_from_mock_config() {
        let config = VectorStoreConfig {
            backend: VectorBackend::Mock,
            url: None,
            namespace: "test-ns".into(),
            pool_max: 2,
            health_timeout_ms: 1000,
            enabled: true,
        };
        let ctx = vector_context_from_config(&config).expect("context");
        assert_eq!(ctx.backend(), VectorBackend::Mock);
        assert_eq!(ctx.top_k(), 5);
        assert_eq!(ctx.namespace(), "test-ns");
    }
}
