use axum::{body::to_bytes, Router, routing::{get, post}};

use crate::server::ApiState;
use crate::server::handlers;
#[cfg(test)]
use crate::server::router_with_state;

#[cfg(test)]
use crate::utils::ApiState as TestState;
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
        let node_id = value["nodes"][0]["node_id"]
            .as_str()
            .expect("node_id present");

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
    }
}
