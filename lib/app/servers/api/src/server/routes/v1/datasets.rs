use axum::{
    Router,
    routing::{delete, post},
};

use crate::server::ApiState;
use crate::server::handlers::datasets;

/// Dataset admin endpoints.
pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/datasets/refresh", post(datasets::refresh_datasets))
        .route("/datasets/upload", post(datasets::upload_dataset))
        .route("/datasets/:name", delete(datasets::delete_dataset))
}
