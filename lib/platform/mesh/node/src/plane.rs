//! Mesh node data-plane orchestration shared by HTTP adapters.
//!
//! See:
//! - docs/system-design/mesh/node-runtime.md
//! - docs/kanban/feature/mvp/040-infra-and-docs/030-mesh-node-hub-propagation.md

use std::{sync::Arc, time::Duration};

use chrono::Utc;
use log::warn;
use refractive_swan_cache_store::{CacheBackend, CacheStore, cache_from_config};
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
use refractive_swan_terminology::codesystem::LicenseTier;
use refractive_swan_vector_port::VectorBackend;
use refractive_swan_vector_store::VectorStoreConfig;
use serde_json::{Value, json};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Sqlite};
use tokio::sync::{Mutex, OnceCell};

use crate::governance::{GovernanceDecision, GovernanceEngine, NodePolicy, QueryDescriptor};
use crate::lake::{LakeReader, LakeWriter};
use crate::{config::NodePlaneConfig, vector::vector_context_from_config};

const DP_BUDGET_CACHE_SCALE: f64 = 1_000.0;
const DP_BUDGET_TTL: Duration = Duration::from_secs(86_400);

enum DpLedgerBackend {
    Sqlite(WarehouseConfig),
    Unsupported(&'static str),
}

pub struct DpBudgetLedger {
    backend: DpLedgerBackend,
    pool: OnceCell<Pool<Sqlite>>,
}

impl DpBudgetLedger {
    fn new(config: WarehouseConfig) -> Option<Self> {
        let url = config.url.to_ascii_lowercase();
        if url.starts_with("sqlite://") || url.starts_with("sqlite:") {
            Some(Self {
                backend: DpLedgerBackend::Sqlite(config),
                pool: OnceCell::new(),
            })
        } else if url.starts_with("postgres://") || url.starts_with("postgresql://") {
            Some(Self {
                backend: DpLedgerBackend::Unsupported("postgres"),
                pool: OnceCell::new(),
            })
        } else if url.starts_with("duckdb://") || url.contains("duckdb") {
            Some(Self {
                backend: DpLedgerBackend::Unsupported("duckdb"),
                pool: OnceCell::new(),
            })
        } else {
            None
        }
    }

    async fn pool(&self) -> Result<&Pool<Sqlite>, sqlx::Error> {
        match &self.backend {
            DpLedgerBackend::Sqlite(config) => {
                self.pool
                    .get_or_try_init(|| async {
                        SqlitePoolOptions::new()
                            .max_connections(config.max_connections)
                            .connect(&config.url)
                            .await
                    })
                    .await
            }
            DpLedgerBackend::Unsupported(_) => Err(sqlx::Error::Configuration(
                "ledger backend unsupported".into(),
            )),
        }
    }

    async fn persist(&self, node_id: &str, date: &str, consumed: f64) -> Result<(), sqlx::Error> {
        match &self.backend {
            DpLedgerBackend::Sqlite(_) => {
                let pool = self.pool().await?;
                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS mesh_dp_budget (
                        node_id TEXT NOT NULL,
                        date TEXT NOT NULL,
                        consumed REAL NOT NULL,
                        updated_at TEXT NOT NULL,
                        PRIMARY KEY (node_id, date)
                    )
                    "#,
                )
                .execute(pool)
                .await?;

                sqlx::query(
                    r#"
                    INSERT INTO mesh_dp_budget (node_id, date, consumed, updated_at)
                    VALUES (?1, ?2, ?3, datetime('now'))
                    ON CONFLICT(node_id, date)
                    DO UPDATE SET consumed = excluded.consumed, updated_at = excluded.updated_at
                    "#,
                )
                .bind(node_id)
                .bind(date)
                .bind(consumed)
                .execute(pool)
                .await?;
                Ok(())
            }
            DpLedgerBackend::Unsupported(backend) => {
                warn!(
                    "dp_budget ledger backend {backend} not yet implemented; skipping persistence"
                );
                Ok(())
            }
        }
    }
}

