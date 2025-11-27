use axum::extract::State;

use crate::{
    server::controllers,
    utils::{ApiError, ApiState},
};

pub async fn terminology_insights() -> Result<axum::response::Response, ApiError> {
    controllers::admin::terminology_insights().await
}

pub async fn compliance_policy(
    State(state): State<ApiState>,
) -> Result<axum::response::Response, ApiError> {
    controllers::admin::compliance_policy(State(state)).await
}

pub async fn ingestion_summary(
    State(state): State<ApiState>,
) -> Result<axum::response::Response, ApiError> {
    controllers::admin::ingestion_summary(State(state)).await
}

pub async fn vector_status(
    State(state): State<ApiState>,
) -> Result<axum::response::Response, ApiError> {
    controllers::admin::vector_status(State(state)).await
}

pub async fn datamart_health(
    State(state): State<ApiState>,
) -> Result<axum::response::Response, ApiError> {
    controllers::admin::datamart_health(State(state)).await
}

pub async fn admin_events(
    State(state): State<ApiState>,
) -> Result<axum::response::Response, ApiError> {
    controllers::admin::admin_events(State(state)).await
}

pub async fn feature_toggles(
    State(state): State<ApiState>,
) -> Result<axum::response::Response, ApiError> {
    controllers::admin::feature_toggles(State(state)).await
}
