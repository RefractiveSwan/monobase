use axum::{
    Router,
    routing::{get, post},
};

use crate::server::ApiState;
use crate::server::handlers;
#[cfg(test)]
use crate::server::router_with_state;

#[cfg(test)]
use crate::utils::ApiState as TestState;
#[cfg(test)]
use axum::body::to_bytes;
#[cfg(test)]
use axum::http::StatusCode;
#[cfg(test)]
use axum::{body::Body, http::Request};
#[cfg(test)]
use serde_json::Value;
#[cfg(test)]
use tower::Service;

/// Initial hub endpoints (stubbed federated jobs + node list).
pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/hub/nodes", get(handlers::hub::list_nodes))
        .route("/hub/nodes/:id", get(handlers::hub::node_details))
        .route(
            "/hub/jobs/analytics/ncit-summary",
            post(handlers::hub::federated_ncit_summary),
        )
        .route("/hub/jobs/eval", post(handlers::hub::federated_eval))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> axum::Router {
        router_with_state(TestState::default())
    }

    #[tokio::test]
    async fn hub_nodes_endpoints_work() {
        let mut app = app();
        let resp = app
            .call(
                Request::builder()
                    .uri("/hub/nodes")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert!(value.get("nodes").is_some());
        let nodes = value.get("nodes").and_then(|v| v.as_array()).unwrap();
        let first = nodes.first().unwrap();
        let node_id = first["node_id"].as_str().expect("node_id present");
        assert_eq!(first["vector_backend"].as_str(), Some("disabled"));
        assert!(first.get("metrics").is_some());

        // Node detail
        let detail_resp = app
            .call(
                Request::builder()
                    .uri(format!("/hub/nodes/{node_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(detail_resp.status().is_success());
    }

    #[tokio::test]
    async fn hub_jobs_dispatch_returns_results() {
        let mut app = app();
        let resp = app
            .call(
                Request::builder()
                    .method("POST")
                    .uri("/hub/jobs/analytics/ncit-summary")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let summary: Value = serde_json::from_slice(&body).unwrap();
        assert!(summary.get("rows").is_some());

        let eval_resp = app
            .call(
                Request::builder()
                    .method("POST")
                    .uri("/hub/jobs/eval?dataset=bronze_pet_ct_small")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(eval_resp.status(), StatusCode::OK);
        let eval_body = to_bytes(eval_resp.into_body(), usize::MAX).await.unwrap();
        let eval_json: Value = serde_json::from_slice(&eval_body).unwrap();
        assert_eq!(
            eval_json.get("dataset").and_then(Value::as_str),
            Some("bronze_pet_ct_small")
        );
        assert!(eval_json.get("per_node").is_some());
        assert!(eval_json.get("aggregated").is_some());
    }
}
