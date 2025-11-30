use refractive_swan_contracts::{
    eval::{DatasetListEntry, EvalSummary},
    pipeline::{MappingState, ValidationSeverity},
};
use refractive_swan_web_dto::{
    AdminEvent, AnalyticsSummaryResponse, EvalSummary as DtoEvalSummary,
};

use crate::client::MetricsSnapshot;
use crate::{client::CohortFilters, vector::VectorMode, views::layout::ViewChrome};

/// Default the API cannot list available eval datasets.
/// Keep this aligned with docs/runbook/mapping-eval-quickstart.md examples.
pub const DEFAULT_EVAL_DATASET: &str = "gold_pet_ct_small";

#[derive(Debug, Clone)]
pub struct PageContext {
    pub health: Option<HealthOverview>,
    pub health_error: Option<String>,
    pub metrics: Option<MetricsSnapshot>,
    pub alert: Option<AlertMessage>,
    pub results: Option<MappingResultsView>,
    pub eval: Option<EvalContext>,
    pub datasets: Vec<DatasetListEntry>,
    pub selected_eval_dataset: String,
    pub eval_report_html: Option<String>,
    pub eval_panel_error: Option<String>,
    pub analytics_summary: Option<AnalyticsSummaryView>,
    pub analytics_error: Option<String>,
    pub cohort: Option<CohortView>,
    pub cohort_filters: CohortFilters,
    pub cohort_error: Option<String>,
    pub chrome: ViewChrome,
    pub validation_summary: Option<ValidationSummaryView>,
    pub mapping_history: Vec<MappingHistoryView>,
    pub eval_jobs: Vec<EvalJobView>,
    pub terminology_insights: Option<TerminologyInsightsView>,
    pub compliance_policy: Option<CompliancePolicyView>,
    pub ingestion_stats: Option<IngestionStatsView>,
    pub vector_config: VectorConfigView,
    pub dataset_admin: Option<DatasetAdminView>,
    pub regression_jobs: Vec<RegressionJobView>,
    pub maintenance_message: Option<String>,
    pub mesh_nodes: Vec<MeshNodeView>,
    pub mesh_jobs: Vec<MeshJobView>,
    pub mesh_governance: Option<String>,
    pub environment: Option<EnvironmentView>,
    pub mesh_admin_events: Vec<AdminEvent>,
    pub hub_analytics: Option<AnalyticsSummaryResponse>,
    pub hub_eval: Option<DtoEvalSummary>,
    pub mesh_alerts: Vec<String>,
}

impl Default for PageContext {
    fn default() -> Self {
        Self {
            health: None,
            health_error: None,
            metrics: None,
            alert: None,
            results: None,
            eval: None,
            datasets: Vec::new(),
            selected_eval_dataset: DEFAULT_EVAL_DATASET.to_string(),
            eval_report_html: None,
            eval_panel_error: None,
            analytics_summary: None,
            analytics_error: None,
            cohort: None,
            cohort_filters: CohortFilters::default(),
            cohort_error: None,
            chrome: ViewChrome::default(),
            validation_summary: None,
            mapping_history: Vec::new(),
            eval_jobs: Vec::new(),
            terminology_insights: None,
            compliance_policy: None,
            ingestion_stats: None,
            vector_config: VectorConfigView::default(),
            dataset_admin: None,
            regression_jobs: Vec::new(),
            maintenance_message: None,
            mesh_nodes: Vec::new(),
            mesh_jobs: Vec::new(),
            mesh_governance: None,
            environment: None,
            mesh_admin_events: Vec::new(),
            hub_analytics: None,
            hub_eval: None,
            mesh_alerts: Vec::new(),
        }
    }
}

