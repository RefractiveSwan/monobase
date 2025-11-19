use std::sync::Arc;

use dfps_pipeline::VectorPipelineContext;
use dfps_vector_store::{
    MockVectorStore, QdrantVectorStore, VectorBackend, VectorStore, VectorStoreConfig,
};

pub fn pipeline_vector_context_from_env() -> Option<VectorPipelineContext> {
    let config = VectorStoreConfig::from_env().ok()?;
    if !config.enabled {
        return None;
    }
    let store: Arc<dyn VectorStore> = match config.backend {
        VectorBackend::Qdrant => {
            let store = QdrantVectorStore::from_config(&config).ok()?;
            Arc::new(store)
        }
        VectorBackend::Mock => Arc::new(MockVectorStore::new(config.namespace.clone())),
        _ => return None,
    };
    Some(VectorPipelineContext::new(store, config))
}
