//! End-to-end pipeline facade stitching together FHIR ingestion and NCIt mapping.
//!
//! Matches the flows in:
//! - docs/system-design/fhir/index.md#quickstart
//! - docs/system-design/ncit/behavior/sequence-servicerequest.md
//! - lib/domain/pipeline/README.md (REFR-09 notes)
//! - lib/domain/contracts (dfps_contracts) for cross-surface DTO alignment
//!   by exposing a single entrypoint from Bundle -> staging -> NCIt concepts,
//!   with optional vector-store contexts injected by callers.

use dfps_core::{
    fhir::Bundle,
    mapping::{DimNCITConcept, MappingResult},
    staging::{StgServiceRequestFlat, StgSrCodeExploded},
};
use dfps_ingestion::{
    ExternalValidationContext, ValidatedBundle, ValidationMode, bundle_to_staging_from_validated,
    bundle_to_staging_with_validation, validation::ValidationReport,
};
use dfps_mapping::{
    DeterministicEmbeddingProvider, map_staging_codes, map_staging_codes_with_vector,
};
use dfps_observability::{PipelineMetrics, VectorUsageSnapshot};
use dfps_vector_port::{
    VectorBackend, VectorItem, VectorSearchResult, VectorStore, VectorStoreConfig, VectorStoreError,
};
use log::warn;
use std::sync::Arc;
use thiserror::Error;

/// Aggregated pipeline output for a single Bundle ingestion/mapping run.
///
/// This type is re-exported in `dfps_contracts` so app surfaces can rely on a
/// single schema without bespoke DTOs.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PipelineOutput {
    pub flats: Vec<StgServiceRequestFlat>,
    pub exploded_codes: Vec<StgSrCodeExploded>,
    pub mapping_results: Vec<MappingResult>,
    pub dim_concepts: Vec<DimNCITConcept>,
    pub vector_usage: Option<VectorUsageSnapshot>,
}

/// Combined pipeline output + validation metadata for a Bundle.
#[derive(Debug)]
pub struct PipelineExecution {
    pub output: PipelineOutput,
    pub validation: ValidationReport,
    pub metrics: PipelineMetrics,
}

/// Runtime toggles for a pipeline run.
#[derive(Clone, Copy, Debug)]
pub struct PipelineRunConfig<'a> {
    pub validation_mode: ValidationMode,
    pub external_validation: ExternalValidationContext<'a>,
    pub mapping: MappingRunConfig,
}

impl<'a> Default for PipelineRunConfig<'a> {
    fn default() -> Self {
        Self {
            validation_mode: ValidationMode::default(),
            external_validation: ExternalValidationContext::default(),
            mapping: MappingRunConfig::default(),
        }
    }
}

impl<'a> PipelineRunConfig<'a> {
    /// Override the ingestion validation mode (Strict/Lenient/External*).
    pub fn with_validation_mode(mut self, mode: ValidationMode) -> Self {
        self.validation_mode = mode;
        self
    }

    /// Inject an external validator/profile context.
    pub fn with_external_validation(mut self, context: ExternalValidationContext<'a>) -> Self {
        self.external_validation = context;
        self
    }

    /// Override mapping behavior (lexical-only, future knobs).
    pub fn with_mapping_config(mut self, mapping: MappingRunConfig) -> Self {
        self.mapping = mapping;
        self
    }
}

/// Mapping-specific configuration knobs (lexical/vector hints).
#[derive(Clone, Copy, Debug)]
pub struct MappingRunConfig {
    pub lexical_only: bool,
}

impl MappingRunConfig {
    /// Force lexical mapping even if a vector context is provided.
    pub fn lexical_only() -> Self {
        Self { lexical_only: true }
    }

    /// Explicitly toggle lexical-only mode.
    pub fn with_lexical_only(mut self, enabled: bool) -> Self {
        self.lexical_only = enabled;
        self
    }
}

impl Default for MappingRunConfig {
    fn default() -> Self {
        Self {
            lexical_only: false,
        }
    }
}

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("ingestion error: {0}")]
    Ingestion(#[from] dfps_ingestion::IngestionError),
}

