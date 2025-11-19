use dfps_core::{
    mapping::{DimNCITConcept, MappingResult},
    staging::{StgServiceRequestFlat, StgSrCodeExploded},
};
use dfps_eval::{DatasetManifest, EvalSummary};
use dfps_observability::PipelineMetrics;
use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;

use crate::config::AppConfig;

#[derive(Debug, Clone)]
pub struct BackendClient {
    client: Client,
    base_url: String,
}

impl BackendClient {
    pub fn from_config(config: &AppConfig) -> Result<Self, ClientError> {
        let client = Client::builder()
            .timeout(config.client_timeout)
            .build()
            .map_err(ClientError::Http)?;
        Ok(Self {
            client,
            base_url: config.backend_base_url.clone(),
        })
    }

    fn endpoint(&self, path: &str) -> String {
        let mut base = self.base_url.trim_end_matches('/').to_string();
        base.push_str(path);
        base
    }

    pub async fn health(&self) -> Result<HealthResponse, ClientError> {
        let response = self.client.get(self.endpoint("/health")).send().await?;
        Self::handle_json(response).await
    }

    pub async fn metrics_summary(&self) -> Result<PipelineMetrics, ClientError> {
        let response = self
            .client
            .get(self.endpoint("/metrics/summary"))
            .send()
            .await?;
        Self::handle_json(response).await
    }

    pub async fn eval_summary(&self, dataset: &str) -> Result<EvalSummary, ClientError> {
        let response = self
            .client
            .get(self.endpoint("/api/eval/summary"))
            .query(&[("dataset", dataset)])
            .send()
            .await?;
        Self::handle_json(response).await
    }

    pub async fn map_bundles(
        &self,
        payload: serde_json::Value,
    ) -> Result<MapBundlesResponse, ClientError> {
        let response = self
            .client
            .post(self.endpoint("/api/map-bundles"))
            .json(&payload)
            .send()
            .await?;
        Self::handle_json(response).await
    }

    pub async fn eval_datasets(&self) -> Result<Vec<DatasetManifest>, ClientError> {
        let response = self
            .client
            .get(self.endpoint("/api/eval/datasets"))
            .send()
            .await?;
        Self::handle_json(response).await
    }

    pub async fn eval_run(
        &self,
        dataset: &str,
        top_k: usize,
    ) -> Result<EvalRunResponse, ClientError> {
        let response = self
            .client
            .post(self.endpoint("/api/eval/run"))
            .json(&serde_json::json!({ "dataset": dataset, "top_k": top_k }))
            .send()
            .await?;
        Self::handle_json(response).await
    }

    pub async fn analytics_summary(&self) -> Result<AnalyticsSummaryResponse, ClientError> {
        let response = self
            .client
            .get(self.endpoint("/analytics/ncit-summary"))
            .send()
            .await?;
        Self::handle_json(response).await
    }

    pub async fn analytics_cohort(
        &self,
        filters: &CohortFilters,
    ) -> Result<CohortResponse, ClientError> {
        let response = self
            .client
            .get(self.endpoint("/analytics/cohort"))
            .query(filters)
            .send()
            .await?;
        Self::handle_json(response).await
    }

    async fn handle_json<T>(response: Response) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
    {
        let status = response.status();
        if status.is_success() {
            response.json::<T>().await.map_err(ClientError::Http)
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(ClientError::Backend { status, body })
        }
    }
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("backend returned {status}: {body}")]
    Backend { status: StatusCode, body: String },
    #[error("bundle payload missing")]
    EmptyBundle,
    #[error("invalid bundle JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("unable to read upload: {0}")]
    Upload(String),
    #[error("utf-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MapBundlesResponse {
    pub flats: Vec<StgServiceRequestFlat>,
    pub exploded_codes: Vec<StgSrCodeExploded>,
    pub mapping_results: Vec<MappingResult>,
    pub dim_concepts: Vec<DimNCITConcept>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EvalRunResponse {
    pub dataset: String,
    pub manifest: Option<DatasetManifest>,
    pub summary: EvalSummary,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnalyticsSummaryResponse {
    pub rows: Vec<AnalyticsSummaryRow>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnalyticsSummaryRow {
    pub ncit_id: String,
    pub preferred_name: Option<String>,
    pub mapping_state: Option<String>,
    pub time_bucket: Option<String>,
    pub count: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CohortResponse {
    pub total: usize,
    pub rows: Vec<CohortRow>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CohortFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ncit_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_to: Option<String>,
}
