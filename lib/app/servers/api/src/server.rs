use std::{
    collections::HashSet,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use axum::{
    Json, Router,
    body::Bytes,
    extract::{Json as JsonPayload, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use refractive_swan_compliance::assert_export_allowed;
use refractive_swan_contracts::{
    DimNCITConcept, ErrorCode, ErrorKind, MappingResult, MappingState, PipelineMetrics,
    PipelineOutput, StgServiceRequestFlat, StgSrCodeExploded, VectorUsageSnapshot,
};
use refractive_swan_core::fhir::Bundle;
use refractive_swan_datamart::{CohortFilters, DatamartError};
use refractive_swan_mesh_dto::MeshNodeId;
use refractive_swan_mesh_node::{NodeDataPlane, NodePlaneConfig};
use refractive_swan_observability::{log_no_match, log_pipeline_output_with_summary};
use refractive_swan_pipeline::{PipelineError, PipelineRunConfig};
use refractive_swan_terminology::codesystem::LicenseTier;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;
use tokio::{net::TcpListener, sync::Mutex};
use uuid::Uuid;

use crate::{
    config::ApiConfig,
    dto::{AnalyticsSummaryResponse, CohortResponse, EvalRunResponse},
};
/// Runtime configuration for the HTTP server.
#[derive(Debug, Clone)]
pub struct ApiServerConfig {
    pub host: String,
    pub port: u16,
}

impl ApiServerConfig {
    pub fn from_env() -> Self {
        let host = match refractive_swan_configuration::string_var("refractive_swan_API_HOST") {
            Ok(Some(value)) if !value.trim().is_empty() => value.trim().to_string(),
            Ok(_) => "127.0.0.1".into(),
            Err(err) => {
                warn!(target: "refractive_swan_api", "invalid refractive_swan_API_HOST: {err}; using default 127.0.0.1");
                "127.0.0.1".into()
            }
        };
        let port = match refractive_swan_configuration::port_var("refractive_swan_API_PORT") {
            Ok(Some(value)) => value,
            Ok(None) => 8080,
            Err(err) => {
                warn!(target: "refractive_swan_api", "invalid refractive_swan_API_PORT: {err}; using default 8080");
                8080
            }
        };
        Self { host, port }
    }
}

impl Default for ApiServerConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl ApiServerConfig {
    fn socket_addr(&self) -> Result<SocketAddr, ServerError> {
        let ip: IpAddr = self
            .host
            .parse()
            .map_err(|source| ServerError::InvalidHost {
                host: self.host.clone(),
                source,
            })?;
        Ok(SocketAddr::new(ip, self.port))
    }
}

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("invalid bind host '{host}': {source}")]
    InvalidHost {
        host: String,
        #[source]
        source: std::net::AddrParseError,
    },
    #[error("failed to bind server at {addr}: {source}")]
    Bind {
        addr: SocketAddr,
        #[source]
        source: std::io::Error,
    },
    #[error("server error: {0}")]
    Serve(#[source] std::io::Error),
}

fn license_tiers_from_output(output: &PipelineOutput) -> Vec<LicenseTier> {
    let mut tiers = HashSet::new();
    for mapping in &output.mapping_results {
        if let Some(label) = mapping.license_tier.as_deref()
            && let Some(tier) = parse_license_tier(label)
        {
            tiers.insert(tier);
        }
    }
    tiers.into_iter().collect()
}

fn parse_license_tier(value: &str) -> Option<LicenseTier> {
    match value.trim() {
        "licensed" => Some(LicenseTier::Licensed),
        "open" => Some(LicenseTier::Open),
        "internal_only" => Some(LicenseTier::InternalOnly),
        _ => None,
    }
}

fn enforce_export_policy(
    output: &PipelineOutput,
    policy: &refractive_swan_compliance::Policy,
    request_id: Uuid,
) -> Result<(), ApiError> {
    let tiers = license_tiers_from_output(output);
    assert_export_allowed(&tiers, policy).map_err(|err| {
        ApiError::compliance(
            format!(
                "export blocked by compliance mode {}: {}",
                policy.mode.as_str(),
                err
            ),
            request_id,
        )
    })
}

/// Application node state that wires domain ports + adapters (pipeline, datamart,
/// datasets, metrics, compliance policy) for handlers.
#[derive(Clone)]
pub struct ApiState {
    plane: Arc<NodeDataPlane>,
    latest_eval: Arc<Mutex<Option<crate::dto::EvalRunResponse>>>,
}

impl ApiState {
    pub fn from_plane_config(config: NodePlaneConfig) -> Self {
        let plane = NodeDataPlane::from_config(MeshNodeId::new_random(), config);
        Self {
            plane: Arc::new(plane),
            latest_eval: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for ApiState {
    fn default() -> Self {
        let config = NodePlaneConfig::from_env("app.web.api")
            .expect("refractive_swan_api: failed to load plane configuration");
        Self::from_plane_config(config)
    }
}

/// Start the HTTP server using the provided configuration.
///
/// Builds the router, wires shared state, and blocks until Ctrl+C (or shutdown).
pub async fn run(config: ApiConfig) -> Result<(), ServerError> {
    let addr = config.server.socket_addr()?;
    info!(target: "refractive_swan_api", "starting web backend on {addr}");
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|source| ServerError::Bind { addr, source })?;

    let router = router(ApiState::from_plane_config(config.plane));

    axum::serve(listener, router.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(ServerError::Serve)?;

    info!(target: "refractive_swan_api", "server stopped");
    Ok(())
}

pub fn router(state: ApiState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/metrics/summary", get(metrics_summary))
        .route("/analytics/ncit-summary", get(analytics_ncit_summary))
        .route("/analytics/cohort", get(analytics_cohort))
        .route("/api/map-bundles", post(map_bundles))
        .route("/api/eval/summary", get(eval_summary))
        .route("/api/eval/datasets", get(list_eval_datasets))
        .route("/api/eval/run", post(run_eval))
        .route("/api/eval/latest", get(latest_eval))
        .with_state(state)
}

#[derive(Deserialize)]
struct EvalQuery {
    dataset: String,
    #[serde(default = "default_top_k")]
    top_k: usize,
}

fn default_top_k() -> usize {
    1
}

#[derive(Deserialize)]
struct EvalRunRequest {
    dataset: String,
    #[serde(default = "default_top_k")]
    top_k: usize,
}

#[derive(Debug, Default, Deserialize)]
struct CohortQuery {
    ncit_id: Option<String>,
    status: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
}

impl From<&CohortQuery> for CohortFilters {
    fn from(query: &CohortQuery) -> Self {
        Self {
            ncit_id: query.ncit_id.clone(),
            status: query.status.clone(),
            date_from: query.date_from.clone(),
            date_to: query.date_to.clone(),
        }
    }
}

async fn health() -> impl IntoResponse {
    let request_id = Uuid::new_v4();
    info!(target: "refractive_swan_api", "request_id={request_id} health");
    Json(json!({ "status": "ok" }))
}

async fn analytics_ncit_summary(State(state): State<ApiState>) -> Result<Response, ApiError> {
    let request_id = Uuid::new_v4();
    info!(target: "refractive_swan_api", "request_id={request_id} analytics_ncit_summary");
    {
        let metrics_handle = state.plane.metrics();
        let mut metrics = metrics_handle.lock().await;
        metrics.analytics_requests += 1;
    }
    let response = match state.plane.datamart().ncit_summary().await {
        Ok(response) => response,
        Err(DatamartError::Disabled) => {
            warn!(
                target: "refractive_swan_api",
                "request_id={request_id} analytics summary skipped (datamart disabled)"
            );
            AnalyticsSummaryResponse { rows: Vec::new() }
        }
        Err(err) => {
            warn!(
                target: "refractive_swan_api",
                "request_id={request_id} analytics summary query failed: {err}"
            );
            AnalyticsSummaryResponse { rows: Vec::new() }
        }
    };
    Ok(Json(response).into_response())
}

async fn analytics_cohort(
    State(state): State<ApiState>,
    Query(query): Query<CohortQuery>,
) -> Result<Response, ApiError> {
    let request_id = Uuid::new_v4();
    info!(
        target: "refractive_swan_api",
        "request_id={request_id} analytics_cohort ncit_id={:?} status={:?} date_from={:?} date_to={:?}",
        query.ncit_id,
        query.status,
        query.date_from,
        query.date_to
    );
    let filters = CohortFilters::from(&query);
    let response = match state.plane.datamart().cohort(&filters).await {
        Ok(response) => response,
        Err(DatamartError::Disabled) => {
            warn!(
                target: "refractive_swan_api",
                "request_id={request_id} cohort query skipped (datamart disabled)"
            );
            CohortResponse {
                total: 0,
                rows: Vec::new(),
            }
        }
        Err(err) => {
            warn!(
                target: "refractive_swan_api",
                "request_id={request_id} cohort query failed: {err}"
            );
            CohortResponse {
                total: 0,
                rows: Vec::new(),
            }
        }
    };
    {
        let metrics_handle = state.plane.metrics();
        let mut metrics = metrics_handle.lock().await;
        metrics.analytics_requests += 1;
        metrics.cohort_queries += 1;
        metrics.cohort_results_total += response.total;
        if metrics.cohort_queries > 0 {
            metrics.avg_cohort_size =
                Some(metrics.cohort_results_total as f32 / metrics.cohort_queries as f32);
        }
    }
    Ok(Json(response).into_response())
}

async fn eval_summary(
    State(state): State<ApiState>,
    Query(query): Query<EvalQuery>,
) -> Result<Response, ApiError> {
    let request_id = Uuid::new_v4();
    let dataset = query.dataset;
    info!(target: "refractive_swan_api", "request_id={request_id} eval_summary dataset={dataset}");
    let cases = state
        .plane
        .dataset_store()
        .load_dataset(&dataset)
        .map_err(|err| ApiError::invalid_dataset(err.to_string(), request_id))?;
    let summary = run_eval_internal(&cases, query.top_k);
    Ok(Json(summary).into_response())
}

async fn list_eval_datasets(State(state): State<ApiState>) -> Result<Response, ApiError> {
    let manifests = state
        .plane
        .dataset_store()
        .list_manifests()
        .map_err(|err| ApiError::invalid_dataset(err.to_string(), Uuid::new_v4()))?;
    Ok(Json(manifests).into_response())
}

async fn run_eval(
    State(state): State<ApiState>,
    JsonPayload(body): JsonPayload<EvalRunRequest>,
) -> Result<Response, ApiError> {
    let request_id = Uuid::new_v4();
    info!(
        target: "refractive_swan_api",
        "request_id={request_id} eval_run dataset={} top_k={}",
        body.dataset,
        body.top_k
    );
    let outcome = state
        .plane
        .dataset_store()
        .load_dataset_with_manifest(&body.dataset)
        .map_err(|err| ApiError::invalid_dataset(err.to_string(), request_id))?;
    let summary = run_eval_internal(&outcome.cases, body.top_k);
    let response = EvalRunResponse {
        dataset: body.dataset.clone(),
        manifest: Some(outcome.manifest),
        summary,
    };
    {
        let mut latest = state.latest_eval.lock().await;
        *latest = Some(response.clone());
    }
    Ok(Json(response).into_response())
}

async fn latest_eval(State(state): State<ApiState>) -> Result<Response, ApiError> {
    let latest = state.latest_eval.lock().await;
    if let Some(run) = &*latest {
        Ok(Json(run).into_response())
    } else {
        Err(ApiError::invalid_dataset(
            "no eval has run yet".to_string(),
            Uuid::new_v4(),
        ))
    }
}

fn run_eval_internal(cases: &[refractive_swan_eval::EvalCase], top_k: usize) -> refractive_swan_eval::EvalSummary {
    let summary =
        refractive_swan_eval::run_eval_with_mapper(cases, |rows| refractive_swan_mapping::map_staging_codes(rows).0);
    if top_k > 1 {
        // Placeholder until engine exposes true top-k.
        return summary;
    }
    summary
}

async fn metrics_summary(State(state): State<ApiState>) -> impl IntoResponse {
    let request_id = Uuid::new_v4();
    let metrics_handle = state.plane.metrics();
    let metrics = metrics_handle.lock().await.clone();
    info!(
        target: "refractive_swan_api",
        "request_id={request_id} metrics_summary bundles={} mappings={} compliance_mode={:?}",
        metrics.bundle_count,
        metrics.mapping_count,
        metrics.compliance_mode
    );
    Json(metrics)
}

async fn map_bundles(State(state): State<ApiState>, body: Bytes) -> Result<Response, ApiError> {
    let request_id = Uuid::new_v4();
    let bundles = parse_bundles(&body, request_id)?;
    if bundles.is_empty() {
        return Err(ApiError::invalid_json(
            "request body did not contain any Bundles",
            request_id,
        ));
    }
    info!(
        target: "refractive_swan_api",
        "request_id={request_id} map_bundles start bundles={}",
        bundles.len()
    );

    let mut response = MapBundlesResponse::default();
    let policy = state.plane.policy();
    let mut request_metrics = PipelineMetrics {
        compliance_mode: Some(policy.mode.as_str().to_string()),
        ..PipelineMetrics::default()
    };
    let pipeline = state.plane.pipeline();
    let datamart = state.plane.datamart();
    let vector_context = state.plane.vector_context();

    for bundle in bundles {
        let exec = {
            let config = PipelineRunConfig::default();
            pipeline
                .map_bundle_with_validation(&bundle, &config, vector_context.as_ref())
                .map_err(|err| match err {
                    PipelineError::Ingestion(source) => {
                        ApiError::ingestion(source.to_string(), request_id)
                    }
                })?
        };
        enforce_export_policy(&exec.output, policy, request_id)?;

        log_pipeline_output_with_summary(
            &exec.output.flats,
            &exec.output.exploded_codes,
            &exec.output.mapping_results,
            &exec.metrics,
            &mut request_metrics,
        );
        if let Err(err) = datamart.persist(&exec.output, policy).await {
            match err {
                DatamartError::Disabled => {
                    warn!(
                        target: "refractive_swan_api",
                        "request_id={request_id} datamart persist skipped (disabled)"
                    );
                }
                other => {
                    warn!(
                        target: "refractive_swan_api",
                        "request_id={request_id} datamart persist failed: {other}"
                    );
                }
            }
        }

        for mapping in &exec.output.mapping_results {
            if matches!(mapping.state, MappingState::NoMatch) {
                log_no_match(mapping);
            }
        }
        response.record_output(exec.output);
    }

    {
        let metrics_handle = state.plane.metrics();
        let mut global = metrics_handle.lock().await;
        global.merge(&request_metrics);
    }
    let total_flats = response.flats.len();
    let total_mappings = response.mapping_results.len();
    let total_dim_concepts = response.dim_concepts.len();
    let payload = response.into_pipeline_output();

    info!(
        target: "refractive_swan_api",
        "request_id={request_id} map_bundles complete bundles={} flats={} mappings={} dim_concepts={} automap={} needs_review={} no_match={} license_blocked={} compliance_mode={}",
        request_metrics.bundle_count,
        total_flats,
        total_mappings,
        total_dim_concepts,
        request_metrics.auto_mapped,
        request_metrics.needs_review,
        request_metrics.no_match,
        request_metrics.license_blocked,
        state.plane.policy().mode.as_str()
    );
    if request_metrics.license_blocked > 0 {
        warn!(
            target: "refractive_swan_compliance",
            "request_id={request_id} compliance_blocked reason=license_blocked mode={} count={}",
            state.plane.policy().mode.as_str(),
            request_metrics.license_blocked
        );
    }

    Ok(Json(payload).into_response())
}

async fn shutdown_signal() {
    match tokio::signal::ctrl_c().await {
        Ok(()) => info!(target: "refractive_swan_api", "received shutdown signal"),
        Err(err) => warn!(target: "refractive_swan_api", "failed waiting for ctrl_c: {err}"),
    }
}

#[derive(Default)]
struct MapBundlesResponse {
    flats: Vec<StgServiceRequestFlat>,
    exploded_codes: Vec<StgSrCodeExploded>,
    mapping_results: Vec<MappingResult>,
    dim_concepts: Vec<DimNCITConcept>,
    seen_dim_ids: HashSet<String>,
    vector_usage: Option<VectorUsageSnapshot>,
}

impl MapBundlesResponse {
    fn record_output(&mut self, output: PipelineOutput) {
        self.flats.extend(output.flats);
        self.exploded_codes.extend(output.exploded_codes);
        self.mapping_results.extend(output.mapping_results);
        for concept in output.dim_concepts {
            if self.seen_dim_ids.insert(concept.ncit_id.clone()) {
                self.dim_concepts.push(concept);
            }
        }
        if self.vector_usage.is_none() {
            self.vector_usage = output.vector_usage;
        }
    }

    fn into_pipeline_output(self) -> PipelineOutput {
        PipelineOutput {
            flats: self.flats,
            exploded_codes: self.exploded_codes,
            mapping_results: self.mapping_results,
            dim_concepts: self.dim_concepts,
            vector_usage: self.vector_usage,
        }
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    code: ErrorCode,
    kind: ErrorKind,
    message: String,
    request_id: Uuid,
}

#[derive(Debug)]
enum ApiError {
    InvalidJson {
        message: String,
        request_id: Uuid,
    },
    Ingestion {
        message: String,
        request_id: Uuid,
    },
    InvalidDataset {
        message: String,
        request_id: Uuid,
    },
    Compliance {
        message: String,
        request_id: Uuid,
    },
    #[allow(dead_code)]
    Internal {
        message: String,
        request_id: Uuid,
    },
}

impl ApiError {
    fn invalid_json(message: impl Into<String>, request_id: Uuid) -> Self {
        let message = message.into();
        warn!(
            target: "refractive_swan_api",
            "request_id={request_id} invalid json: {message}"
        );
        Self::InvalidJson {
            message,
            request_id,
        }
    }

    fn ingestion(message: impl Into<String>, request_id: Uuid) -> Self {
        let message = message.into();
        warn!(
            target: "refractive_swan_api",
            "request_id={request_id} invalid fhir payload: {message}"
        );
        Self::Ingestion {
            message,
            request_id,
        }
    }

    fn invalid_dataset(message: impl Into<String>, request_id: Uuid) -> Self {
        let message = message.into();
        warn!(
            target: "refractive_swan_api",
            "request_id={request_id} invalid dataset: {message}"
        );
        Self::InvalidDataset {
            message,
            request_id,
        }
    }

    fn compliance(message: impl Into<String>, request_id: Uuid) -> Self {
        let message = message.into();
        warn!(
            target: "refractive_swan_api",
            "request_id={request_id} compliance blocked: {message}"
        );
        Self::Compliance {
            message,
            request_id,
        }
    }

    #[allow(dead_code)]
    fn internal(message: impl Into<String>, request_id: Uuid) -> Self {
        let message = message.into();
        error!(
            target: "refractive_swan_api",
            "request_id={request_id} internal error: {message}"
        );
        Self::Internal {
            message,
            request_id,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, kind, message, request_id) = match self {
            ApiError::InvalidJson {
                message,
                request_id,
            } => (
                StatusCode::BAD_REQUEST,
                ErrorCode::InvalidJson,
                ErrorKind::AppHttpServer,
                message,
                request_id,
            ),
            ApiError::Ingestion {
                message,
                request_id,
            } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorCode::InvalidFhir,
                ErrorKind::DomainIngestion,
                message,
                request_id,
            ),
            ApiError::InvalidDataset {
                message,
                request_id,
            } => (
                StatusCode::BAD_REQUEST,
                ErrorCode::InvalidDataset,
                ErrorKind::DomainMapping,
                message,
                request_id,
            ),
            ApiError::Compliance {
                message,
                request_id,
            } => (
                StatusCode::FORBIDDEN,
                ErrorCode::ComplianceBlocked,
                ErrorKind::DomainCompliance,
                message,
                request_id,
            ),
            ApiError::Internal {
                message,
                request_id,
            } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::InternalError,
                ErrorKind::AppHttpServer,
                message,
                request_id,
            ),
        };
        (
            status,
            Json(ErrorResponse {
                code,
                kind,
                message,
                request_id,
            }),
        )
            .into_response()
    }
}

fn parse_bundles(body: &[u8], request_id: Uuid) -> Result<Vec<Bundle>, ApiError> {
    if body.iter().all(|byte| byte.is_ascii_whitespace()) {
        return Err(ApiError::invalid_json("request body is empty", request_id));
    }

    match serde_json::from_slice::<Value>(body) {
        Ok(Value::Object(map)) => {
            let bundle: Bundle = serde_json::from_value(Value::Object(map))
                .map_err(|err| ApiError::invalid_json(err.to_string(), request_id))?;
            Ok(vec![bundle])
        }
        Ok(Value::Array(items)) => {
            let mut bundles = Vec::with_capacity(items.len());
            for item in items {
                let bundle: Bundle = serde_json::from_value(item)
                    .map_err(|err| ApiError::invalid_json(err.to_string(), request_id))?;
                bundles.push(bundle);
            }
            Ok(bundles)
        }
        Ok(_) => Err(ApiError::invalid_json(
            "expected a Bundle object or array of Bundles",
            request_id,
        )),
        Err(_) => parse_ndjson(body, request_id),
    }
}

fn parse_ndjson(body: &[u8], request_id: Uuid) -> Result<Vec<Bundle>, ApiError> {
    let text = std::str::from_utf8(body)
        .map_err(|err| ApiError::invalid_json(err.to_string(), request_id))?;
    let mut bundles = Vec::new();

    for (idx, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let bundle: Bundle = serde_json::from_str(trimmed).map_err(|err| {
            ApiError::invalid_json(format!("ndjson line {}: {}", idx + 1, err), request_id)
        })?;
        bundles.push(bundle);
    }

    if bundles.is_empty() {
        Err(ApiError::invalid_json(
            "request body did not contain Bundle entries",
            request_id,
        ))
    } else {
        Ok(bundles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_bundle_json(id: &str) -> String {
        serde_json::json!({
            "resourceType": "Bundle",
            "type": "collection",
            "entry": [{
                "resource": {
                    "resourceType": "ServiceRequest",
                    "id": id,
                    "status": "active",
                    "intent": "order",
                    "subject": { "reference": "Patient/p1" }
                }
            }]
        })
        .to_string()
    }

    #[test]
    fn parses_single_bundle_object() {
        let body = sample_bundle_json("sr-1");
        let bundles = parse_bundles(body.as_bytes(), Uuid::nil()).unwrap();
        assert_eq!(bundles.len(), 1);
        assert_eq!(bundles[0].resource_type, "Bundle");
    }

    #[test]
    fn parses_array_of_bundles() {
        let body = format!(
            "[{},{}]",
            sample_bundle_json("sr-1"),
            sample_bundle_json("sr-2")
        );
        let bundles = parse_bundles(body.as_bytes(), Uuid::nil()).unwrap();
        assert_eq!(bundles.len(), 2);
    }

    #[test]
    fn parses_ndjson_payload() {
        let body = format!(
            "{}\n{}\n",
            sample_bundle_json("sr-1"),
            sample_bundle_json("sr-2")
        );
        let bundles = parse_bundles(body.as_bytes(), Uuid::nil()).unwrap();
        assert_eq!(bundles.len(), 2);
    }

    #[test]
    fn rejects_empty_payload() {
        let err = parse_bundles(b"", Uuid::nil()).unwrap_err();
        assert!(matches!(err, ApiError::InvalidJson { .. }));
    }
}
