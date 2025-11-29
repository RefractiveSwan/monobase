use crate::server::controllers;

pub async fn list_nodes(
    state: axum::extract::State<crate::utils::ApiState>,
) -> impl axum::response::IntoResponse {
    controllers::hub::list_nodes(state).await
}

pub async fn node_details(
    state: axum::extract::State<crate::utils::ApiState>,
    path: axum::extract::Path<String>,
) -> impl axum::response::IntoResponse {
    controllers::hub::node_details(state, path).await
}

pub async fn federated_ncit_summary(
    state: axum::extract::State<crate::utils::ApiState>,
) -> impl axum::response::IntoResponse {
    controllers::hub::federated_ncit_summary(state).await
}

pub async fn federated_eval(
    state: axum::extract::State<crate::utils::ApiState>,
    query: axum::extract::Query<controllers::hub::EvalQuery>,
) -> impl axum::response::IntoResponse {
    controllers::hub::federated_eval(state, query).await
}
