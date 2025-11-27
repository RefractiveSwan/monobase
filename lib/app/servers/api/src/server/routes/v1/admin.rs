use axum::{Router, routing::get};

use crate::server::ApiState;
use crate::server::handlers::admin;

/// Admin/insights endpoints (terminology/compliance/ingestion/vector/datamart/toggles).
pub fn router() -> Router<ApiState> {
    Router::new()
        .route(
            "/admin/terminology/insights",
            get(admin::terminology_insights),
        )
        .route("/admin/compliance/policy", get(admin::compliance_policy))
        .route("/admin/ingestion/summary", get(admin::ingestion_summary))
        .route("/admin/vector/status", get(admin::vector_status))
        .route("/admin/datamart/health", get(admin::datamart_health))
        .route("/admin/events", get(admin::admin_events))
        .route("/admin/toggles", get(admin::feature_toggles))
}
