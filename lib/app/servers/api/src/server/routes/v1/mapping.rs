use axum::{Router, routing::post};

use crate::server::ApiState;
use crate::server::handlers::mapping;

/// Mapping endpoints.
pub fn router() -> Router<ApiState> {
    Router::new().route("/map-bundles", post(mapping::map_bundles))
}
