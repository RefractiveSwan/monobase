use axum::{
    Router,
    routing::{get, post},
};

use crate::server::ApiState;
use crate::server::handlers;

/// Mesh endpoints for node capabilities, health, and governance.
pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/mesh/health", get(handlers::mesh::health))
        .route("/mesh/capabilities", get(handlers::mesh::capabilities))
        .route("/mesh/governance", get(handlers::mesh::governance))
        .route("/mesh/job", post(handlers::mesh::job))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use refractive_swan_contracts::NodeCapabilities;
    use refractive_swan_contracts::{MeshJobDescriptor, MeshJobType};
    use refractive_swan_mesh_dto::MeshNodeId;
    use refractive_swan_mesh_node::NodePlaneConfig;
    use serde_json::Value;
    use tower::Service;

    use crate::server::router_with_state;
    use crate::utils::ApiState;

    fn app_with_node(id: &str) -> axum::Router {
        let config = NodePlaneConfig::from_env_with_node_id(
            "app.web.api",
            MeshNodeId::from_string(id.to_string()),
        )
        .expect("plane config");
        let state = ApiState::from_plane_config(config);
        router_with_state(state)
    }

    #[tokio::test]
    async fn mesh_capabilities_reports_configured_node() {
        let mut app = app_with_node("mesh-test-node");
        let response = app
            .call(
                Request::builder()
                    .uri("/mesh/capabilities")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let caps: NodeCapabilities = serde_json::from_slice(&body).unwrap();
        assert_eq!(caps.node_id.as_str(), "mesh-test-node");
        assert_eq!(caps.warehouse_backend, "disabled");
        assert!(!caps.compliance_mode.is_empty());
    }

    #[tokio::test]
    async fn mesh_health_includes_metrics_snapshot() {
        let mut app = app_with_node("mesh-health-node");
        let response = app
            .call(
                Request::builder()
                    .uri("/mesh/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value.get("status").and_then(Value::as_str), Some("ok"));
        assert_eq!(
            value.get("node_id").and_then(Value::as_str),
            Some("mesh-health-node")
        );
        assert!(value.get("metrics").is_some());
        assert_eq!(
            value.get("cache_backend").and_then(Value::as_str),
            Some("disabled")
        );
        assert_eq!(
            value
                .get("cache")
                .and_then(|v| v.get("status"))
                .and_then(Value::as_str),
            Some("disabled")
        );
    }

    #[tokio::test]
    async fn mesh_governance_surfaces_policy_mode() {
        let mut app = app_with_node("mesh-governance-node");
        let response = app
            .call(
                Request::builder()
                    .uri("/mesh/governance")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value.get("mode").and_then(Value::as_str), Some("internal"));
        assert!(value.get("allowed_actions").is_some());
        assert!(value.get("allowed_tiers").is_some());
    }

    #[tokio::test]
    async fn mesh_job_executes_node_introspection() {
        let mut app = app_with_node("mesh-job-node");
        let descriptor = MeshJobDescriptor {
            job_id: "job-1".into(),
            job_type: MeshJobType::NodeIntrospection,
            parameters: serde_json::Value::default(),
            governance_context: None,
        };
        let response = app
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
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value.get("status").and_then(Value::as_str), Some("success"));
        assert!(value.get("output").is_some());
    }

    #[tokio::test]
    async fn mesh_job_mapping_health_reports_counts() {
        let mut app = app_with_node("mesh-health-job-node");
        let descriptor = MeshJobDescriptor {
            job_id: "job-2".into(),
            job_type: MeshJobType::MappingHealthCheck,
            parameters: serde_json::Value::default(),
            governance_context: None,
        };
        let response = app
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
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value.get("status").and_then(Value::as_str), Some("success"));
        let output = value.get("output").and_then(Value::as_object).unwrap();
        assert!(output.get("auto_mapped").and_then(Value::as_u64).is_some());
        assert_eq!(
            output.get("vector_backend").and_then(Value::as_str),
            Some("disabled")
        );
        assert!(output.get("datamart_persisted").is_some());
        assert!(output.get("vector_health").is_some());
        assert!(output.get("warehouse_health").is_some());
        assert_eq!(
            output.get("cache_backend").and_then(Value::as_str),
            Some("disabled")
        );
        assert!(output.get("cache_health").is_some());
    }
}
