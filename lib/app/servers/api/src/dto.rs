use dfps_eval::{DatasetManifest, EvalSummary};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSummaryResponse {
    pub rows: Vec<AnalyticsNcitSummaryRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsNcitSummaryRow {
    pub ncit_id: String,
    pub preferred_name: Option<String>,
    pub mapping_state: Option<String>,
    pub time_bucket: Option<String>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohortResponse {
    pub total: usize,
    pub rows: Vec<CohortRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohortRow {
    pub sr_id: String,
    pub patient_id: Option<String>,
    pub encounter_id: Option<String>,
    pub ncit_id: Option<String>,
    pub status: String,
    pub intent: String,
    pub description: String,
    pub ordered_at: Option<String>,
    pub mapping_state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRunResponse {
    pub dataset: String,
    pub manifest: Option<DatasetManifest>,
    pub summary: EvalSummary,
}
