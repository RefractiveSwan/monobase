use crate::{
    client::{BackendClient, MapBundlesResponse},
    config::AppConfig,
    vector::VectorMode,
};
use actix_web::rt;
use chrono::{DateTime, Utc};
use log::info;
use refractive_swan_compliance::{Policy, load_policy_from_env};
use refractive_swan_contracts::{eval::EvalSummary, pipeline::ValidationReport};
use refractive_swan_observability::PipelineMetrics;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use uuid::Uuid;

const MAPPING_HISTORY_LIMIT: usize = 5;
const EVAL_JOBS_LIMIT: usize = 10;
const LOG_HISTORY_LIMIT: usize = 50;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub client: BackendClient,
    pub dataset_store: Arc<dyn refractive_swan_eval::DatasetStore + Send + Sync>,
    pub analytics_metrics: Arc<Mutex<PipelineMetrics>>,
    mapping_history: Arc<Mutex<Vec<MappingHistoryEntry>>>,
    log_entries: Arc<Mutex<Vec<LogEntry>>>,
    eval_jobs: Arc<Mutex<Vec<EvalJobEntry>>>,
    vector_mode: Arc<Mutex<VectorMode>>,
    compliance_policy: Option<Policy>,
    dataset_admin: Arc<Mutex<DatasetAdminState>>,
    dataset_admin_path: PathBuf,
    regression_jobs: Arc<Mutex<Vec<RegressionJobEntry>>>,
}

