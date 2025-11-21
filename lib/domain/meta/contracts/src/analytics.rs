//! Analytics contracts shared across API/CLI/datamart.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Row returned by analytics summary queries.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AnalyticsSummaryRow {
    pub ncit_id: String,
    pub preferred_name: Option<String>,
    pub mapping_state: Option<String>,
    pub time_bucket: Option<String>,
    pub count: usize,
}

/// Response containing aggregated NCIt summary rows.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AnalyticsSummaryResponse {
    pub rows: Vec<AnalyticsSummaryRow>,
}

/// Row returned by cohort queries.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

/// Response payload for cohort queries.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CohortResponse {
    pub total: usize,
    pub rows: Vec<CohortRow>,
}