impl PageContext {
    pub fn with_chrome(chrome: ViewChrome) -> Self {
        Self {
            chrome,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct HealthOverview {
    pub status: String,
    pub ok: bool,
}

#[derive(Debug, Clone)]
pub struct AlertMessage {
    pub kind: AlertKind,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertKind {
    Info,
    Error,
}

#[derive(Debug, Clone)]
pub struct MappingResultsView {
    pub request_summary: ServiceRequestSummary,
    pub rows: Vec<MappingRowView>,
    pub no_matches: Vec<NoMatchRowView>,
}

#[derive(Debug, Clone)]
pub struct EvalContext {
    pub dataset: String,
    pub summary: EvalSummary,
}

#[derive(Debug, Clone, Default)]
pub struct AnalyticsSummaryView {
    pub top_concepts: Vec<AnalyticsConceptTile>,
    pub state_counts: Vec<CountStat>,
    pub time_buckets: Vec<AnalyticsTimeBucket>,
}

#[derive(Debug, Clone, Default)]
pub struct AnalyticsSummaryFilter {
    pub state: Option<String>,
    pub range_days: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct AnalyticsConceptTile {
    pub ncit_id: String,
    pub preferred_name: String,
    pub total: usize,
}

#[derive(Debug, Clone)]
pub struct AnalyticsTimeBucket {
    pub bucket: String,
    pub state_counts: Vec<CountStat>,
}

#[derive(Debug, Clone)]
pub struct CohortView {
    pub total: usize,
    pub rows: Vec<CohortRowView>,
}

#[derive(Debug, Clone)]
pub struct CohortRowView {
    pub sr_id: String,
    pub patient_id: String,
    pub encounter_id: String,
    pub ncit_id: String,
    pub description: String,
    pub status: String,
    pub intent: String,
    pub ordered_at: String,
    pub mapping_state: String,
}

#[derive(Debug, Clone, Default)]
pub struct ServiceRequestSummary {
    pub total: usize,
    pub statuses: Vec<CountStat>,
    pub intents: Vec<CountStat>,
}

#[derive(Debug, Clone)]
pub struct CountStat {
    pub label: String,
    pub count: usize,
}

#[derive(Debug, Clone)]
pub struct MappingRowView {
    pub sr_id: String,
    pub system: String,
    pub code: String,
    pub display: String,
    pub ncit_id: Option<String>,
    pub ncit_label: Option<String>,
    pub state: MappingState,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NoMatchRowView {
    pub sr_id: String,
    pub system: String,
    pub code: String,
    pub display: String,
    pub reason: Option<String>,
}

impl From<&MappingRowView> for NoMatchRowView {
    fn from(value: &MappingRowView) -> Self {
        NoMatchRowView {
            sr_id: value.sr_id.clone(),
            system: value.system.clone(),
            code: value.code.clone(),
            display: value.display.clone(),
            reason: value.reason.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationSummaryView {
    pub total: usize,
    pub errors: usize,
    pub warnings: usize,
    pub info: usize,
    pub has_errors: bool,
    pub issues: Vec<ValidationIssueView>,
}

#[derive(Debug, Clone)]
pub struct ValidationIssueView {
    pub id: String,
    pub bundle_label: String,
    pub severity: ValidationSeverity,
    pub message: String,
    pub requirement: String,
}

#[derive(Debug, Clone)]
pub struct MappingHistoryView {
    pub id: String,
    pub submitted_at: String,
    pub duration_ms: u64,
    pub status: MappingHistoryStatusView,
    pub mapped: Option<usize>,
    pub error: Option<String>,
    pub validation_errors: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum MappingHistoryStatusView {
    Success,
    Error,
}

#[derive(Debug, Clone)]
pub struct EvalJobView {
    pub id: String,
    pub dataset: String,
    pub submitted_at: String,
    pub status: EvalJobStatusView,
    pub summary: Option<EvalSummaryViewData>,
    pub error: Option<String>,
    pub top_k: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum EvalJobStatusView {
    Queued,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct EvalSummaryViewData {
    pub precision: f32,
    pub recall: f32,
    pub coverage: f32,
    pub f1: f32,
    pub top1_accuracy: f32,
    pub top3_accuracy: f32,
    pub score_buckets: Vec<ScoreBucketView>,
    pub per_system: Vec<SystemMetricView>,
}

#[derive(Debug, Clone)]
pub struct ScoreBucketView {
    pub label: String,
    pub accuracy: f32,
    pub total: usize,
}

#[derive(Debug, Clone)]
pub struct SystemMetricView {
    pub label: String,
    pub precision: f32,
    pub recall: f32,
    pub f1: f32,
    pub total_cases: usize,
}

#[derive(Debug, Clone)]
pub struct LogEntryView {
    pub id: String,
    pub timestamp: String,
    pub message: String,
    pub kind: LogEntryKindView,
}

#[derive(Debug, Clone, Copy)]
pub enum LogEntryKindView {
    Info,
    NoMatch,
    Error,
}

impl From<&EvalSummary> for EvalSummaryViewData {
    fn from(summary: &EvalSummary) -> Self {
        let score_buckets = summary
            .score_buckets
            .iter()
            .map(|bucket| ScoreBucketView {
                label: bucket.bucket.clone(),
                accuracy: bucket.accuracy,
                total: bucket.total,
            })
            .collect();
        let mut per_system: Vec<SystemMetricView> = summary
            .by_system
            .iter()
            .map(|(system, metrics)| SystemMetricView {
                label: system.clone(),
                precision: metrics.precision,
                recall: metrics.recall,
                f1: metrics.f1,
                total_cases: metrics.total_cases,
            })
            .collect();
        per_system.sort_by(|a, b| b.total_cases.cmp(&a.total_cases));
        Self {
            precision: summary.precision,
            recall: summary.recall,
            coverage: summary.coverage,
            f1: summary.f1,
            top1_accuracy: summary.top1_accuracy,
            top3_accuracy: summary.top3_accuracy,
            score_buckets,
            per_system,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TerminologyInsightsView {
    pub total_systems: usize,
    pub licensed: usize,
    pub open: usize,
    pub code_systems: Vec<TerminologyCodeSystemView>,
    pub ontologies: Vec<TerminologyOntologyView>,
}

#[derive(Debug, Clone)]
pub struct TerminologyCodeSystemView {
    pub name: String,
    pub url: String,
    pub license_tier: String,
    pub source_kind: String,
}

#[derive(Debug, Clone)]
pub struct TerminologyOntologyView {
    pub id: String,
    pub name: String,
    pub iri: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct CompliancePolicyView {
    pub mode: String,
    pub actions: Vec<ComplianceActionView>,
}

#[derive(Debug, Clone)]
pub struct ComplianceActionView {
    pub action: String,
    pub allowed: bool,
    pub tiers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct IngestionStatsView {
    pub total_runs: usize,
    pub errors: usize,
    pub warnings: usize,
    pub info: usize,
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VectorConfigView {
    pub mode: VectorMode,
    pub backend: Option<String>,
    pub namespace: Option<String>,
    pub env_enabled: bool,
}

impl Default for VectorConfigView {
    fn default() -> Self {
        Self {
            mode: VectorMode::Enabled,
            backend: None,
            namespace: None,
            env_enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DatasetAdminView {
    pub entries: Vec<DatasetAdminEntryView>,
    pub fixtures: Vec<DatasetFixtureView>,
    pub disabled_total: usize,
    pub root: String,
    pub datamart_url: Option<String>,
    pub datamart_reset_supported: bool,
}

#[derive(Debug, Clone)]
pub struct DatasetAdminEntryView {
    pub name: String,
    pub version: String,
    pub license: Option<String>,
    pub source: Option<String>,
    pub tier: String,
    pub notes: Option<String>,
    pub n_cases: usize,
    pub sha256: String,
    pub disabled: bool,
}

#[derive(Debug, Clone)]
pub struct DatasetFixtureView {
    pub name: String,
    pub manifest_href: String,
    pub data_href: String,
}

#[derive(Debug, Clone)]
pub struct RegressionJobView {
    pub id: String,
    pub label: String,
    pub status: RegressionJobStatusView,
    pub submitted_at: String,
    pub completed_at: Option<String>,
    pub log: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegressionJobStatusView {
    Pending,
    Running,
    Passed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct MeshNodeView {
    pub id: String,
    pub vector_backend: String,
    pub warehouse_backend: String,
    pub cache_backend: Option<String>,
    pub cache_status: Option<String>,
    pub last_seen_ms: Option<u128>,
    pub dp_budget_remaining: Option<f64>,
    pub dp_budget_status: Option<String>,
    pub compliance_mode: String,
    pub max_dataset_size: u64,
    pub tags: Vec<String>,
    pub status: String,
    pub metrics: Option<refractive_swan_web_dto::PipelineMetrics>,
}

#[derive(Debug, Clone)]
pub struct MeshJobView {
    pub job_id: String,
    pub job_type: String,
    pub status: String,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct EnvironmentView {
    pub frontend_listen_addr: String,
    pub backend_base_url: String,
    pub docs_url: Option<String>,
    pub github_url: Option<String>,
    pub feature_flags: Vec<EnvVarView>,
    pub diagnostics: DiagnosticsView,
}

#[derive(Debug, Clone)]
pub struct EnvVarView {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct DiagnosticsView {
    pub health: Option<crate::client::HealthResponse>,
    pub metrics: Option<crate::client::MetricsSnapshot>,
}
