//! Mesh node data-plane orchestration shared by HTTP adapters.
//!
//! See:
//! - docs/system-design/mesh/node-runtime.md
//! - docs/kanban/feature/mvp/040-infra-and-docs/030-mesh-node-hub-propagation.md

use std::sync::Arc;

use refractive_swan_compliance::{ComplianceAction, Policy};
use refractive_swan_contracts::MeshJobDescriptor;
use refractive_swan_contracts::eval::EvalRunResponse;
use refractive_swan_contracts::{
    ExportJobSummary, MeshError, MeshErrorCode, MeshErrorKind, MeshJobResult, MeshJobStatus,
    MeshJobType, NodeIntrospectionView,
};
use refractive_swan_datamart::{DatamartSink, SqliteDatamart, WarehouseConfig};
use refractive_swan_datamart_port::CohortFilters;
use refractive_swan_eval::DatasetStore;
use refractive_swan_eval::fake_data::fixtures::{self, Registry};
use refractive_swan_mesh_dto::{MeshNodeId, NodeCapabilities};
use refractive_swan_observability::PipelineMetrics;
use refractive_swan_observability::metrics_snapshot_json;
use refractive_swan_pipeline::{
    DefaultPipeline, PipelinePort, PipelineRunConfig, VectorPipelineContext,
    run_eval_dataset_with_pipeline,
};
use refractive_swan_vector_port::VectorBackend;
use refractive_swan_vector_store::VectorStoreConfig;
use serde_json::{Value, json};
use tokio::sync::Mutex;

use crate::lake::{LakeReader, LakeWriter};
use crate::{config::NodePlaneConfig, vector::vector_context_from_config};

/// Shared orchestration surface for node-local data plane operations.
///
/// This struct wires the domain pipeline, datamart sink, dataset store, and
/// optional vector context together so HTTP/gRPC adapters can remain thin.
pub struct NodeDataPlane {
    node_id: MeshNodeId,
    policy: Policy,
    vector_context: Option<VectorPipelineContext>,
    vector_config: Option<VectorStoreConfig>,
    dataset_store: Arc<dyn DatasetStore + Send + Sync>,
    metrics: Arc<Mutex<PipelineMetrics>>,
    pipeline: Arc<dyn PipelinePort + Send + Sync>,
    datamart: Arc<dyn DatamartSink + Send + Sync>,
    datamart_config: Option<WarehouseConfig>,
    #[allow(dead_code)]
    lake_writer: Option<Arc<dyn LakeWriter + Send + Sync>>,
    #[allow(dead_code)]
    lake_reader: Option<Arc<dyn LakeReader + Send + Sync>>,
    max_dataset_size: u64,
    tags: Vec<String>,
}

