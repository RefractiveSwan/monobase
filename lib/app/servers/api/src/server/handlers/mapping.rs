use axum::{
    body::Bytes,
    extract::{Query, State},
};

use crate::{
    server::controllers,
    types::MapQuery,
    utils::{ApiError, ApiState},
};

pub async fn map_bundles(
    State(state): State<ApiState>,
    Query(params): Query<MapQuery>,
    body: Bytes,
) -> Result<axum::response::Response, ApiError> {
    controllers::mapping::map_bundles(State(state), Query(params), body).await
}
