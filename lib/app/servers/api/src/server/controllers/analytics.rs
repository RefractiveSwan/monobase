use axum::{Json, extract::Query, response::IntoResponse};
use log::{info, warn};
use uuid::Uuid;
use refractive_swan_datamart::{CohortFilters, DatamartError};
use refractive_swan_web_dto::{AnalyticsSummaryResponse, CohortResponse};

use crate::{
    types::CohortQuery,
    utils::{ApiError, ApiState},
};

impl From<&CohortQuery> for CohortFilters {
    fn from(query: &CohortQuery) -> Self {
        Self {
            ncit_id: query.ncit_id.clone(),
            status: query.status.clone(),
            date_from: query.date_from.clone(),
            date_to: query.date_to.clone(),
        }
    }
}

/// Analytics NCIT summary (used by dashboards).
pub async fn ncit_summary(
    state: axum::extract::State<ApiState>,
) -> Result<axum::response::Response, ApiError> {
    let request_id = Uuid::new_v4();
    info!(target: "refractive_swan_api", "request_id={request_id} analytics_ncit_summary");
    {
        let metrics_handle = state.plane.metrics();
        let mut metrics = metrics_handle.lock().await;
        metrics.analytics_requests += 1;
    }
    let response = match state.plane.datamart().ncit_summary().await {
        Ok(response) => response,
        Err(DatamartError::Disabled) => {
            warn!(
                target: "refractive_swan_api",
                "request_id={request_id} analytics summary skipped (datamart disabled)"
            );
            AnalyticsSummaryResponse { rows: Vec::new() }
        }
        Err(err) => {
            warn!(
                target: "refractive_swan_api",
                "request_id={request_id} analytics summary query failed: {err}"
            );
            AnalyticsSummaryResponse { rows: Vec::new() }
        }
    };
    Ok(Json(response).into_response())
}

/// Cohort query handler with filters.
pub async fn cohort(
    state: axum::extract::State<ApiState>,
    Query(query): Query<CohortQuery>,
) -> Result<axum::response::Response, ApiError> {
    let request_id = Uuid::new_v4();
    info!(
        target: "refractive_swan_api",
        "request_id={request_id} analytics_cohort filters={:?}",
        query
    );
    {
        let metrics_handle = state.plane.metrics();
        let mut metrics = metrics_handle.lock().await;
        metrics.analytics_requests += 1;
        metrics.cohort_queries += 1;
    }
    let response = match state
        .plane
        .datamart()
        .cohort(&CohortFilters::from(&query))
        .await
    {
        Ok(response) => response,
        Err(DatamartError::Disabled) => {
            warn!(
                target: "refractive_swan_api",
                "request_id={request_id} analytics cohort skipped (datamart disabled)"
            );
            CohortResponse {
                rows: Vec::new(),
                total: 0,
            }
        }
        Err(err) => {
            warn!(
                target: "refractive_swan_api",
                "request_id={request_id} analytics cohort query failed: {err}"
            );
            CohortResponse {
                rows: Vec::new(),
                total: 0,
            }
        }
    };
    Ok(Json(response).into_response())
}