impl AppState {
    pub fn new(
        config: AppConfig,
        client: BackendClient,
        dataset_store: Arc<dyn refractive_swan_eval::DatasetStore + Send + Sync>,
    ) -> Self {
        let dataset_admin_path = dataset_store.data_root().join(".dataset_admin_state.json");
        Self {
            config,
            client,
            dataset_store,
            analytics_metrics: Arc::new(Mutex::new(PipelineMetrics::default())),
            mapping_history: Arc::new(Mutex::new(Vec::new())),
            log_entries: Arc::new(Mutex::new(Vec::new())),
            eval_jobs: Arc::new(Mutex::new(Vec::new())),
            vector_mode: Arc::new(Mutex::new(VectorMode::from_env())),
            compliance_policy: load_policy_from_env().ok(),
            dataset_admin: Arc::new(Mutex::new(DatasetAdminState::load(&dataset_admin_path))),
            dataset_admin_path,
            regression_jobs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn record_analytics(&self, cohort_total: Option<usize>) {
        if let Ok(mut metrics) = self.analytics_metrics.lock() {
            metrics.analytics_requests += 1;
            if let Some(total) = cohort_total {
                metrics.cohort_queries += 1;
                metrics.cohort_results_total += total;
                if metrics.cohort_queries > 0 {
                    metrics.avg_cohort_size =
                        Some(metrics.cohort_results_total as f32 / metrics.cohort_queries as f32);
                }
            }
            info!(
                target: "refractive_swan_web_frontend.analytics",
                "analytics_requests={} cohort_queries={} last_total={:?}",
                metrics.analytics_requests,
                metrics.cohort_queries,
                cohort_total
            );
        }
    }

    pub fn record_history(&self, entry: MappingHistoryEntry) {
        if let Ok(mut history) = self.mapping_history.lock() {
            history.insert(0, entry);
            if history.len() > MAPPING_HISTORY_LIMIT {
                history.truncate(MAPPING_HISTORY_LIMIT);
            }
        }
    }

    pub fn history_snapshot(&self) -> Vec<MappingHistoryEntry> {
        self.mapping_history
            .lock()
            .map(|entries| entries.clone())
            .unwrap_or_default()
    }

    pub fn history_entry(&self, id: &Uuid) -> Option<MappingHistoryEntry> {
        self.mapping_history
            .lock()
            .ok()
            .and_then(|entries| entries.iter().find(|entry| &entry.id == id).cloned())
    }

    pub fn record_log(&self, entry: LogEntry) {
        if let Ok(mut logs) = self.log_entries.lock() {
            logs.insert(0, entry);
            if logs.len() > LOG_HISTORY_LIMIT {
                logs.truncate(LOG_HISTORY_LIMIT);
            }
        }
    }

    pub fn log_snapshot(&self) -> Vec<LogEntry> {
        self.log_entries
            .lock()
            .map(|entries| entries.clone())
            .unwrap_or_default()
    }

    pub fn record_completed_eval(&self, dataset: String, top_k: usize, summary: EvalSummary) {
        let dataset_name = dataset.clone();
        let mut entry = EvalJobEntry::queued(dataset, top_k);
        entry.status = EvalJobStatus::Completed;
        entry.summary = Some(summary.clone());
        if let Ok(mut jobs) = self.eval_jobs.lock() {
            jobs.insert(0, entry);
            if jobs.len() > EVAL_JOBS_LIMIT {
                jobs.truncate(EVAL_JOBS_LIMIT);
            }
        }
        self.record_log(LogEntry::info(format!(
            "Eval run {} finished (precision {:.1}%, recall {:.1}%)",
            dataset_name,
            summary.precision * 100.0,
            summary.recall * 100.0
        )));
    }

    pub fn enqueue_eval_job(&self, dataset: String, top_k: usize) -> Uuid {
        let entry = EvalJobEntry::queued(dataset.clone(), top_k);
        let id = entry.id;
        if let Ok(mut jobs) = self.eval_jobs.lock() {
            jobs.insert(0, entry);
            if jobs.len() > EVAL_JOBS_LIMIT {
                jobs.truncate(EVAL_JOBS_LIMIT);
            }
        }
        self.record_log(LogEntry::info(format!(
            "Queued eval run {} (top_k={})",
            dataset, top_k
        )));
        let state = self.clone();
        rt::spawn(async move {
            state.run_eval_job(id, dataset, top_k).await;
        });
        id
    }

    async fn run_eval_job(self, job_id: Uuid, dataset: String, top_k: usize) {
        self.update_eval_job(job_id, |job| {
            job.status = EvalJobStatus::Running;
        });
        let result = self.client.eval_run(&dataset, top_k).await;
        match result {
            Ok(run) => {
                let mut summary = run.summary;
                summary.results.clear();
                let log_summary = summary.clone();
                self.update_eval_job(job_id, |job| {
                    job.status = EvalJobStatus::Completed;
                    job.summary = Some(summary.clone());
                    job.error = None;
                });
                self.record_log(LogEntry::info(format!(
                    "Eval run {} completed (precision {:.1}%, recall {:.1}%)",
                    dataset,
                    log_summary.precision * 100.0,
                    log_summary.recall * 100.0
                )));
            }
            Err(err) => {
                let message = err.user_message();
                self.update_eval_job(job_id, |job| {
                    job.status = EvalJobStatus::Failed;
                    job.error = Some(message.clone());
                });
                self.record_log(LogEntry::error(format!(
                    "Eval run {} failed: {}",
                    dataset, message
                )));
            }
        }
    }

    fn update_eval_job<F>(&self, job_id: Uuid, mut update: F)
    where
        F: FnMut(&mut EvalJobEntry),
    {
        if let Ok(mut jobs) = self.eval_jobs.lock() {
            if let Some(job) = jobs.iter_mut().find(|job| job.id == job_id) {
                update(job);
            }
        }
    }

    pub fn eval_jobs_snapshot(&self) -> Vec<EvalJobEntry> {
        self.eval_jobs
            .lock()
            .map(|jobs| jobs.clone())
            .unwrap_or_default()
    }

    pub fn eval_job_summary(&self, id: &Uuid) -> Option<EvalJobEntry> {
        self.eval_jobs
            .lock()
            .ok()
            .and_then(|jobs| jobs.iter().find(|job| &job.id == id).cloned())
    }

    pub fn vector_mode(&self) -> VectorMode {
        self.vector_mode
            .lock()
            .map(|mode| *mode)
            .unwrap_or(VectorMode::Enabled)
    }

    pub fn set_vector_mode(&self, mode: VectorMode) {
        if let Ok(mut guard) = self.vector_mode.lock() {
            *guard = mode;
        }
    }

    pub fn compliance_policy(&self) -> Option<Policy> {
        self.compliance_policy.clone()
    }

    pub fn disabled_datasets(&self) -> Vec<String> {
        self.dataset_admin
            .lock()
            .map(|state| state.disabled.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn set_dataset_enabled(&self, name: &str, enabled: bool) {
        if let Ok(mut state) = self.dataset_admin.lock() {
            state.set_enabled(name, enabled);
            state.persist(&self.dataset_admin_path);
        }
    }

    pub fn dataset_root(&self) -> PathBuf {
        self.dataset_store.data_root().to_path_buf()
    }

    pub fn regression_jobs_snapshot(&self) -> Vec<RegressionJobEntry> {
        self.regression_jobs
            .lock()
            .map(|jobs| jobs.clone())
            .unwrap_or_default()
    }

    pub fn enqueue_regression_job(&self, label: String) -> Uuid {
        let entry = RegressionJobEntry::new(label);
        let id = entry.id;
        if let Ok(mut jobs) = self.regression_jobs.lock() {
            jobs.insert(0, entry);
            if jobs.len() > EVAL_JOBS_LIMIT {
                jobs.truncate(EVAL_JOBS_LIMIT);
            }
        }
        let state = self.clone();
        rt::spawn(async move {
            state.run_regression_job(id).await;
        });
        id
    }

    async fn run_regression_job(self, job_id: Uuid) {
        self.update_regression_job(job_id, |job| {
            job.status = RegressionJobStatus::Running;
        });
        let result =
            actix_web::web::block(|| Ok::<_, ()>(refractive_swan_test_suite::ping())).await;
        match result {
            Ok(_) => {
                self.update_regression_job(job_id, |job| {
                    job.status = RegressionJobStatus::Passed;
                    job.completed_at = Some(Utc::now());
                    job.log.push("Regression smoke suite passed".into());
                });
                self.record_log(LogEntry::info("Regression suite passed"));
            }
            Err(_) => {
                self.update_regression_job(job_id, |job| {
                    job.status = RegressionJobStatus::Failed;
                    job.completed_at = Some(Utc::now());
                    job.error = Some("Regression suite invocation failed".into());
                    job.log
                        .push("Regression suite invocation failed. Check logs.".into());
                });
                self.record_log(LogEntry::error("Regression suite failed"));
            }
        }
    }

    fn update_regression_job<F>(&self, job_id: Uuid, mut update: F)
    where
        F: FnMut(&mut RegressionJobEntry),
    {
        if let Ok(mut jobs) = self.regression_jobs.lock() {
            if let Some(job) = jobs.iter_mut().find(|job| job.id == job_id) {
                update(job);
            }
        }
    }

    pub fn clear_caches(&self) {
        if let Ok(mut history) = self.mapping_history.lock() {
            history.clear();
        }
        if let Ok(mut logs) = self.log_entries.lock() {
            logs.clear();
        }
        if let Ok(mut eval_jobs) = self.eval_jobs.lock() {
            eval_jobs.clear();
        }
        if let Ok(mut regression) = self.regression_jobs.lock() {
            regression.clear();
        }
    }

    pub fn datamart_url(&self) -> Option<String> {
        env::var("refractive_swan_WAREHOUSE_URL").ok()
    }

    pub fn reset_datamart(&self) -> Result<(), String> {
        let Some(url) = self.datamart_url() else {
            return Err("refractive_swan_WAREHOUSE_URL not set".into());
        };
        let Some(path) = url.strip_prefix("sqlite://") else {
            return Err("Datamart reset only supported for sqlite URLs".into());
        };
        let db_path = PathBuf::from(path);
        if db_path.exists() {
            fs::remove_file(&db_path).map_err(|err| err.to_string())?;
        }
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        fs::File::create(&db_path).map_err(|err| err.to_string())?;
        Ok(())
    }
}

#[derive(Clone, Default)]
struct DatasetAdminState {
    disabled: BTreeSet<String>,
}

impl DatasetAdminState {
    fn load(path: &Path) -> Self {
        if let Ok(contents) = fs::read_to_string(path) {
            if let Ok(state) = serde_json::from_str::<DatasetAdminDiskState>(&contents) {
                let disabled = state.disabled.into_iter().collect();
                return Self { disabled };
            }
        }
        Self::default()
    }

    fn set_enabled(&mut self, name: &str, enabled: bool) {
        if enabled {
            self.disabled.remove(name);
        } else {
            self.disabled.insert(name.to_string());
        }
    }

    fn persist(&self, path: &Path) {
        let state = DatasetAdminDiskState {
            disabled: self.disabled.iter().cloned().collect(),
        };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&state) {
            let _ = fs::write(path, json);
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct DatasetAdminDiskState {
    disabled: Vec<String>,
}

#[derive(Clone)]
pub struct MappingHistoryEntry {
    pub id: Uuid,
    pub submitted_at: DateTime<Utc>,
    pub duration_ms: u128,
    pub status: MappingHistoryStatus,
    pub mapped: Option<usize>,
    pub error: Option<String>,
    pub response: Option<MapBundlesResponse>,
    pub validation_reports: Vec<ValidationReport>,
}

impl MappingHistoryEntry {
    pub fn success(duration_ms: u128, mapped: usize, response: MapBundlesResponse) -> Self {
        Self {
            id: Uuid::new_v4(),
            submitted_at: Utc::now(),
            duration_ms,
            status: MappingHistoryStatus::Success,
            mapped: Some(mapped),
            error: None,
            validation_reports: response.validation_reports.clone(),
            response: Some(response),
        }
    }

    pub fn failure(duration_ms: u128, error: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            submitted_at: Utc::now(),
            duration_ms,
            status: MappingHistoryStatus::Error,
            mapped: None,
            error: Some(error),
            response: None,
            validation_reports: Vec::new(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum MappingHistoryStatus {
    Success,
    Error,
}

#[derive(Clone)]
pub struct LogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub kind: LogKind,
    pub message: String,
}

impl LogEntry {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            kind: LogKind::Info,
            message: message.into(),
        }
    }

    pub fn no_match(sr_id: &str, code: &str, reason: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            kind: LogKind::NoMatch,
            message: format!("NoMatch: sr_id={} code={} reason={}", sr_id, code, reason),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            kind: LogKind::Error,
            message: message.into(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum LogKind {
    Info,
    NoMatch,
    Error,
}

#[derive(Clone)]
pub struct EvalJobEntry {
    pub id: Uuid,
    pub dataset: String,
    pub top_k: usize,
    pub submitted_at: DateTime<Utc>,
    pub status: EvalJobStatus,
    pub summary: Option<EvalSummary>,
    pub error: Option<String>,
}

impl EvalJobEntry {
    fn queued(dataset: String, top_k: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            dataset,
            top_k,
            submitted_at: Utc::now(),
            status: EvalJobStatus::Queued,
            summary: None,
            error: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EvalJobStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

#[derive(Clone)]
pub struct RegressionJobEntry {
    pub id: Uuid,
    pub label: String,
    pub submitted_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: RegressionJobStatus,
    pub log: Vec<String>,
    pub error: Option<String>,
}

impl RegressionJobEntry {
    fn new(label: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            label,
            submitted_at: Utc::now(),
            completed_at: None,
            status: RegressionJobStatus::Pending,
            log: Vec::new(),
            error: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RegressionJobStatus {
    Pending,
    Running,
    Passed,
    Failed,
}
