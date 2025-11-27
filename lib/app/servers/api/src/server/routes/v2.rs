use crate::server::ApiState;
use axum::Router;

/// v2 GraphQL surface placeholder.
///
/// Reserved for a future GraphQL schema; currently returns an empty router so
/// the namespace is wired without exposing endpoints.
pub fn router() -> Router<ApiState> {
    Router::new()
}
