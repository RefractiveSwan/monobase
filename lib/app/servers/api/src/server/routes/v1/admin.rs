use axum::{Router, routing::get};

use crate::server::ApiState;
use crate::server::handlers::admin;

/// Admin/insights endpoints (terminology/compliance/ingestion/vector/datamart/toggles).
pub fn router() -> Router<ApiState> {
    Router::new()
        .route(
            "/admin/terminology/insights",
            get(admin::terminology_insights),
        )
        .route("/admin/compliance/policy", get(admin::compliance_policy))
        .route("/admin/ingestion/summary", get(admin::ingestion_summary))
        .route("/admin/vector/status", get(admin::vector_status))
        .route("/admin/datamart/health", get(admin::datamart_health))
        .route("/admin/events", get(admin::admin_events))
        .route("/admin/toggles", get(admin::feature_toggles))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use serde_json::Value;
    use tower::Service;

    use crate::server::router_with_state;
    use crate::utils::ApiState;

    fn app() -> axum::Router {
        router_with_state(ApiState::default())
    }

    #[tokio::test]
    async fn admin_toggles_include_mesh_view() {
        let mut app = app();
        let resp = app
            .call(
                Request::builder()
                    .uri("/admin/toggles")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert!(json.get("vector").is_some());
        let mesh = json.get("mesh").and_then(Value::as_object).unwrap();
        assert!(mesh.get("mesh_enabled").is_some());
        assert!(mesh.get("hub_enabled").is_some());
        assert!(mesh.get("node_id").is_some());
    }
}
