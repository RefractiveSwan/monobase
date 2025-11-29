use crate::server::controllers;

/// Mesh-aware health surface that embeds metrics snapshot.
pub async fn health(
    state: axum::extract::State<crate::utils::ApiState>,
) -> impl axum::response::IntoResponse {
    controllers::mesh::health(state).await
}

/// Advertise node capabilities to hubs/frontends.
pub async fn capabilities(
    state: axum::extract::State<crate::utils::ApiState>,
) -> impl axum::response::IntoResponse {
    controllers::mesh::capabilities(state).await
}

/// Governance summary for the node.
pub async fn governance(
    state: axum::extract::State<crate::utils::ApiState>,
) -> impl axum::response::IntoResponse {
    controllers::mesh::governance(state).await
}

/// Run a mesh job descriptor.
pub async fn job(
    state: axum::extract::State<crate::utils::ApiState>,
    payload: axum::Json<refractive_swan_contracts::MeshJobDescriptor>,
) -> impl axum::response::IntoResponse {
    controllers::mesh::mesh_job(state, payload).await
}
