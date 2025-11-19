//! End-to-end pipeline facade stitching together FHIR ingestion and NCIt mapping.
//!
//! Matches the flows in:
//! - docs/system-design/fhir/index.md#quickstart
//! - docs/system-design/ncit/behavior/sequence-servicerequest.md
//! - lib/domain/pipeline/README.md (REFR-09 notes)
//! by exposing a single entrypoint from Bundle -> staging -> NCIt concepts,
//! with optional vector-store contexts injected by callers.

use dfps_core::{
    fhir::Bundle,
    mapping::{DimNCITConcept, MappingResult},
    staging::{StgServiceRequestFlat, StgSrCodeExploded},
};
use dfps_ingestion::bundle_to_staging;
use dfps_mapping::{
    DeterministicEmbeddingProvider, map_staging_codes, map_staging_codes_with_vector,
};
use dfps_vector_store::{
    VectorBackend, VectorItem, VectorSearchResult, VectorStore, VectorStoreConfig,
    VectorStoreError, VectorUsageSnapshot,
};
use log::warn;
use std::sync::Arc;
use thiserror::Error;

/// Aggregated pipeline output for a single Bundle ingestion/mapping run.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PipelineOutput {
    pub flats: Vec<StgServiceRequestFlat>,
    pub exploded_codes: Vec<StgSrCodeExploded>,
    pub mapping_results: Vec<MappingResult>,
    pub dim_concepts: Vec<DimNCITConcept>,
    pub vector_usage: Option<VectorUsageSnapshot>,
}

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("ingestion error: {0}")]
    Ingestion(#[from] dfps_ingestion::IngestionError),
}

pub fn bundle_to_mapped_sr(bundle: &Bundle) -> Result<PipelineOutput, PipelineError> {
    bundle_to_mapped_sr_with_vector_context(bundle, None)
}

pub fn bundle_to_mapped_sr_with_vector_context(
    bundle: &Bundle,
    vector: Option<&VectorPipelineContext>,
) -> Result<PipelineOutput, PipelineError> {
    let (flats, exploded) = bundle_to_staging(bundle)?;
    let (mapping_results, dim_concepts, usage) = vector
        .and_then(|ctx| try_vector_mapping(&exploded, ctx))
        .unwrap_or_else(|| {
            let (results, dims) = map_staging_codes(exploded.clone());
            (results, dims, None)
        });

    Ok(PipelineOutput {
        flats,
        exploded_codes: exploded,
        mapping_results,
        dim_concepts,
        vector_usage: usage,
    })
}

fn try_vector_mapping(
    exploded: &[StgSrCodeExploded],
    ctx: &VectorPipelineContext,
) -> Option<(
    Vec<MappingResult>,
    Vec<DimNCITConcept>,
    Option<VectorUsageSnapshot>,
)> {
    let embedder = DeterministicEmbeddingProvider::new();
    let erased = Arc::new(ErasedVectorStore::new(Arc::clone(&ctx.store)));
    match map_staging_codes_with_vector(
        exploded.to_owned(),
        erased,
        ctx.config.clone(),
        embedder,
        ctx.top_k,
    ) {
        Ok((results, dims, _summary, usage)) => Some((results, dims, Some(usage))),
        Err(err) => {
            warn!("vector mapping failed; falling back to lexical path: {err}");
            None
        }
    }
}

#[derive(Clone)]
pub struct VectorPipelineContext {
    store: Arc<dyn VectorStore>,
    config: VectorStoreConfig,
    top_k: usize,
}

impl VectorPipelineContext {
    pub fn new(store: Arc<dyn VectorStore>, config: VectorStoreConfig) -> Self {
        Self {
            store,
            config,
            top_k: 5,
        }
    }

    pub fn with_top_k(mut self, top_k: usize) -> Self {
        self.top_k = top_k.max(1);
        self
    }
}

impl std::fmt::Debug for VectorPipelineContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorPipelineContext")
            .field("backend", &self.config.backend)
            .field("namespace", &self.config.namespace)
            .field("top_k", &self.top_k)
            .finish()
    }
}

struct ErasedVectorStore(Arc<dyn VectorStore>);

impl ErasedVectorStore {
    fn new(inner: Arc<dyn VectorStore>) -> Self {
        Self(inner)
    }
}

impl VectorStore for ErasedVectorStore {
    fn backend(&self) -> VectorBackend {
        self.0.backend()
    }

    fn health(&self, namespace: &str) -> Result<(), VectorStoreError> {
        self.0.health(namespace)
    }

    fn index_items(&self, namespace: &str, items: &[VectorItem]) -> Result<(), VectorStoreError> {
        self.0.index_items(namespace, items)
    }

    fn search(
        &self,
        namespace: &str,
        query_vec: &[f32],
        top_k: usize,
    ) -> Result<VectorSearchResult, VectorStoreError> {
        self.0.search(namespace, query_vec, top_k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dfps_test_suite::regression;
    use dfps_vector_store::{MockVectorStore, VectorBackend};
    use serde_json::json;
    use std::sync::Arc;

    fn lexical_output() -> PipelineOutput {
        let bundle = regression::baseline_fhir_bundle();
        bundle_to_mapped_sr(&bundle).expect("lexical pipeline output")
    }

    fn mock_vector_context() -> VectorPipelineContext {
        let config = VectorStoreConfig {
            backend: VectorBackend::Mock,
            url: None,
            namespace: "test".into(),
            pool_max: 1,
            health_timeout_ms: 500,
            enabled: true,
        };
        let store: Arc<dyn VectorStore> = Arc::new(MockVectorStore::new(config.namespace.clone()));
        VectorPipelineContext::new(store, config)
    }

    #[test]
    fn lexical_path_produces_output_without_vector_usage() {
        let output = lexical_output();
        assert!(output.vector_usage.is_none());
        assert!(!output.mapping_results.is_empty());
    }

    #[test]
    fn vector_path_records_usage() {
        let bundle = regression::baseline_fhir_bundle();
        let ctx = mock_vector_context();
        let output =
            bundle_to_mapped_sr_with_vector_context(&bundle, Some(&ctx)).expect("vector run");
        assert!(
            output.vector_usage.is_some(),
            "vector-enabled context should populate usage snapshots"
        );
    }

    #[test]
    fn vector_context_absent_leaves_usage_empty() {
        let bundle = regression::baseline_fhir_bundle();
        let output = bundle_to_mapped_sr_with_vector_context(&bundle, None).expect("lexical run");
        assert!(output.vector_usage.is_none());
    }

    #[test]
    fn invalid_bundle_surfaces_ingestion_error() {
        let bundle: Bundle = serde_json::from_value(json!({
            "resourceType": "Bundle",
            "type": "collection",
            "entry": [
                {
                    "resource": {
                        "resourceType": "ServiceRequest",
                        "id": "sr-1",
                        "status": "active",
                        "intent": "order"
                    }
                }
            ]
        }))
        .expect("bundle json");
        let err = bundle_to_mapped_sr(&bundle).unwrap_err();
        match err {
            PipelineError::Ingestion(_) => {}
        }
    }
}