/// Shared orchestration surface for node-local data plane operations.
///
/// This struct wires the domain pipeline, datamart sink, dataset store, and
/// optional vector context together so HTTP/gRPC adapters can remain thin.
pub struct NodeDataPlane {
    node_id: MeshNodeId,
    policy: Policy,
    vector_context: Option<VectorPipelineContext>,
    vector_config: Option<VectorStoreConfig>,
    cache: Option<Arc<dyn CacheStore + Send + Sync>>,
    cache_backend: CacheBackend,
    dp_ledger: Option<DpBudgetLedger>,
    dataset_store: Arc<dyn DatasetStore + Send + Sync>,
    metrics: Arc<Mutex<PipelineMetrics>>,
    pipeline: Arc<dyn PipelinePort + Send + Sync>,
    datamart: Arc<dyn DatamartSink + Send + Sync>,
    datamart_config: Option<WarehouseConfig>,
    #[allow(dead_code)]
    lake_writer: Option<Arc<dyn LakeWriter + Send + Sync>>,
    #[allow(dead_code)]
    lake_reader: Option<Arc<dyn LakeReader + Send + Sync>>,
    node_policy: Option<Arc<Mutex<NodePolicy>>>,
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
            cache,
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
        let node_policy = Some(Arc::new(Mutex::new(NodePolicy::from_env())));
        let dp_ledger = datamart_config.clone().and_then(DpBudgetLedger::new);
        let (cache_backend, cache_store) = match cache.as_ref() {
            Some(cfg) => match cache_from_config(cfg) {
                Ok(store) => (cfg.backend.clone(), store),
                Err(err) => {
                    eprintln!(
                        "refractive_swan_mesh_node: cache backend {:?} unavailable: {err}",
                        cfg.backend
                    );
                    (cfg.backend.clone(), None)
                }
            },
            None => (CacheBackend::Disabled, None),
        };
        Self::new(
            node_id,
            policy,
            dataset_store,
            pipeline,
            datamart,
            vector_context,
            vector_config,
            datamart_config,
            dp_ledger,
            cache_store,
            cache_backend,
            max_dataset_size,
            tags,
            None,
            None,
            node_policy,
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
        dp_ledger: Option<DpBudgetLedger>,
        cache: Option<Arc<dyn CacheStore + Send + Sync>>,
        cache_backend: CacheBackend,
        max_dataset_size: u64,
        tags: Vec<String>,
        lake_writer: Option<Arc<dyn LakeWriter + Send + Sync>>,
        lake_reader: Option<Arc<dyn LakeReader + Send + Sync>>,
        node_policy: Option<Arc<Mutex<NodePolicy>>>,
    ) -> Self {
        let node_id_str = node_id.to_string();
        let compliance_mode = policy.mode.as_str().to_string();
        Self {
            node_id,
            policy,
            vector_context,
            vector_config,
            cache,
            cache_backend,
            dp_ledger,
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
            node_policy,
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

    pub fn cache_backend(&self) -> String {
        self.cache_backend_label()
    }

    fn cache_backend_label(&self) -> String {
        match self.cache_backend {
            CacheBackend::Redis => "redis".into(),
            CacheBackend::InMemory => "inmemory".into(),
            CacheBackend::Disabled => "disabled".into(),
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

        let mut metrics = self.metrics_snapshot_json().await;
        Self::annotate_requester(&mut metrics, job);
        Ok(MeshJobResult {
            job_id: job.job_id.clone(),
            status: MeshJobStatus::Success,
            metrics: Some(metrics),
            output: Some(serde_json::to_value(response).expect("serialize eval response")),
            error: None,
        })
    }

    async fn run_analytics_job(&self, job: &MeshJobDescriptor) -> Result<MeshJobResult, MeshError> {
        let mut dp_applied = false;
        if let Some(decision) = self
            .evaluate_policy(job, Some(crate::governance::QueryClass::AnalyticsJob))
            .await
        {
            match decision {
                GovernanceDecision::Deny(reason) => {
                    let code = policy_error_code("policy_denied:analytics", &reason);
                    return Ok(self.denied(
                        job,
                        MeshError::new(MeshErrorKind::PolicyDenied, code, reason),
                    ));
                }
                GovernanceDecision::AllowWithNoise { epsilon } => {
                    if epsilon > 0.0 && self.node_policy.as_ref().is_some() {
                        dp_applied = true;
                        if let Err(err) = self.consume_budget(epsilon).await {
                            return Ok(self.denied(
                                job,
                                MeshError::new(
                                    MeshErrorKind::PolicyDenied,
                                    MeshErrorCode::new("policy_denied:dp_budget_exceeded"),
                                    err.to_string(),
                                ),
                            ));
                        }
                    }
                }
                GovernanceDecision::Allow => {}
            }
        }
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
                let mut payload = serde_json::to_value(summary).expect("serialize summary");
                if let Some(obj) = payload.as_object_mut() {
                    obj.insert("dp_applied".into(), serde_json::json!(dp_applied));
                }
                payload
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

        let mut metrics = self.metrics_snapshot_json().await;
        Self::annotate_requester(&mut metrics, job);
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
        let cache_health = self.cache_health().await;
        let warehouse_health = match self.datamart.ncit_summary().await {
            Ok(_) => json!({ "status": "ok" }),
            Err(err) => json!({ "status": "error", "message": err.to_string() }),
        };

        let counts = state_counts(&execution.output.mapping_results);
        let mut metrics = self.metrics_snapshot_json().await;
        Self::annotate_requester(&mut metrics, job);
        let output = json!({
            "status": "ok",
            "vector_backend": self.vector_backend_label(),
            "warehouse_backend": self.warehouse_backend_label(),
            "cache_backend": self.cache_backend_label(),
            "vector_health": vector_health,
            "warehouse_health": warehouse_health,
            "cache_health": cache_health,
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
            let code = export_blocked_code(&self.policy);
            return self.denied(
                job,
                MeshError::new(
                    MeshErrorKind::PolicyDenied,
                    code,
                    "export blocked by compliance policy".into(),
                ),
            );
        }
        let mut dp_epsilon_used = None;
        if let Some(decision) = self
            .evaluate_policy(job, Some(crate::governance::QueryClass::ExportJob))
            .await
        {
            match decision {
                GovernanceDecision::Deny(reason) => {
                    let code = policy_error_code("policy_denied:export", &reason);
                    return self.denied(
                        job,
                        MeshError::new(MeshErrorKind::PolicyDenied, code, reason),
                    );
                }
                GovernanceDecision::AllowWithNoise { epsilon } => {
                    dp_epsilon_used = Some(epsilon);
                    if let Err(err) = self.consume_budget(epsilon).await {
                        return self.denied(
                            job,
                            MeshError::new(
                                MeshErrorKind::PolicyDenied,
                                MeshErrorCode::new("policy_denied:dp_budget_exceeded"),
                                err,
                            ),
                        );
                    }
                }
                GovernanceDecision::Allow => {}
            }
        }
        let summary = ExportJobSummary {
            export: "ncit_summary".into(),
            load_summary: refractive_swan_contracts::LoadSummary::default(),
            dp_epsilon_used,
        };
        let mut metrics = self.metrics_snapshot_json().await;
        Self::annotate_requester(&mut metrics, job);
        MeshJobResult {
            job_id: job.job_id.clone(),
            status: MeshJobStatus::Success,
            metrics: Some(metrics),
            output: Some(serde_json::to_value(summary).expect("serialize export summary")),
            error: None,
        }
    }

    async fn node_introspection(&self, job: &MeshJobDescriptor) -> MeshJobResult {
        let mut metrics = self.metrics_snapshot_json().await;
        Self::annotate_requester(&mut metrics, job);
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

    pub async fn cache_health(&self) -> Value {
        match self.cache.as_ref() {
            Some(store) => match store.health_check().await {
                Ok(_) => json!({
                    "status": "ok",
                    "backend": self.cache_backend_label(),
                }),
                Err(err) => json!({
                    "status": "error",
                    "backend": self.cache_backend_label(),
                    "message": err.to_string(),
                }),
            },
            None => json!({
                "status": "disabled",
                "backend": self.cache_backend_label(),
            }),
        }
    }

    async fn evaluate_policy(
        &self,
        job: &MeshJobDescriptor,
        class_override: Option<crate::governance::QueryClass>,
    ) -> Option<GovernanceDecision> {
        let policy = self.node_policy.as_ref()?;
        self.sync_dp_budget_from_cache().await;
        let mut descriptor = QueryDescriptor::from_job(job);
        if let Some(class) = class_override {
            descriptor.class = class;
        }
        let policy_guard = policy.lock().await;
        Some(GovernanceEngine::evaluate(&descriptor, &policy_guard))
    }

    async fn consume_budget(&self, epsilon: f64) -> Result<(), String> {
        if let Some(policy) = self.node_policy.as_ref() {
            let mut guard = policy.lock().await;
            GovernanceEngine::consume_budget(&mut guard, epsilon);
            let exceeded = guard.dp_budget_consumed > guard.dp_budget_daily;
            drop(guard);
            self.persist_dp_budget(epsilon).await;
            if exceeded {
                return Err("dp_budget_exceeded".into());
            }
        }
        Ok(())
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

    async fn sync_dp_budget_from_cache(&self) {
        let (Some(cache), Some(policy)) = (self.cache.as_ref(), self.node_policy.as_ref()) else {
            return;
        };
        let Some(key) = self.dp_budget_cache_key() else {
            return;
        };
        if let Ok(Some(bytes)) = cache.get(&key).await
            && let Ok(text) = String::from_utf8(bytes)
            && let Ok(value) = text.parse::<i64>()
        {
            let mut guard = policy.lock().await;
            guard.dp_budget_consumed = (value as f64) / DP_BUDGET_CACHE_SCALE;
        }
    }

    async fn persist_dp_budget(&self, epsilon: f64) {
        let date = self.dp_budget_date();
        if let Some(cache) = self.cache.as_ref()
            && let Some(key) = self.dp_budget_cache_key_for_date(&date)
        {
            let delta = (epsilon * DP_BUDGET_CACHE_SCALE).round() as i64;
            if let Ok(value) = cache.incr(&key, delta, Some(DP_BUDGET_TTL)).await
                && let Some(policy) = self.node_policy.as_ref()
            {
                let mut guard = policy.lock().await;
                guard.dp_budget_consumed = (value as f64) / DP_BUDGET_CACHE_SCALE;
            }
        }
        if let (Some(ledger), Some(policy)) = (self.dp_ledger.as_ref(), self.node_policy.as_ref()) {
            let consumed = {
                let guard = policy.lock().await;
                guard.dp_budget_consumed
            };
            if let Err(err) = ledger.persist(self.node_id.as_str(), &date, consumed).await {
                warn!(
                    "dp_budget ledger persist failed for node {}: {err}",
                    self.node_id
                );
            }
        }
    }

    fn dp_budget_cache_key(&self) -> Option<String> {
        let date = self.dp_budget_date();
        self.dp_budget_cache_key_for_date(&date)
    }

    fn dp_budget_cache_key_for_date(&self, date: &str) -> Option<String> {
        self.cache.as_ref()?;
        Some(format!("dp_budget:{}:{date}", self.node_id))
    }

    fn dp_budget_date(&self) -> String {
        Utc::now().date_naive().to_string()
    }

    fn annotate_requester(metrics: &mut Value, job: &MeshJobDescriptor) {
        if let Some(requester) = job
            .governance_context
            .as_ref()
            .and_then(|ctx| ctx.get("requester"))
            .and_then(|v| v.as_str())
            && let Some(obj) = metrics.as_object_mut()
        {
            obj.insert("requester".into(), json!(requester));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use refractive_swan_cache_store::CacheBackend;
    use refractive_swan_compliance::ComplianceMode;
    use refractive_swan_contracts::{MeshJobDescriptor, MeshJobStatus, MeshJobType};
    use serde_json::json;
    use std::collections::BTreeSet;
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
            cache: None,
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
            cache: None,
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

    #[tokio::test]
    async fn dp_budget_denial_uses_specific_error_code() {
        let temp_root = std::env::temp_dir().join("mesh-node-dp-budget-test");
        let _ = fs::create_dir_all(&temp_root);
        let policy = Policy::default_for_mode(ComplianceMode::Internal);
        let node_policy = NodePolicy {
            dp_budget_daily: 0.1,
            dp_budget_consumed: 0.1,
            max_cardinality_no_dp: 10,
            export_allowed: true,
            hub_registration_allowed: true,
        };
        let plane = NodeDataPlane::new(
            MeshNodeId::from_string("node-dp".into()),
            policy,
            Arc::new(refractive_swan_eval::FileDatasetStore::new(&temp_root)),
            Arc::new(DefaultPipeline),
            Arc::new(SqliteDatamart::from_optional_config(None)),
            None,
            None,
            None,
            None,
            None,
            CacheBackend::Disabled,
            1,
            vec![],
            None,
            None,
            Some(Arc::new(Mutex::new(node_policy))),
        );
        let descriptor = MeshJobDescriptor {
            job_id: "job-dp".into(),
            job_type: MeshJobType::AnalyticsQuery,
            parameters: json!({ "query_type": "ncit_summary", "expected_cardinality": 1_000 }),
            governance_context: None,
        };
        let result = plane.run_mesh_job(&descriptor).await;
        assert_eq!(result.status, MeshJobStatus::Denied);
        let code = result.error.as_ref().unwrap().code.as_str();
        assert_eq!(code, "policy_denied:dp_budget_exceeded");
    }

    #[tokio::test]
    async fn export_policy_denial_maps_license_tier() {
        let temp_root = std::env::temp_dir().join("mesh-node-export-deny-test");
        let _ = fs::create_dir_all(&temp_root);
        let mut policy = Policy::default_for_mode(ComplianceMode::OpenSource);
        policy.allowed_actions.remove(&ComplianceAction::Export);
        policy
            .allowed_tiers
            .insert(ComplianceAction::Export, BTreeSet::new());
        let plane = NodeDataPlane::new(
            MeshNodeId::from_string("node-export".into()),
            policy,
            Arc::new(refractive_swan_eval::FileDatasetStore::new(&temp_root)),
            Arc::new(DefaultPipeline),
            Arc::new(SqliteDatamart::from_optional_config(None)),
            None,
            None,
            None,
            None,
            None,
            CacheBackend::Disabled,
            1,
            vec![],
            None,
            None,
            Some(Arc::new(Mutex::new(NodePolicy::default()))),
        );
        let descriptor = MeshJobDescriptor {
            job_id: "job-export".into(),
            job_type: MeshJobType::ExportJob,
            parameters: serde_json::Value::default(),
            governance_context: None,
        };
        let result = plane.run_mesh_job(&descriptor).await;
        assert_eq!(result.status, MeshJobStatus::Denied);
        let code = result.error.as_ref().unwrap().code.as_str();
        assert_eq!(code, "policy_denied:export_blocked_for_tier_licensed");
    }

    #[tokio::test]
    async fn dp_budget_persists_to_sqlite_ledger() {
        let temp_root = std::env::temp_dir().join("mesh-node-ledger-test");
        let _ = fs::create_dir_all(&temp_root);
        let db_path = temp_root.join("dp-ledger.db");
        let _ = fs::File::create(&db_path);
        let url = format!(
            "sqlite:///{path}",
            path = db_path.to_string_lossy().replace('\\', "/")
        );
        let warehouse = refractive_swan_datamart::WarehouseConfig {
            url: url.clone(),
            schema: None,
            max_connections: 5,
        };
        let ledger = DpBudgetLedger::new(warehouse.clone()).unwrap();
        let plane = NodeDataPlane::new(
            MeshNodeId::from_string("node-ledger".into()),
            Policy::default_for_mode(ComplianceMode::Internal),
            Arc::new(refractive_swan_eval::FileDatasetStore::new(&temp_root)),
            Arc::new(DefaultPipeline),
            Arc::new(SqliteDatamart::from_optional_config(Some(
                warehouse.clone(),
            ))),
            None,
            None,
            Some(warehouse),
            Some(ledger),
            None,
            CacheBackend::Disabled,
            1,
            vec![],
            None,
            None,
            Some(Arc::new(Mutex::new(NodePolicy::default()))),
        );
        plane.consume_budget(0.25).await;
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&url)
            .await
            .unwrap();
        let row: (f64,) = sqlx::query_as(
            "SELECT consumed FROM mesh_dp_budget WHERE node_id = ?1 AND date = date('now')",
        )
        .bind("node-ledger")
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(row.0 >= 0.25);
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

fn policy_error_code(default: &str, reason: &str) -> MeshErrorCode {
    if reason.contains("dp_budget_exceeded") {
        MeshErrorCode::new("policy_denied:dp_budget_exceeded")
    } else if reason.contains("export_not_allowed") || reason.contains("export_blocked") {
        MeshErrorCode::new("policy_denied:export_blocked_for_tier_licensed")
    } else {
        MeshErrorCode::new(default)
    }
}

fn export_blocked_code(policy: &Policy) -> MeshErrorCode {
    if !policy.is_allowed(ComplianceAction::Export, LicenseTier::Licensed) {
        MeshErrorCode::new("policy_denied:export_blocked_for_tier_licensed")
    } else {
        MeshErrorCode::new("policy_denied:export_not_allowed")
    }
}