/// Port trait for application layers to orchestrate Bundle -> PipelineOutput flows.
pub trait PipelinePort {
    fn map_bundle(
        &self,
        bundle: &Bundle,
        config: &PipelineRunConfig<'_>,
        vector: Option<&VectorPipelineContext>,
    ) -> Result<PipelineOutput, PipelineError> {
        self.map_bundle_with_validation(bundle, config, vector)
            .map(|exec| exec.output)
    }

    fn map_bundle_with_validation(
        &self,
        bundle: &Bundle,
        config: &PipelineRunConfig<'_>,
        vector: Option<&VectorPipelineContext>,
    ) -> Result<PipelineExecution, PipelineError>;

    fn map_validated_bundle(
        &self,
        bundle: &ValidatedBundle,
        config: &PipelineRunConfig<'_>,
        vector: Option<&VectorPipelineContext>,
    ) -> Result<PipelineExecution, PipelineError> {
        bundle_to_mapped_sr_from_validated_bundle(bundle, config, vector)
    }
}

/// Default orchestrator implementing [`PipelinePort`].
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultPipeline;

impl PipelinePort for DefaultPipeline {
    fn map_bundle_with_validation(
        &self,
        bundle: &Bundle,
        config: &PipelineRunConfig<'_>,
        vector: Option<&VectorPipelineContext>,
    ) -> Result<PipelineExecution, PipelineError> {
        bundle_to_mapped_sr_with_validation(bundle, config, vector)
    }
}

pub fn bundle_to_mapped_sr(bundle: &Bundle) -> Result<PipelineOutput, PipelineError> {
    bundle_to_mapped_sr_with_validation(bundle, &PipelineRunConfig::default(), None)
        .map(|exec| exec.output)
}

pub fn bundle_to_mapped_sr_with_vector_context(
    bundle: &Bundle,
    vector: Option<&VectorPipelineContext>,
) -> Result<PipelineOutput, PipelineError> {
    bundle_to_mapped_sr_with_validation(bundle, &PipelineRunConfig::default(), vector)
        .map(|exec| exec.output)
}

/// Run the pipeline with explicit config + optional vector context, returning validation details.
pub fn bundle_to_mapped_sr_with_validation<'a>(
    bundle: &Bundle,
    config: &PipelineRunConfig<'a>,
    vector: Option<&VectorPipelineContext>,
) -> Result<PipelineExecution, PipelineError> {
    let (ValidatedStage { flats, exploded }, validation) = staging_rows(bundle, config)?;
    let (mapping_results, dim_concepts, usage) = if config.mapping.lexical_only {
        let (results, dims) = map_staging_codes(exploded.clone());
        (results, dims, None)
    } else {
        vector
            .and_then(|ctx| try_vector_mapping(&exploded, ctx))
            .unwrap_or_else(|| {
                let (results, dims) = map_staging_codes(exploded.clone());
                (results, dims, None)
            })
    };
    let metrics = summarize_metrics(&flats, &exploded, &mapping_results, usage.as_ref());

    Ok(PipelineExecution {
        output: PipelineOutput {
            flats,
            exploded_codes: exploded,
            mapping_results,
            dim_concepts,
            vector_usage: usage,
        },
        validation,
        metrics,
    })
}

/// Run the pipeline using a pre-validated bundle to avoid re-running validation.
pub fn bundle_to_mapped_sr_from_validated_bundle(
    bundle: &ValidatedBundle,
    config: &PipelineRunConfig<'_>,
    vector: Option<&VectorPipelineContext>,
) -> Result<PipelineExecution, PipelineError> {
    let (ValidatedStage { flats, exploded }, validation) = staging_rows_from_validated(bundle)?;
    let (mapping_results, dim_concepts, usage) = if config.mapping.lexical_only {
        let (results, dims) = map_staging_codes(exploded.clone());
        (results, dims, None)
    } else {
        vector
            .and_then(|ctx| try_vector_mapping(&exploded, ctx))
            .unwrap_or_else(|| {
                let (results, dims) = map_staging_codes(exploded.clone());
                (results, dims, None)
            })
    };
    let metrics = summarize_metrics(&flats, &exploded, &mapping_results, usage.as_ref());

    Ok(PipelineExecution {
        output: PipelineOutput {
            flats,
            exploded_codes: exploded,
            mapping_results,
            dim_concepts,
            vector_usage: usage,
        },
        validation,
        metrics,
    })
}

