use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use refractive_swan_api::{router_with_state as api_router, utils::ApiState};
use refractive_swan_contracts::{MeshJobDescriptor, MeshJobType};
use refractive_swan_mesh_dto::MeshNodeId;
use refractive_swan_mesh_node::NodePlaneConfig;
use refractive_swan_test_suite::ScopedEnvVar;
use tower::Service;

/// Build an in-process app with a given node_id.
fn app_with_node(id: &str, _env: Option<ScopedEnvVar>) -> Router {
    let config = NodePlaneConfig::from_env_with_node_id(
        "app.web.api",
        MeshNodeId::from_string(id.to_string()),
    )
    .expect("plane config");
    let state = ApiState::from_plane_config(config);
    api_router(state)
}

/// Smoke test: call /mesh/job on two nodes and hit hub-style federated endpoints.
#[tokio::test]
async fn mesh_nodes_handle_jobs_and_health() {
    let mut node_a = app_with_node("mesh-int-node-a", None);
    let mut node_b = app_with_node("mesh-int-node-b", None);

    // health on both nodes
    for app in [&mut node_a, &mut node_b] {
        let resp = app
            .call(
                Request::builder()
                    .uri("/mesh/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // run a MappingHealthCheck job on node_a
    let descriptor = MeshJobDescriptor {
        job_id: "job-health".into(),
        job_type: MeshJobType::MappingHealthCheck,
        parameters: serde_json::Value::default(),
        governance_context: None,
    };
    let resp = node_a
        .call(
            Request::builder()
                .method("POST")
                .uri("/mesh/job")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&descriptor).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // simulate hub analytics query against node_b via mesh job
    let descriptor = MeshJobDescriptor {
        job_id: "job-analytics".into(),
        job_type: MeshJobType::AnalyticsQuery,
        parameters: serde_json::json!({"query_type":"ncit_summary"}),
        governance_context: None,
    };
    let resp = node_b
        .call(
            Request::builder()
                .method("POST")
                .uri("/mesh/job")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&descriptor).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(resp.status().is_success());
}
