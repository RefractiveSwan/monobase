//! End-to-end pipeline facade stitching together FHIR ingestion and NCIt mapping.
//!
//! Matches the flows in `docs/system-design/fhir/index.md#quickstart` and
//! `docs/system-design/ncit/behavior/sequence-servicerequest.md` by exposing a
//! single entrypoint from Bundle -> staging -> NCIt concepts.

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
    MockVectorStore, QdrantVectorStore, VectorBackend, VectorStoreConfig, VectorUsageSnapshot,
};
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
    let (flats, exploded) = bundle_to_staging(bundle)?;
    let (mapping_results, dim_concepts, usage) =
        try_vector_mapping(&exploded).unwrap_or_else(|| {
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
) -> Option<(
    Vec<MappingResult>,
    Vec<DimNCITConcept>,
    Option<VectorUsageSnapshot>,
)> {
    let config = VectorStoreConfig::from_env().ok()?;
    if !config.enabled {
        return None;
    }

    let embedder = DeterministicEmbeddingProvider::new();
    let top_k = 5;

    match config.backend {
        VectorBackend::Qdrant => {
            let store = QdrantVectorStore::from_config(&config).ok()?;
            let store = Arc::new(store);
            map_staging_codes_with_vector(exploded.to_owned(), store, config, embedder, top_k)
                .ok()
                .map(|(results, dims, _summary, usage)| (results, dims, Some(usage)))
        }
        VectorBackend::Mock => {
            let store = Arc::new(MockVectorStore::new(config.namespace.clone()));
            map_staging_codes_with_vector(exploded.to_owned(), store, config, embedder, top_k)
                .ok()
                .map(|(results, dims, _summary, usage)| (results, dims, Some(usage)))
        }
        _ => None,
    }
}
