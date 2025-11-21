//! Datamart sink port used by API/CLI surfaces.

use async_trait::async_trait;
use dfps_compliance::Policy;
use dfps_contracts::{AnalyticsSummaryResponse, CohortResponse, LoadSummary, PipelineOutput};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatamartError {
    #[error("datamart not configured")]
    Disabled,
    #[error("datamart load error: {0}")]
    Load(String),
    #[error("datamart backend error: {0}")]
    Backend(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CohortFilters {
    pub ncit_id: Option<String>,
    pub status: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[async_trait]
pub trait DatamartSink: Send + Sync {
    async fn persist(
        &self,
        output: &PipelineOutput,
        policy: &Policy,
    ) -> Result<LoadSummary, DatamartError>;

    async fn ncit_summary(&self) -> Result<AnalyticsSummaryResponse, DatamartError>;

    async fn cohort(&self, filters: &CohortFilters) -> Result<CohortResponse, DatamartError>;
}
