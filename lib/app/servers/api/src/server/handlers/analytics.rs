use axum::extract::{Query, State};

use crate::{
    server::controllers,
    types::CohortQuery,
    utils::{ApiError, ApiState},
};

pub async fn analytics_ncit_summary(
    State(state): State<ApiState>,
) -> Result<axum::response::Response, ApiError> {
    controllers::analytics::ncit_summary(State(state)).await
}

pub async fn analytics_cohort(
    State(state): State<ApiState>,
    Query(query): Query<CohortQuery>,
) -> Result<axum::response::Response, ApiError> {
    controllers::analytics::cohort(State(state), Query(query)).await
}
