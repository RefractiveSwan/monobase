//! Outbound adapter for dfps_web_frontend. Wraps reqwest so routes/views only
//! talk to contracts/DTOs instead of domain crates. Ports documented in
//! docs/system-design/base/dependency-seams.md.

use dfps_contracts::{
    PipelineOutput,
    errors::{ErrorCode, ErrorKind},
};
use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;

use crate::config::AppConfig;

pub use dfps_contracts::{
    AnalyticsSummaryResponse, AnalyticsSummaryRow, CohortResponse, CohortRow, DatasetManifest,
    EvalRunResponse, EvalSummary, PipelineMetrics,
};
pub type MapBundlesResponse = PipelineOutput;

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
            Err(ClientError::Backend(BackendError::from_http(status, body)))
        }
    }
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("{0}")]
    Backend(BackendError),
    #[error("bundle payload missing")]
    EmptyBundle,
    #[error("invalid bundle JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("unable to read upload: {0}")]
    Upload(String),
    #[error("utf-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

impl ClientError {
    /// Human-friendly error fragment that can be shown in views.
    pub fn user_message(&self) -> String {
        match self {
            ClientError::Backend(err) => err.user_message(),
            ClientError::Http(inner) => {
                format!("Unable to reach backend: {}", inner)
            }
            ClientError::InvalidJson(inner) => format!("Invalid JSON: {inner}"),
            ClientError::Upload(msg) => msg.clone(),
            ClientError::Utf8(inner) => format!("UTF-8 error: {inner}"),
            ClientError::EmptyBundle => "No bundle payload supplied".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct BackendError {
    status: StatusCode,
    code: Option<ErrorCode>,
    kind: Option<ErrorKind>,
    message: String,
    request_id: Option<String>,
}

impl BackendError {
    fn from_http(status: StatusCode, body: String) -> Self {
        match serde_json::from_str::<BackendErrorBody>(&body) {
            Ok(payload) => Self {
                status,
                code: Some(payload.code),
                kind: Some(payload.kind),
                message: payload.message,
                request_id: Some(payload.request_id),
            },
            Err(_) => Self {
                status,
                code: None,
                kind: None,
                message: body.clone(),
                request_id: None,
            },
        }
    }

    fn user_message(&self) -> String {
        if let Some(code) = self.code {
            if let Some(request_id) = &self.request_id {
                return format!(
                    "{} ({}) [request_id={request_id}]",
                    self.message,
                    code.as_str()
                );
            }
            return format!("{} ({})", self.message, code.as_str());
        }
        if !self.message.trim().is_empty() {
            return format!(
                "Backend responded with status {}: {}",
                self.status.as_u16(),
                self.message
            );
        }
        format!("Backend responded with status {}", self.status.as_u16())
    }
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(code) = self.code {
            write!(f, "{} ({})", self.message, code.as_str())?;
        } else if !self.message.trim().is_empty() {
            write!(f, "{} (status {})", self.message, self.status)?;
        } else {
            write!(f, "status {}", self.status)?;
        }
        if let Some(request_id) = &self.request_id {
            write!(f, " [request_id={request_id}]")?;
        }
        if let Some(kind) = self.kind {
            write!(f, " [{kind:?}]")?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct BackendErrorBody {
    code: ErrorCode,
    kind: ErrorKind,
    message: String,
    request_id: String,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn pipeline_output_contract_round_trips() {
        let payload = json!({
            "flats": [{
                "sr_id": "SR-1",
                "patient_id": "P1",
                "encounter_id": null,
                "status": "active",
                "intent": "order",
                "description": "PET-CT",
                "ordered_at": "2024-05-01T12:00:00Z"
            }],
            "exploded_codes": [{
                "sr_id": "SR-1",
                "system": "http://loinc.org",
                "code": "24606-6",
                "display": "FDG uptake"
            }],
            "mapping_results": [{
                "code_element_id": "SR-1::http://loinc.org::24606-6",
                "cui": "C0001",
                "ncit_id": "C1234",
                "score": 0.99,
                "strategy": "lexical",
                "state": "auto_mapped",
                "thresholds": { "auto_map_min": 0.9, "needs_review_min": 0.7 },
                "source_version": { "ncit": "ncit-2024", "umls": "umls-2024" },
                "reason": null,
                "license_tier": null,
                "source_kind": null
            }],
            "dim_concepts": [{
                "ncit_id": "C1234",
                "preferred_name": "FDG Uptake",
                "semantic_group": "Test"
            }],
            "vector_usage": null
        });
        let contract: MapBundlesResponse = serde_json::from_value(payload.clone()).unwrap();
        assert_eq!(contract.flats.len(), 1);
        let round_trip = serde_json::to_value(&contract).unwrap();
        assert!(round_trip.get("mapping_results").is_some());
        assert!(round_trip.get("vector_usage").is_some());
    }

    #[test]
    fn analytics_contract_deserializes_backend_payloads() {
        let payload = json!({
            "rows": [{
                "ncit_id": "C1234",
                "preferred_name": "FDG Uptake",
                "mapping_state": "auto_mapped",
                "time_bucket": "2024-05-01",
                "count": 3
            }]
        });
        let summary: AnalyticsSummaryResponse = serde_json::from_value(payload).unwrap();
        assert_eq!(summary.rows[0].ncit_id, "C1234");

        let cohort_json = json!({
            "total": 1,
            "rows": [{
                "sr_id": "SR-1",
                "patient_id": "P1",
                "encounter_id": "E1",
                "ncit_id": "C1234",
                "status": "active",
                "intent": "order",
                "description": "PET",
                "ordered_at": "2024-05-01T12:00:00Z",
                "mapping_state": "auto_mapped"
            }]
        });
        let cohort: CohortResponse = serde_json::from_value(cohort_json).unwrap();
        assert_eq!(cohort.total, 1);
        assert_eq!(cohort.rows[0].ncit_id.as_deref(), Some("C1234"));
    }

    #[test]
    fn backend_error_user_messages_include_code() {
        let body = json!({
            "code": "invalid_fhir",
            "kind": "domain_ingestion",
            "message": "bundle missing entries",
            "request_id": "4ed499d6-bafd-45d5-b3c6-d2da7a5e0a10"
        })
        .to_string();
        let err = BackendError::from_http(StatusCode::UNPROCESSABLE_ENTITY, body);
        assert!(err.user_message().contains("invalid_fhir"));
        assert!(
            err.to_string()
                .contains("4ed499d6-bafd-45d5-b3c6-d2da7a5e0a10")
        );
    }
}
