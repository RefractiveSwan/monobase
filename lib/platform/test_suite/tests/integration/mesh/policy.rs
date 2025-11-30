use axum::body::Body;
use axum::http::{Request, StatusCode};
use refractive_swan_contracts::{MeshJobDescriptor, MeshJobStatus, MeshJobType};
use refractive_swan_mesh_dto::MeshNodeId;
use refractive_swan_mesh_node::NodePlaneConfig;
use refractive_swan_test_suite::scoped_env_var;

use refractive_swan_api::{utils::ApiState, router_with_state as api_router};
use tower::Service;

/// If DP budget is zero, governance should deny a DP-consuming job.
#[tokio::test]
async fn dp_budget_denial_is_reported() {
    let _budget = scoped_env_var("refractive_swan_DP_BUDGET_DAILY", "0.0");
    let config = NodePlaneConfig::from_env_with_node_id(
        "app.web.api",
        MeshNodeId::from_string("mesh-dp-deny".into()),
    )
    .expect("plane config");
    let state = ApiState::from_plane_config(config);
    let mut app = api_router(state);

    let descriptor = MeshJobDescriptor {
        job_id: "dp-deny".into(),
        job_type: MeshJobType::AnalyticsQuery,
        parameters: serde_json::json!({"query_type":"ncit_summary", "expected_cardinality": 1000}),
        governance_context: None,
    };

    let resp = app
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
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: refractive_swan_contracts::MeshJobResult =
        serde_json::from_slice(&body).unwrap();
    let code = result
        .error
        .as_ref()
        .map(|e| e.code.as_str())
        .unwrap_or_default();
    assert!(
        code.contains("dp_budget"),
        "expected dp_budget error, got status={:?} code={code}",
        result.status
    );
}
