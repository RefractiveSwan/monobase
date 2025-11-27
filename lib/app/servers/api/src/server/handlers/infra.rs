use crate::server::controllers;
use axum::response::IntoResponse;

/// Basic health endpoint.
pub async fn health() -> impl IntoResponse {
    controllers::infra::health().await
}

/// Pipeline metrics snapshot for dashboards.
pub async fn metrics_summary(
    state: axum::extract::State<crate::utils::ApiState>,
) -> impl IntoResponse {
    controllers::infra::metrics_summary(state).await
}
