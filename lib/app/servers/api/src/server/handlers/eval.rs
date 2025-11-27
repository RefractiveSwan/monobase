use axum::extract::{Json as JsonPayload, Query, State};

use crate::{
    server::controllers,
    types::{EvalQuery, EvalRunRequest},
    utils::{ApiError, ApiState},
};

pub async fn eval_summary(
    State(state): State<ApiState>,
    Query(query): Query<EvalQuery>,
) -> Result<axum::response::Response, ApiError> {
    controllers::eval::eval_summary(State(state), Query(query)).await
}

pub async fn list_eval_datasets(
    State(state): State<ApiState>,
) -> Result<axum::response::Response, ApiError> {
    controllers::eval::list_datasets(State(state)).await
}

pub async fn run_eval(
    State(state): State<ApiState>,
    JsonPayload(body): JsonPayload<EvalRunRequest>,
) -> Result<axum::response::Response, ApiError> {
    controllers::eval::run_eval(State(state), JsonPayload(body)).await
}

pub async fn latest_eval(
    State(state): State<ApiState>,
) -> Result<axum::response::Response, ApiError> {
    controllers::eval::latest(State(state)).await
}
