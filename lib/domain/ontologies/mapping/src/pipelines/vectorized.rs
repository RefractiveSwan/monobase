use std::sync::Arc;

use dfps_compliance::Policy;
use dfps_core::mapping::{CodeElement, DimNCITConcept, MappingResult};
use dfps_core::staging::StgSrCodeExploded;
use dfps_vector_store::{EmbeddingProvider, VectorStore, VectorStoreConfig};

use crate::config::MappingConfig;
use crate::data::load_umls_xrefs;
use crate::engine::{MappingEngine, RuleReranker};
use crate::pipelines::helpers::dim_concepts;
use crate::rankers::{LexicalRanker, VectorRankerBackend, VectorRankerError};
use crate::types::MappingSummary;

pub fn map_staging_codes_with_vector<I, S, E>(
    codes: I,
    store: Arc<S>,
    vector_config: VectorStoreConfig,
    embedding: E,
    top_k: usize,
) -> Result<
    (
        Vec<MappingResult>,
        Vec<DimNCITConcept>,
        MappingSummary,
        dfps_vector_store::VectorUsageSnapshot,
    ),
    VectorRankerError,
>
where
    I: IntoIterator<Item = StgSrCodeExploded>,
    S: VectorStore,
    E: EmbeddingProvider<CodeElement>,
{
    map_staging_codes_with_vector_and_config(
        codes,
        store,
        vector_config,
        embedding,
        top_k,
        &MappingConfig::default(),
    )
}

pub fn map_staging_codes_with_vector_and_policy<I, S, E>(
    codes: I,
    store: Arc<S>,
    vector_config: VectorStoreConfig,
    embedding: E,
    top_k: usize,
    policy: &Policy,
) -> Result<
    (
        Vec<MappingResult>,
        Vec<DimNCITConcept>,
        MappingSummary,
        dfps_vector_store::VectorUsageSnapshot,
    ),
    VectorRankerError,
>
where
    I: IntoIterator<Item = StgSrCodeExploded>,
    S: VectorStore,
    E: EmbeddingProvider<CodeElement>,
{
    let mapping_config = MappingConfig::with_policy(policy.clone());
    map_staging_codes_with_vector_and_config(
        codes,
        store,
        vector_config,
        embedding,
        top_k,
        &mapping_config,
    )
}

pub fn map_staging_codes_with_vector_and_config<I, S, E>(
    codes: I,
    store: Arc<S>,
    vector_config: VectorStoreConfig,
    embedding: E,
    top_k: usize,
    mapping_config: &MappingConfig,
) -> Result<
    (
        Vec<MappingResult>,
        Vec<DimNCITConcept>,
        MappingSummary,
        dfps_vector_store::VectorUsageSnapshot,
    ),
    VectorRankerError,
>
where
    I: IntoIterator<Item = StgSrCodeExploded>,
    S: VectorStore,
    E: EmbeddingProvider<CodeElement>,
{
    let dim_concepts = dim_concepts();
    let xrefs = load_umls_xrefs();
    let vector_ranker = VectorRankerBackend::from_config(store, embedding, vector_config, top_k)?;
    let usage_handle = vector_ranker.usage_handle();
    let engine = MappingEngine::new(LexicalRanker, vector_ranker, RuleReranker);
    let (results, summary) = crate::pipelines::staging::map_with_engine(
        codes.into_iter(),
        &engine,
        &xrefs,
        None as Option<&dyn dfps_terminology::TerminologyClient>,
        mapping_config,
    );
    let snapshot = usage_handle.snapshot();
    Ok((results, dim_concepts, summary, snapshot))
}
