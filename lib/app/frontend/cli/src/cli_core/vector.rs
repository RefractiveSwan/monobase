use std::sync::Arc;

use dfps_pipeline::VectorPipelineContext;
use dfps_vector_store::{
    MockVectorStore, QdrantVectorStore, VectorBackend, VectorStore, VectorStoreConfig,
    config_from_env,
};

#[cfg(feature = "backend-pgvector")]
use dfps_vector_store::PgVectorStore;

use super::{CliError, CliResult};

pub fn load_vector_config() -> CliResult<VectorStoreConfig> {
    config_from_env().map_err(|err| CliError::config(format!("vector config error: {err}")))
}

pub fn pipeline_vector_context_from_env() -> CliResult<Option<VectorPipelineContext>> {
    let config = match config_from_env() {
        Ok(cfg) => cfg,
        Err(err) => return Err(CliError::config(format!("vector config error: {err}"))),
    };
    if !config.enabled {
        return Ok(None);
    }
    let store = mapping_vector_store(&config)?;
    Ok(Some(VectorPipelineContext::new(store, config)))
}

pub fn mapping_vector_store(config: &VectorStoreConfig) -> CliResult<Arc<dyn VectorStore>> {
    let backend = config.backend.clone();
    match backend {
        VectorBackend::Qdrant => {
            let store = QdrantVectorStore::from_config(config)
                .map_err(|err| CliError::external(format!("qdrant client error: {err}")))?;
            Ok(Arc::new(store))
        }
        VectorBackend::Mock => Ok(Arc::new(MockVectorStore::new(config.namespace.clone()))),
        VectorBackend::PgVector => {
            #[cfg(feature = "backend-pgvector")]
            {
                let store = PgVectorStore::from_config(config)
                    .map_err(|err| CliError::external(format!("pgvector client error: {err}")))?;
                Ok(Arc::new(store))
            }
            #[cfg(not(feature = "backend-pgvector"))]
            {
                Err(CliError::config(
                    "pgvector backend not enabled; recompile with backend-pgvector feature"
                        .to_string(),
                ))
            }
        }
        other => Err(CliError::config(format!(
            "backend '{other:?}' not supported in CLI"
        ))),
    }
}
