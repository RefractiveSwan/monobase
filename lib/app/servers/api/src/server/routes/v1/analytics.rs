use axum::{Router, routing::get};

use crate::server::ApiState;
use crate::server::handlers::analytics;

/// Analytics endpoints (NCIT summary + cohort queries).
pub fn router() -> Router<ApiState> {
    Router::new()
        .route(
            "/analytics/ncit-summary",
            get(analytics::analytics_ncit_summary),
        )
        .route("/analytics/cohort", get(analytics::analytics_cohort))
}
