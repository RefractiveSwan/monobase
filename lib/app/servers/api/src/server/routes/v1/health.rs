use axum::{Router, routing::get};

use crate::server::ApiState;
use crate::server::handlers::infra;

/// Health + metrics probes.
pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/health", get(infra::health))
        .route("/metrics/summary", get(infra::metrics_summary))
}
