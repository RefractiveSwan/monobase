use axum::extract::{Json as JsonPayload, Path, State};

use crate::{
    server::controllers,
    types::UploadDatasetRequest,
    utils::{ApiError, ApiState},
};

pub async fn refresh_datasets(
    State(state): State<ApiState>,
) -> Result<axum::response::Response, ApiError> {
    controllers::datasets::refresh(State(state)).await
}

pub async fn upload_dataset(
    State(state): State<ApiState>,
    JsonPayload(body): JsonPayload<UploadDatasetRequest>,
) -> Result<axum::response::Response, ApiError> {
    controllers::datasets::upload(State(state), JsonPayload(body)).await
}

pub async fn delete_dataset(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    controllers::datasets::delete(State(state), Path(name)).await
}