struct ValidatedStage {
    flats: Vec<StgServiceRequestFlat>,
    exploded: Vec<StgSrCodeExploded>,
}

fn staging_rows(
    bundle: &Bundle,
    config: &PipelineRunConfig<'_>,
) -> Result<(ValidatedStage, ValidationReport), PipelineError> {
    let validated = bundle_to_staging_with_validation(
        bundle,
        config.validation_mode,
        config.external_validation,
    )?;
    let (flats, exploded) = validated.value;
    Ok((ValidatedStage { flats, exploded }, validated.report))
}

fn staging_rows_from_validated(
    bundle: &ValidatedBundle,
) -> Result<(ValidatedStage, ValidationReport), PipelineError> {
    let validated = bundle_to_staging_from_validated(bundle)?;
    let (flats, exploded) = validated.value;
    Ok((ValidatedStage { flats, exploded }, validated.report))
}

fn summarize_metrics(
    flats: &[StgServiceRequestFlat],
    exploded: &[StgSrCodeExploded],
    mappings: &[MappingResult],
    usage: Option<&VectorUsageSnapshot>,
) -> PipelineMetrics {
    let mut metrics = PipelineMetrics::default();
    metrics.record(flats, exploded, mappings);
    if let Some(snapshot) = usage {
        metrics.record_vector_usage(snapshot.clone(), None);
    }
    metrics
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

/// Caller-provided vector configuration + store handle.
///
/// Construct this in app/platform crates (after reading env/config) and pass a
/// borrowed reference into the pipeline. The context is intentionally opaque so
/// dfps_pipeline stays environment-free.
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
    use dfps_ingestion::{ExternalValidationContext, ValidatedBundle, ValidationMode};
    use dfps_test_suite::regression;
    use dfps_vector_port::{MockVectorStore, VectorBackend};
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

    #[test]
    fn pipeline_run_config_customization_executes() {
        let bundle = regression::baseline_fhir_bundle();
        let config = PipelineRunConfig::default().with_validation_mode(ValidationMode::Strict);
        let output = bundle_to_mapped_sr_with_validation(&bundle, &config, None)
            .expect("strict lexical run")
            .output;
        assert!(!output.mapping_results.is_empty());
    }

    #[test]
    fn mapping_config_can_force_lexical_path() {
        let bundle = regression::baseline_fhir_bundle();
        let ctx = mock_vector_context();
        let config =
            PipelineRunConfig::default().with_mapping_config(MappingRunConfig::lexical_only());
        let output = bundle_to_mapped_sr_with_validation(&bundle, &config, Some(&ctx))
            .expect("lexical override")
            .output;
        assert!(
            output.vector_usage.is_none(),
            "lexical-only mapping should ignore vector contexts"
        );
    }

    #[test]
    fn validated_bundle_path_matches_regular_execution() {
        let bundle = regression::baseline_fhir_bundle();
        let validated = ValidatedBundle::try_new(
            bundle.clone(),
            ValidationMode::Lenient,
            ExternalValidationContext::default(),
        )
        .expect("validated bundle");
        let config = PipelineRunConfig::default();
        let regular =
            bundle_to_mapped_sr_with_validation(&bundle, &config, None).expect("regular execution");
        let reused = bundle_to_mapped_sr_from_validated_bundle(&validated, &config, None)
            .expect("validated execution");
        assert_eq!(regular.output.flats, reused.output.flats);
        assert_eq!(regular.output.exploded_codes, reused.output.exploded_codes);
        assert_eq!(
            regular.output.mapping_results,
            reused.output.mapping_results
        );
        assert_eq!(regular.validation.issues, reused.validation.issues);
    }

    #[test]
    fn validation_report_is_returned() {
        let bundle = regression::baseline_fhir_bundle();
        let exec =
            bundle_to_mapped_sr_with_validation(&bundle, &PipelineRunConfig::default(), None)
                .expect("pipeline run");
        assert_eq!(exec.validation.issues.len(), 0);
        assert!(!exec.output.mapping_results.is_empty());
    }
}
