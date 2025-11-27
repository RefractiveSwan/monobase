use axum::Json;
use log::info;
use serde_json::json;
use uuid::Uuid;

use crate::utils::ApiState;

/// Basic health endpoint.
pub async fn health() -> impl axum::response::IntoResponse {
    let request_id = Uuid::new_v4();
    info!(target: "refractive_swan_api", "request_id={request_id} health");
    Json(json!({ "status": "ok" }))
}

/// Pipeline metrics snapshot for dashboards.
pub async fn metrics_summary(
    state: axum::extract::State<ApiState>,
) -> impl axum::response::IntoResponse {
    let request_id = Uuid::new_v4();
    let metrics_handle = state.plane.metrics();
    let metrics = metrics_handle.lock().await.clone();
    info!(
        target: "refractive_swan_api",
        "request_id={request_id} metrics_summary bundles={} mappings={} compliance_mode={:?}",
        metrics.bundle_count,
        metrics.mapping_count,
        metrics.compliance_mode
    );
    Json(metrics)
}
