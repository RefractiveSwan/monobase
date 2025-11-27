use axum::Router;

mod admin;
mod analytics;
mod datasets;
mod eval;
mod health;
mod mapping;

use crate::server::ApiState;

/// v1 ReST routes (current HTTP surface).
///
/// Layout:
/// - Health/metrics: `/health`, `/metrics/summary`
/// - Analytics: `/analytics/ncit-summary`, `/analytics/cohort`
/// - Mapping: `/map-bundles`
/// - Evaluation: `/eval/*`
/// - Dataset admin: `/datasets/*`
/// - Admin/insights: `/admin/*`
pub fn router() -> Router<ApiState> {
    Router::new()
        .merge(health::router())
        .merge(analytics::router())
        .merge(mapping::router())
        .merge(eval::router())
        .merge(datasets::router())
        .merge(admin::router())
}
