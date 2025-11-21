use refractive_swan_contracts::{
    PipelineMetrics,
    eval::{DatasetManifest, EvalSummary},
    pipeline::MappingState,
};

use crate::client::CohortFilters;

/// Default the API cannot list available eval datasets.
/// Keep this aligned with docs/runbook/mapping-eval-quickstart.md examples.
pub const DEFAULT_EVAL_DATASET: &str = "gold_pet_ct_small";

#[derive(Debug, Clone)]
pub struct PageContext {
    pub health: Option<HealthOverview>,
    pub health_error: Option<String>,
    pub metrics: Option<PipelineMetrics>,
    pub alert: Option<AlertMessage>,
    pub results: Option<MappingResultsView>,
    pub eval: Option<EvalContext>,
    pub datasets: Vec<DatasetManifest>,
    pub selected_eval_dataset: String,
    pub eval_report_html: Option<String>,
    pub eval_panel_error: Option<String>,
    pub analytics_summary: Option<AnalyticsSummaryView>,
    pub analytics_error: Option<String>,
    pub cohort: Option<CohortView>,
    pub cohort_filters: CohortFilters,
    pub cohort_error: Option<String>,
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