impl NodeDataPlane {
    /// Build a plane using defaults for pipeline/datamart/vector contexts.
    pub fn from_config(config: NodePlaneConfig) -> Self {
        let NodePlaneConfig {
            node_id,
            policy,
            dataset_store,
            datamart,
            vector,
            max_dataset_size,
            tags,
        } = config;
        let dataset_store: Arc<dyn DatasetStore + Send + Sync> = Arc::new(dataset_store);
        let vector_config = vector.clone();
        let vector_context = vector.as_ref().and_then(vector_context_from_config);
        let pipeline: Arc<dyn PipelinePort + Send + Sync> = Arc::new(DefaultPipeline);
        let datamart_config = datamart.clone();
        let datamart: Arc<dyn DatamartSink + Send + Sync> =
            Arc::new(SqliteDatamart::from_optional_config(datamart));
        Self::new(
            node_id,
            policy,
            dataset_store,
            pipeline,
            datamart,
            vector_context,
            vector_config,
            datamart_config,
            max_dataset_size,
            tags,
            None,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        node_id: MeshNodeId,
        policy: Policy,
        dataset_store: Arc<dyn DatasetStore + Send + Sync>,
        pipeline: Arc<dyn PipelinePort + Send + Sync>,
        datamart: Arc<dyn DatamartSink + Send + Sync>,
        vector_context: Option<VectorPipelineContext>,
        vector_config: Option<VectorStoreConfig>,
        datamart_config: Option<WarehouseConfig>,
        max_dataset_size: u64,
        tags: Vec<String>,
        lake_writer: Option<Arc<dyn LakeWriter + Send + Sync>>,
        lake_reader: Option<Arc<dyn LakeReader + Send + Sync>>,
    ) -> Self {
        let node_id_str = node_id.to_string();
        let compliance_mode = policy.mode.as_str().to_string();
        Self {
            node_id,
            policy,
            vector_context,
            vector_config,
            dataset_store,
            metrics: Arc::new(Mutex::new(PipelineMetrics {
                mesh_node_id: Some(node_id_str),
                compliance_mode: Some(compliance_mode),
                ..PipelineMetrics::default()
            })),
            pipeline,
            datamart,
            datamart_config,
            lake_writer,
            lake_reader,
            max_dataset_size,
            tags,
        }
    }

    /// Node identifier advertised to the mesh hub.
    pub fn node_id(&self) -> &MeshNodeId {
        &self.node_id
    }

    /// Compliance policy enforced for pipeline + analytics jobs.
    pub fn policy(&self) -> &Policy {
        &self.policy
    }

    /// Mutable access to the compliance policy (tests/config updates).
    pub fn policy_mut(&mut self) -> &mut Policy {
        &mut self.policy
    }

    /// Optional vector context supplied by the platform/store layer.
    pub fn vector_context(&self) -> Option<VectorPipelineContext> {
        self.vector_context.clone()
    }

    /// Cloneable metrics handle for HTTP adapters.
    pub fn metrics(&self) -> Arc<Mutex<PipelineMetrics>> {
        Arc::clone(&self.metrics)
    }

    pub fn dataset_store(&self) -> Arc<dyn DatasetStore + Send + Sync> {
        Arc::clone(&self.dataset_store)
    }

    pub fn datamart(&self) -> Arc<dyn DatamartSink + Send + Sync> {
        Arc::clone(&self.datamart)
    }

    pub fn pipeline(&self) -> Arc<dyn PipelinePort + Send + Sync> {
        Arc::clone(&self.pipeline)
    }

    /// Node capabilities advertised to hubs or dashboards.
    pub fn capabilities(&self, _base_url: &str) -> NodeCapabilities {
        NodeCapabilities {
            node_id: self.node_id.clone(),
            vector_backend: self.vector_backend_label(),
            warehouse_backend: self.warehouse_backend_label(),
            compliance_mode: self.policy.mode.as_str().to_string(),
            max_dataset_size: self.max_dataset_size,
            tags: self.tags.clone(),
        }
    }

    fn vector_backend_label(&self) -> String {
        match self.vector_config.as_ref().map(|cfg| cfg.backend.clone()) {
            Some(VectorBackend::Qdrant) => "qdrant".into(),
            Some(VectorBackend::PgVector) => "pgvector".into(),
            Some(VectorBackend::Milvus) => "milvus".into(),
            Some(VectorBackend::Mock) => "mock".into(),
            None => "disabled".into(),
        }
    }

    fn warehouse_backend_label(&self) -> String {
        match &self.datamart_config {
            Some(cfg) => {
                let url = cfg.url.to_ascii_lowercase();
                if url.starts_with("sqlite://") || url.starts_with("sqlite:") {
                    "sqlite".into()
                } else if url.starts_with("postgres://") || url.starts_with("postgresql://") {
                    "postgres".into()
                } else {
                    "external".into()
                }
            }
            None => "disabled".into(),
        }
    }

    /// Execute a mesh job descriptor and return a structured result.
    pub async fn run_mesh_job(&self, job: &MeshJobDescriptor) -> MeshJobResult {
        match job.job_type {
            MeshJobType::EvalDataset => self
                .run_eval_dataset_job(job)
                .await
                .unwrap_or_else(|err| self.failure(job, err)),
            MeshJobType::AnalyticsQuery => self
                .run_analytics_job(job)
                .await
                .unwrap_or_else(|err| self.failure(job, err)),
            MeshJobType::MappingHealthCheck => self.mapping_health(job).await,
            MeshJobType::ExportJob => self.run_export_job(job).await,
            MeshJobType::NodeIntrospection => self.node_introspection(job).await,
        }
    }

    async fn run_eval_dataset_job(
        &self,
        job: &MeshJobDescriptor,
    ) -> Result<MeshJobResult, MeshError> {
        let dataset = parse_dataset_name(&job.parameters).ok_or_else(|| {
            MeshError::new(
                MeshErrorKind::InvalidRequest,
                MeshErrorCode::new("invalid_request:missing_dataset"),
                "dataset parameter is required".into(),
            )
        })?;
        let outcome = run_eval_dataset_with_pipeline(
            self.dataset_store.as_ref(),
            &dataset,
            self.vector_context.as_ref(),
        )
        .map_err(|err| {
            MeshError::new(
                MeshErrorKind::Internal,
                MeshErrorCode::new("internal:eval_dataset"),
                err.to_string(),
            )
        })?;

        let response = EvalRunResponse {
            dataset: outcome.dataset,
            manifest: Some(outcome.manifest),
            summary: outcome.summary,
        };

        let metrics = self.metrics_snapshot_json().await;
        Ok(MeshJobResult {
            job_id: job.job_id.clone(),
            status: MeshJobStatus::Success,
            metrics: Some(metrics),
            output: Some(serde_json::to_value(response).expect("serialize eval response")),
            error: None,
        })
    }

    async fn run_analytics_job(&self, job: &MeshJobDescriptor) -> Result<MeshJobResult, MeshError> {
        let query_type = job
            .parameters
            .get("query_type")
            .and_then(|v| v.as_str())
            .unwrap_or("ncit_summary");

        let output = match query_type {
            "ncit_summary" => {
                let summary = refractive_swan_datamart::node_ncit_summary(self.datamart.as_ref())
                    .await
                    .map_err(|err| {
                        MeshError::new(
                            MeshErrorKind::Internal,
                            MeshErrorCode::new("internal:ncit_summary"),
                            err.to_string(),
                        )
                    })?;
                serde_json::to_value(summary).expect("serialize summary")
            }
            "cohort" => {
                let filters: CohortFilters =
                    serde_json::from_value(job.parameters.clone()).unwrap_or_default();
                let cohort =
                    refractive_swan_datamart::node_cohort(self.datamart.as_ref(), &filters)
                        .await
                        .map_err(|err| {
                            MeshError::new(
                                MeshErrorKind::Internal,
                                MeshErrorCode::new("internal:cohort"),
                                err.to_string(),
                            )
                        })?;
                serde_json::to_value(cohort).expect("serialize cohort")
            }
            other => {
                return Err(MeshError::new(
                    MeshErrorKind::InvalidRequest,
                    MeshErrorCode::new("invalid_request:unknown_query"),
                    format!("unsupported analytics query_type '{other}'"),
                ));
            }
        };

        let metrics = self.metrics_snapshot_json().await;
        Ok(MeshJobResult {
            job_id: job.job_id.clone(),
            status: MeshJobStatus::Success,
            metrics: Some(metrics),
            output: Some(output),
            error: None,
        })
    }

    async fn mapping_health(&self, job: &MeshJobDescriptor) -> MeshJobResult {
        let bundle = match fixtures::bundles::load(&Registry::default(), "fhir_bundle_sr") {
            Ok(bundle) => bundle,
            Err(err) => {
                return self.failure(
                    job,
                    MeshError::new(
                        MeshErrorKind::Internal,
                        MeshErrorCode::new("internal:health_fixture"),
                        format!("failed to load regression bundle: {err}"),
                    ),
                );
            }
        };

        let pipeline_result = self.pipeline.map_bundle_with_validation(
            &bundle,
            &PipelineRunConfig::default(),
            self.vector_context.as_ref(),
        );

        let execution = match pipeline_result {
            Ok(exec) => exec,
            Err(err) => {
                return self.failure(
                    job,
                    MeshError::new(
                        MeshErrorKind::Internal,
                        MeshErrorCode::new("internal:mapping_health"),
                        err.to_string(),
                    ),
                );
            }
        };

        let mut metrics_handle = self.metrics.lock().await;
        metrics_handle.record(
            &execution.output.flats,
            &execution.output.exploded_codes,
            &execution.output.mapping_results,
        );
        if let Some(usage) = execution.output.vector_usage.clone() {
            metrics_handle.record_vector_usage(usage, None);
        }
        drop(metrics_handle);

        let _persist = self
            .datamart
            .persist(&execution.output, &self.policy)
            .await
            .ok();
        let vector_health = match self.vector_context.as_ref() {
            Some(ctx) => {
                let status = ctx.health().map(|_| "ok").unwrap_or("error");
                json!({
                    "status": status,
                    "backend": self.vector_backend_label(),
                    "namespace": ctx.namespace(),
                })
            }
            None => json!({ "status": "disabled", "backend": "disabled" }),
        };
        let warehouse_health = match self.datamart.ncit_summary().await {
            Ok(_) => json!({ "status": "ok" }),
            Err(err) => json!({ "status": "error", "message": err.to_string() }),
        };

        let counts = state_counts(&execution.output.mapping_results);
        let metrics = self.metrics_snapshot_json().await;
        let output = json!({
            "status": "ok",
            "vector_backend": self.vector_backend_label(),
            "warehouse_backend": self.warehouse_backend_label(),
            "vector_health": vector_health,
            "warehouse_health": warehouse_health,
            "datamart_persisted": _persist.is_some(),
            "auto_mapped": counts.auto_mapped,
            "needs_review": counts.needs_review,
            "no_match": counts.no_match,
            "flats": execution.output.flats.len(),
            "exploded": execution.output.exploded_codes.len(),
        });

        MeshJobResult {
            job_id: job.job_id.clone(),
            status: MeshJobStatus::Success,
            metrics: Some(metrics),
            output: Some(output),
            error: None,
        }
    }

    async fn run_export_job(&self, job: &MeshJobDescriptor) -> MeshJobResult {
        if !self.policy.is_action_allowed(ComplianceAction::Export) {
            return self.denied(
                job,
                MeshError::new(
                    MeshErrorKind::PolicyDenied,
                    MeshErrorCode::new("policy_denied:export"),
                    "export blocked by compliance policy".into(),
                ),
            );
        }
        let summary = ExportJobSummary {
            export: "ncit_summary".into(),
            load_summary: refractive_swan_contracts::LoadSummary::default(),
            dp_epsilon_used: None,
        };
        let metrics = self.metrics_snapshot_json().await;
        MeshJobResult {
            job_id: job.job_id.clone(),
            status: MeshJobStatus::Success,
            metrics: Some(metrics),
            output: Some(serde_json::to_value(summary).expect("serialize export summary")),
            error: None,
        }
    }

    async fn node_introspection(&self, job: &MeshJobDescriptor) -> MeshJobResult {
        let metrics = self.metrics_snapshot_json().await;
        let datasets = self
            .dataset_store
            .list_manifests()
            .unwrap_or_default()
            .into_iter()
            .map(|m| m.name)
            .collect();
        let output = serde_json::to_value(NodeIntrospectionView {
            metrics: metrics.clone(),
            vector_usage: self.vector_context.as_ref().map(|ctx| {
                json!({ "backend": self.vector_backend_label(), "namespace": ctx.namespace() })
            }),
            datasets,
            features: self.tags.clone(),
        })
        .unwrap_or_else(|_| metrics.clone());
        MeshJobResult {
            job_id: job.job_id.clone(),
            status: MeshJobStatus::Success,
            metrics: Some(metrics),
            output: Some(output),
            error: None,
        }
    }

    fn failure(&self, job: &MeshJobDescriptor, error: MeshError) -> MeshJobResult {
        MeshJobResult {
            job_id: job.job_id.clone(),
            status: MeshJobStatus::Failed,
            metrics: None,
            output: None,
            error: Some(error),
        }
    }

    fn denied(&self, job: &MeshJobDescriptor, error: MeshError) -> MeshJobResult {
        MeshJobResult {
            job_id: job.job_id.clone(),
            status: MeshJobStatus::Denied,
            metrics: None,
            output: None,
            error: Some(error),
        }
    }

    async fn metrics_snapshot_json(&self) -> Value {
        let metrics = self.metrics.lock().await.clone();
        metrics_snapshot_json(&metrics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use refractive_swan_compliance::ComplianceMode;
    use std::fs;

    #[test]
    fn plane_respects_prebuilt_policy_in_config() {
        let temp_root = std::env::temp_dir().join("mesh-node-policy-test");
        let _ = fs::create_dir_all(&temp_root);
        let policy = Policy::default_for_mode(ComplianceMode::OpenSource);
        let config = NodePlaneConfig {
            node_id: MeshNodeId::from_string("node-test".into()),
            policy: policy.clone(),
            dataset_store: refractive_swan_eval::FileDatasetStore::new(&temp_root),
            datamart: None,
            vector: None,
            max_dataset_size: 1,
            tags: vec![],
        };
        let plane = NodeDataPlane::from_config(config);
        assert_eq!(plane.policy.mode, policy.mode);
    }

    #[test]
    fn warehouse_backend_label_reflects_datamart_config() {
        let temp_root = std::env::temp_dir().join("mesh-node-datamart-test");
        let _ = fs::create_dir_all(&temp_root);
        let base_config = NodePlaneConfig {
            node_id: MeshNodeId::from_string("node-test".into()),
            policy: Policy::default_for_mode(ComplianceMode::OpenSource),
            dataset_store: refractive_swan_eval::FileDatasetStore::new(&temp_root),
            datamart: None,
            vector: None,
            max_dataset_size: 1,
            tags: vec![],
        };
        let disabled = NodeDataPlane::from_config(base_config.clone());
        assert_eq!(disabled.warehouse_backend_label(), "disabled");

        let cfg = refractive_swan_datamart::WarehouseConfig {
            url: "sqlite://test.db".into(),
            schema: None,
            max_connections: 5,
        };
        let enabled_plane = NodeDataPlane::from_config(NodePlaneConfig {
            datamart: Some(cfg),
            ..base_config
        });
        assert_eq!(enabled_plane.warehouse_backend_label(), "sqlite");
    }
}

fn parse_dataset_name(value: &Value) -> Option<String> {
    value
        .get("dataset")
        .or_else(|| value.get("dataset_name"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

#[derive(Default)]
struct MappingStateCounts {
    auto_mapped: usize,
    needs_review: usize,
    no_match: usize,
}

fn state_counts(results: &[refractive_swan_core::mapping::MappingResult]) -> MappingStateCounts {
    let mut counts = MappingStateCounts::default();
    for result in results {
        match result.state {
            refractive_swan_core::mapping::MappingState::AutoMapped => counts.auto_mapped += 1,
            refractive_swan_core::mapping::MappingState::NeedsReview => counts.needs_review += 1,
            refractive_swan_core::mapping::MappingState::NoMatch => counts.no_match += 1,
        }
    }
    counts
}
