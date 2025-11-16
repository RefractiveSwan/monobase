use std::{
    collections::{HashMap, HashSet},
    env,
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
use chrono::{DateTime, NaiveDate};
use dfps_core::{
    fhir::Bundle,
    mapping::{DimNCITConcept, MappingResult, MappingState},
    staging::{StgServiceRequestFlat, StgSrCodeExploded},
};
use dfps_datamart::{
    DimCode, DimCodeKey, DimEncounter, DimEncounterKey, DimNCIT, DimNCITKey, DimPatient,
    DimPatientKey, FactServiceRequest, WarehouseConfig, connect_sqlite, from_pipeline_output,
    load_from_pipeline_output, migrate,
};
use dfps_observability::{PipelineMetrics, log_no_match, log_pipeline_output};
use dfps_pipeline::{PipelineError, PipelineOutput, bundle_to_mapped_sr};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{Pool, Sqlite};
use thiserror::Error;
use tokio::{
    net::TcpListener,
    sync::{Mutex, OnceCell},
};
use uuid::Uuid;

use crate::dto::{
    AnalyticsNcitSummaryRow, AnalyticsSummaryResponse, CohortResponse, CohortRow, EvalRunResponse,
};
/// Runtime configuration for the HTTP server.
#[derive(Debug, Clone)]
pub struct ApiServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ApiServerConfig {
    fn default() -> Self {
        let host = env::var("DFPS_API_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("DFPS_API_PORT")
            .ok()
            .and_then(|raw| raw.parse::<u16>().ok())
            .unwrap_or(8080);
        Self { host, port }
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

#[derive(Clone)]
struct AnalyticsPersistence {
    config: Option<WarehouseConfig>,
    pool: Arc<OnceCell<Pool<Sqlite>>>,
}

impl AnalyticsPersistence {
    fn from_env() -> Self {
        Self {
            config: WarehouseConfig::from_env().ok(),
            pool: Arc::new(OnceCell::new()),
        }
    }

    async fn pool(&self) -> Option<Pool<Sqlite>> {
        let Some(cfg) = &self.config else {
            return None;
        };
        let pool = self
            .pool
            .get_or_try_init(|| async {
                let pool = connect_sqlite(cfg).await?;
                migrate(&pool).await?;
                Ok::<_, sqlx::Error>(pool)
            })
            .await;
        match pool {
            Ok(pool) => Some(pool.clone()),
            Err(err) => {
                warn!(target: "dfps_api", "analytics persistence disabled: {err}");
                None
            }
        }
    }

    async fn persist(&self, output: &PipelineOutput) {
        if let Some(pool) = self.pool().await {
            if let Err(err) = load_from_pipeline_output(&pool, output).await {
                warn!(target: "dfps_api", "analytics persistence failed: {err}");
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
struct AnalyticsState {
    patients: HashMap<DimPatientKey, DimPatient>,
    encounters: HashMap<DimEncounterKey, DimEncounter>,
    codes: HashMap<DimCodeKey, DimCode>,
    ncit: HashMap<DimNCITKey, DimNCIT>,
    facts: Vec<FactServiceRequest>,
    mapping_by_code: HashMap<DimCodeKey, MappingResult>,
}

impl AnalyticsState {
    fn record_output(&mut self, output: &PipelineOutput) {
        let (dims, mut facts) = from_pipeline_output(output);
        for patient in dims.patients {
            self.patients.entry(patient.key).or_insert(patient);
        }
        for encounter in dims.encounters {
            self.encounters.entry(encounter.key).or_insert(encounter);
        }
        for code in dims.codes {
            self.codes.entry(code.key).or_insert(code);
        }
        for concept in dims.ncit {
            self.ncit.entry(concept.key).or_insert(concept);
        }
        self.facts.append(&mut facts);

        for mapping in &output.mapping_results {
            let key = DimCodeKey::from_code_element_id(&mapping.code_element_id);
            self.mapping_by_code.insert(key, mapping.clone());
        }
    }

    fn ncit_summary(&self) -> AnalyticsSummaryResponse {
        let mut counts: HashMap<(String, Option<String>, Option<String>), usize> = HashMap::new();
        for fact in &self.facts {
            let ncit_id = self
                .lookup_ncit_id(fact.ncit_key)
                .unwrap_or_else(|| "UNKNOWN".into());
            let mapping_state = self
                .mapping_state_for(&fact.code_key)
                .map(mapping_state_label)
                .map(str::to_string);
            let time_bucket = normalize_date(&fact.ordered_at);
            let key = (ncit_id.clone(), mapping_state.clone(), time_bucket.clone());
            *counts.entry(key).or_default() += 1;
        }

        let mut rows: Vec<AnalyticsNcitSummaryRow> = counts
            .into_iter()
            .map(
                |((ncit_id, mapping_state, time_bucket), count)| AnalyticsNcitSummaryRow {
                    preferred_name: self.lookup_ncit_name(&ncit_id),
                    ncit_id,
                    mapping_state,
                    time_bucket,
                    count,
                },
            )
            .collect();
        rows.sort_by(|a, b| {
            b.count
                .cmp(&a.count)
                .then_with(|| a.ncit_id.cmp(&b.ncit_id))
        });
        AnalyticsSummaryResponse { rows }
    }

    fn cohort(&self, query: &CohortQuery) -> CohortResponse {
        let mut rows = Vec::new();
        for fact in &self.facts {
            let ncit_id = self.lookup_ncit_id(fact.ncit_key);
            if let Some(expected) = &query.ncit_id {
                if ncit_id.as_deref() != Some(expected.as_str()) {
                    continue;
                }
            }
            if let Some(status) = &query.status {
                if &fact.status != status {
                    continue;
                }
            }
            if !self.matches_date_filters(&fact.ordered_at, query) {
                continue;
            }

            let patient_id = self
                .patients
                .get(&fact.patient_key)
                .map(|patient| patient.patient_id.clone());
            let encounter_id = fact
                .encounter_key
                .and_then(|key| self.encounters.get(&key))
                .map(|encounter| encounter.encounter_id.clone());
            let mapping_state = self
                .mapping_state_for(&fact.code_key)
                .map(mapping_state_label)
                .map(str::to_string);
            rows.push(CohortRow {
                sr_id: fact.sr_id.clone(),
                patient_id,
                encounter_id,
                ncit_id,
                status: fact.status.clone(),
                intent: fact.intent.clone(),
                description: fact.description.clone(),
                ordered_at: fact.ordered_at.clone(),
                mapping_state,
            });
        }
        CohortResponse {
            total: rows.len(),
            rows,
        }
    }

    fn lookup_ncit_id(&self, key: Option<DimNCITKey>) -> Option<String> {
        key.and_then(|ncit_key| self.ncit.get(&ncit_key))
            .map(|dim| dim.ncit_id.clone())
    }

    fn lookup_ncit_name(&self, ncit_id: &str) -> Option<String> {
        self.ncit
            .values()
            .find(|dim| dim.ncit_id == ncit_id)
            .map(|dim| dim.preferred_name.clone())
    }

    fn mapping_state_for(&self, key: &DimCodeKey) -> Option<MappingState> {
        self.mapping_by_code.get(key).map(|mapping| mapping.state)
    }

    fn matches_date_filters(&self, ordered_at: &Option<String>, query: &CohortQuery) -> bool {
        if query.date_from.is_none() && query.date_to.is_none() {
            return true;
        }
        let Some(date) = normalize_date(ordered_at) else {
            return false;
        };
        if let Some(from) = &query.date_from {
            if date < *from {
                return false;
            }
        }
        if let Some(to) = &query.date_to {
            if date > *to {
                return false;
            }
        }
        true
    }
}

#[derive(Clone)]
pub struct ApiState {
    metrics: Arc<Mutex<PipelineMetrics>>,
    analytics: Arc<Mutex<AnalyticsState>>,
    analytics_persistence: AnalyticsPersistence,
    latest_eval: Arc<Mutex<Option<crate::dto::EvalRunResponse>>>,
}

impl ApiState {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(PipelineMetrics::default())),
            analytics: Arc::new(Mutex::new(AnalyticsState::default())),
            analytics_persistence: AnalyticsPersistence::from_env(),
            latest_eval: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for ApiState {
    fn default() -> Self {
        Self::new()
    }
}

/// Start the HTTP server using the provided configuration.
///
/// Builds the router, wires shared state, and blocks until Ctrl+C (or shutdown).
pub async fn run(config: ApiServerConfig) -> Result<(), ServerError> {
    let addr = config.socket_addr()?;
    info!(target: "dfps_api", "starting web backend on {addr}");
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|source| ServerError::Bind { addr, source })?;

    let router = router(ApiState::default());

    axum::serve(listener, router.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(ServerError::Serve)?;

    info!(target: "dfps_api", "server stopped");
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

async fn health() -> impl IntoResponse {
    let request_id = Uuid::new_v4();
    info!(target: "dfps_api", "request_id={request_id} health");
    Json(json!({ "status": "ok" }))
}

async fn analytics_ncit_summary(State(state): State<ApiState>) -> Result<Response, ApiError> {
    let request_id = Uuid::new_v4();
    info!(target: "dfps_api", "request_id={request_id} analytics_ncit_summary");
    {
        let mut metrics = state.metrics.lock().await;
        metrics.analytics_requests += 1;
    }
    let summary = {
        let analytics = state.analytics.lock().await;
        analytics.ncit_summary()
    };
    Ok(Json(summary).into_response())
}

async fn analytics_cohort(
    State(state): State<ApiState>,
    Query(query): Query<CohortQuery>,
) -> Result<Response, ApiError> {
    let request_id = Uuid::new_v4();
    info!(
        target: "dfps_api",
        "request_id={request_id} analytics_cohort ncit_id={:?} status={:?} date_from={:?} date_to={:?}",
        query.ncit_id,
        query.status,
        query.date_from,
        query.date_to
    );
    let response = {
        let analytics = state.analytics.lock().await;
        analytics.cohort(&query)
    };
    {
        let mut metrics = state.metrics.lock().await;
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

async fn eval_summary(Query(query): Query<EvalQuery>) -> Result<Response, ApiError> {
    let request_id = Uuid::new_v4();
    let dataset = query.dataset;
    info!(target: "dfps_api", "request_id={request_id} eval_summary dataset={dataset}");
    let cases = dfps_eval::load_dataset(&dataset)
        .map_err(|err| ApiError::invalid_dataset(err.to_string(), request_id))?;
    let summary = run_eval_internal(&cases, query.top_k);
    Ok(Json(summary).into_response())
}

async fn list_eval_datasets() -> Result<Response, ApiError> {
    let manifests = dfps_eval::list_manifests()
        .map_err(|err| ApiError::invalid_dataset(err.to_string(), Uuid::new_v4()))?;
    Ok(Json(manifests).into_response())
}

async fn run_eval(
    State(state): State<ApiState>,
    JsonPayload(body): JsonPayload<EvalRunRequest>,
) -> Result<Response, ApiError> {
    let request_id = Uuid::new_v4();
    info!(
        target: "dfps_api",
        "request_id={request_id} eval_run dataset={} top_k={}",
        body.dataset,
        body.top_k
    );
    let outcome = dfps_eval::load_dataset_with_manifest(&body.dataset)
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

fn run_eval_internal(cases: &[dfps_eval::EvalCase], top_k: usize) -> dfps_eval::EvalSummary {
    let summary =
        dfps_eval::run_eval_with_mapper(cases, |rows| dfps_mapping::map_staging_codes(rows).0);
    if top_k > 1 {
        // Placeholder until engine exposes true top-k.
        return summary;
    }
    summary
}

fn mapping_state_label(state: MappingState) -> &'static str {
    match state {
        MappingState::AutoMapped => "auto_mapped",
        MappingState::NeedsReview => "needs_review",
        MappingState::NoMatch => "no_match",
    }
}

fn normalize_date(value: &Option<String>) -> Option<String> {
    value.as_ref().and_then(|raw| {
        if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
            return Some(dt.date_naive().to_string());
        }
        NaiveDate::parse_from_str(raw, "%Y-%m-%d")
            .map(|date| date.to_string())
            .ok()
    })
}

async fn metrics_summary(State(state): State<ApiState>) -> impl IntoResponse {
    let request_id = Uuid::new_v4();
    let metrics = state.metrics.lock().await.clone();
    info!(
        target: "dfps_api",
        "request_id={request_id} metrics_summary bundles={} mappings={}",
        metrics.bundle_count,
        metrics.mapping_count
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
        target: "dfps_api",
        "request_id={request_id} map_bundles start bundles={}",
        bundles.len()
    );

    let mut response = MapBundlesResponse::default();
    let mut dims_seen: HashSet<String> = HashSet::new();
    let mut request_metrics = PipelineMetrics::default();

    for bundle in bundles {
        let output = bundle_to_mapped_sr(&bundle).map_err(|err| match err {
            PipelineError::Ingestion(source) => ApiError::ingestion(source.to_string(), request_id),
        })?;

        log_pipeline_output(
            &output.flats,
            &output.exploded_codes,
            &output.mapping_results,
            &mut request_metrics,
            output.vector_usage.clone(),
            None,
        );
        state.analytics_persistence.persist(&output).await;
        {
            let mut analytics = state.analytics.lock().await;
            analytics.record_output(&output);
        }

        response.flats.extend(output.flats);
        response.exploded_codes.extend(output.exploded_codes);
        for mapping in &output.mapping_results {
            if matches!(mapping.state, MappingState::NoMatch) {
                log_no_match(mapping);
            }
        }
        response.mapping_results.extend(output.mapping_results);

        for concept in output.dim_concepts {
            if dims_seen.insert(concept.ncit_id.clone()) {
                response.dim_concepts.push(concept);
            }
        }
    }

    {
        let mut global = state.metrics.lock().await;
        global.bundle_count += request_metrics.bundle_count;
        global.flats_count += request_metrics.flats_count;
        global.exploded_count += request_metrics.exploded_count;
        global.mapping_count += request_metrics.mapping_count;
        global.auto_mapped += request_metrics.auto_mapped;
        global.needs_review += request_metrics.needs_review;
        global.no_match += request_metrics.no_match;
        global.vector_queries += request_metrics.vector_queries;
        global.vector_hits += request_metrics.vector_hits;
        global.vector_fallbacks += request_metrics.vector_fallbacks;
        global.vector_latency_ms_p95 = global
            .vector_latency_ms_p95
            .or(request_metrics.vector_latency_ms_p95);
    }
    info!(
        target: "dfps_api",
        "request_id={request_id} map_bundles complete bundles={} flats={} mappings={} automap={} needs_review={} no_match={}",
        request_metrics.bundle_count,
        response.flats.len(),
        response.mapping_results.len(),
        request_metrics.auto_mapped,
        request_metrics.needs_review,
        request_metrics.no_match
    );

    Ok(Json(response).into_response())
}

fn shutdown_signal() -> impl std::future::Future<Output = ()> {
    async {
        match tokio::signal::ctrl_c().await {
            Ok(()) => info!(target: "dfps_api", "received shutdown signal"),
            Err(err) => warn!(target: "dfps_api", "failed waiting for ctrl_c: {err}"),
        }
    }
}

#[derive(Default, Serialize)]
struct MapBundlesResponse {
    flats: Vec<StgServiceRequestFlat>,
    exploded_codes: Vec<StgSrCodeExploded>,
    mapping_results: Vec<MappingResult>,
    dim_concepts: Vec<DimNCITConcept>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    code: &'static str,
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
            target: "dfps_api",
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
            target: "dfps_api",
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
            target: "dfps_api",
            "request_id={request_id} invalid dataset: {message}"
        );
        Self::InvalidDataset {
            message,
            request_id,
        }
    }

    #[allow(dead_code)]
    fn internal(message: impl Into<String>, request_id: Uuid) -> Self {
        let message = message.into();
        error!(
            target: "dfps_api",
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
        match self {
            ApiError::InvalidJson {
                message,
                request_id,
            } => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    code: "invalid_json",
                    message,
                    request_id,
                }),
            )
                .into_response(),
            ApiError::Ingestion {
                message,
                request_id,
            } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ErrorResponse {
                    code: "invalid_fhir",
                    message,
                    request_id,
                }),
            )
                .into_response(),
            ApiError::InvalidDataset {
                message,
                request_id,
            } => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    code: "invalid_dataset",
                    message,
                    request_id,
                }),
            )
                .into_response(),
            ApiError::Internal {
                message,
                request_id,
            } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "internal_error",
                    message,
                    request_id,
                }),
            )
                .into_response(),
        }
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

    fn sample_pipeline_output() -> PipelineOutput {
        PipelineOutput {
            flats: vec![StgServiceRequestFlat {
                sr_id: "SR-1".into(),
                patient_id: "PAT-1".into(),
                encounter_id: Some("ENC-1".into()),
                status: "active".into(),
                intent: "order".into(),
                description: "FDG uptake".into(),
                ordered_at: Some("2024-05-01T12:00:00Z".into()),
            }],
            exploded_codes: vec![StgSrCodeExploded {
                sr_id: "SR-1".into(),
                system: Some("http://loinc.org".into()),
                code: Some("24606-6".into()),
                display: Some("FDG uptake".into()),
            }],
            mapping_results: vec![MappingResult {
                code_element_id: "SR-1::http://loinc.org::24606-6".into(),
                cui: Some("C0001".into()),
                ncit_id: Some("C1234".into()),
                score: 0.98,
                strategy: dfps_core::mapping::MappingStrategy::Lexical,
                state: MappingState::AutoMapped,
                thresholds: dfps_core::mapping::MappingThresholds::default(),
                source_version: dfps_core::mapping::MappingSourceVersion::new("ncit", "umls"),
                reason: None,
                license_tier: None,
                source_kind: None,
            }],
            dim_concepts: vec![DimNCITConcept {
                ncit_id: "C1234".into(),
                preferred_name: "FDG Uptake".into(),
                semantic_group: "Test".into(),
            }],
            vector_usage: None,
        }
    }

    #[test]
    fn analytics_state_builds_summary_and_cohort() {
        let mut analytics = AnalyticsState::default();
        analytics.record_output(&sample_pipeline_output());
        let summary = analytics.ncit_summary();
        assert_eq!(summary.rows.len(), 1);
        let row = &summary.rows[0];
        assert_eq!(row.ncit_id, "C1234");
        assert_eq!(row.mapping_state.as_deref(), Some("auto_mapped"));
        assert_eq!(row.count, 1);

        let cohort = analytics.cohort(&CohortQuery {
            ncit_id: Some("C1234".into()),
            status: Some("active".into()),
            date_from: Some("2024-05-01".into()),
            date_to: Some("2024-06-01".into()),
        });
        assert_eq!(cohort.total, 1);
        let cohort_row = &cohort.rows[0];
        assert_eq!(cohort_row.patient_id.as_deref(), Some("PAT-1"));
        assert_eq!(cohort_row.mapping_state.as_deref(), Some("auto_mapped"));
    }
}
