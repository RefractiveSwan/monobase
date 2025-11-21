//! Outbound adapter for refractive_swan_web_frontend. Wraps reqwest so routes/views only
//! talk to contracts/DTOs instead of domain crates.

use log::error;
use reqwest::{Client, RequestBuilder, Response};
use serde::de::DeserializeOwned;

use super::error::{BackendError, ClientError};
use super::types::HealthResponse;
use crate::config::AppConfig;

pub use refractive_swan_web_dto::{
    AnalyticsSummaryResponse, AnalyticsSummaryRow, CohortResponse, CohortRow, DatasetManifest,
    EvalRunResponse, EvalSummary, PipelineMetrics,
};
pub type MapBundlesResponse = refractive_swan_web_dto::PipelineOutput;

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
        let response = self.send(self.client.get(self.endpoint("/health"))).await?;
        Self::handle_json(response).await
    }

    pub async fn metrics_summary(&self) -> Result<PipelineMetrics, ClientError> {
        let response = self
            .send(self.client.get(self.endpoint("/metrics/summary")))
            .await?;
        Self::handle_json(response).await
    }

    pub async fn eval_summary(&self, dataset: &str) -> Result<EvalSummary, ClientError> {
        let response = self
            .send(
                self.client
                    .get(self.endpoint("/api/eval/summary"))
                    .query(&[("dataset", dataset)]),
            )
            .await?;
        Self::handle_json(response).await
    }

    pub async fn map_bundles(
        &self,
        payload: serde_json::Value,
    ) -> Result<MapBundlesResponse, ClientError> {
        let response = self
            .send(
                self.client
                    .post(self.endpoint("/api/map-bundles"))
                    .json(&payload),
            )
            .await?;
        Self::handle_json(response).await
    }

    pub async fn eval_datasets(&self) -> Result<Vec<DatasetManifest>, ClientError> {
        let response = self
            .send(self.client.get(self.endpoint("/api/eval/datasets")))
            .await?;
        Self::handle_json(response).await
    }

    pub async fn eval_run(
        &self,
        dataset: &str,
        top_k: usize,
    ) -> Result<EvalRunResponse, ClientError> {
        let response = self
            .send(
                self.client
                    .post(self.endpoint("/api/eval/run"))
                    .json(&serde_json::json!({ "dataset": dataset, "top_k": top_k })),
            )
            .await?;
        Self::handle_json(response).await
    }

    pub async fn analytics_summary(&self) -> Result<AnalyticsSummaryResponse, ClientError> {
        let response = self
            .send(self.client.get(self.endpoint("/analytics/ncit-summary")))
            .await?;
        Self::handle_json(response).await
    }

    pub async fn analytics_cohort(
        &self,
        filters: &super::types::CohortFilters,
    ) -> Result<CohortResponse, ClientError> {
        let response = self
            .send(
                self.client
                    .get(self.endpoint("/analytics/cohort"))
                    .query(filters),
            )
            .await?;
        Self::handle_json(response).await
    }

    async fn send(&self, builder: RequestBuilder) -> Result<Response, ClientError> {
        match builder.send().await {
            Ok(resp) => Ok(resp),
            Err(err) => {
                error!(
                    target: "refractive_swan_web_frontend.client",
                    "backend request failed: {err}"
                );
                Err(ClientError::Http(err))
            }
        }
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
            let err = ClientError::Backend(BackendError::from_http(status, body));
            err.log();
            Err(err)
        }
    }
}
