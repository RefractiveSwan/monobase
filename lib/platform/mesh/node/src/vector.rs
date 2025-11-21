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
