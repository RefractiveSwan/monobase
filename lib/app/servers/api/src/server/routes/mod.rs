//! Router assembly for the API server.
//!
//! - `/api/v1/**` — current ReST surface (health/metrics/analytics/mapping/eval/datasets/admin)
//! - `/api/v2/**` — GraphQL placeholder (to be populated when the GQL schema lands)
//! - `/api/v3/**` — gRPC placeholder (reserved for tonic/gRPC services)
//!
//! Backward compatibility:
//! - `/api/**` is kept as an alias for the v1 ReST routes.
//! - Root health/metrics routes remain exposed for probes.

use axum::Router;

use super::ApiState;

mod v1;
mod v2;
mod v3;

/// Build the top-level router with versioned namespaces and backward-compatible aliases.
pub fn build_router() -> Router<ApiState> {
    let v1_rest: Router<ApiState> = v1::router();
    let v2_graphql: Router<ApiState> = v2::router();
    let v3_grpc: Router<ApiState> = v3::router();

    Router::new()
        .nest("/api/v1", v1_rest.clone())
        .nest("/api/v2", v2_graphql)
        .nest("/api/v3", v3_grpc)
        .nest("/api", v1_rest.clone())
        // Keep health/metrics/etc. available at root for probes/legacy clients.
        .merge(v1_rest)
}
