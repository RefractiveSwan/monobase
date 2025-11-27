use crate::server::ApiState;
use axum::Router;

/// v3 gRPC surface placeholder.
///
/// Reserved for tonic/gRPC adapters; today this keeps the namespace stable
/// without registering handlers.
pub fn router() -> Router<ApiState> {
    Router::new()
}
