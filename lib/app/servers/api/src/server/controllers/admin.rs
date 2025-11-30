use axum::{Json, extract::State, response::IntoResponse};
use refractive_swan_datamart::DatamartError;
use refractive_swan_terminology::codesystem::LicenseTier;
use refractive_swan_terminology::{list_code_systems, list_ontologies};

use crate::{
    types::{
        ComplianceView, DatamartHealthView, FeatureTogglesView, IngestionSummary,
        MeshFeatureTogglesView, TerminologyInsights, VectorStatusView,
    },
    utils::{ApiError, ApiState},
};
use refractive_swan_web_dto::MeshTogglesView;

/// Return terminology registry coverage/insights for dashboards.
pub async fn terminology_insights() -> Result<axum::response::Response, ApiError> {
    let systems = list_code_systems();
    let licensed = systems
        .iter()
        .filter(|meta| matches!(meta.license_tier, LicenseTier::Licensed))
        .count();
    let open = systems
        .iter()
        .filter(|meta| matches!(meta.license_tier, LicenseTier::Open))
        .count();
    let insights = TerminologyInsights {
        total_systems: systems.len(),
        licensed,
        open,
        ontologies: list_ontologies().len(),
    };
    Ok(Json(insights).into_response())
}

/// Expose compliance policy mode and allowed actions.
pub async fn compliance_policy(
    State(state): State<ApiState>,
) -> Result<axum::response::Response, ApiError> {
    let policy = state.plane.policy().clone();
    let actions = refractive_swan_compliance::ComplianceAction::all()
        .into_iter()
        .filter(|action| policy.is_action_allowed(*action))
        .map(|a| format!("{:?}", a))
        .collect();
    let view = ComplianceView {
        mode: policy.mode.as_str().to_string(),
        actions,
    };
    Ok(Json(view).into_response())
}

/// Summarize ingestion/mapping counters for admin panes.
pub async fn ingestion_summary(
    State(state): State<ApiState>,
) -> Result<axum::response::Response, ApiError> {
    let metrics = state.plane.metrics().lock().await.clone();
    let view = IngestionSummary {
        bundles: metrics.bundle_count,
        mapping_count: metrics.mapping_count,
        license_blocked: metrics.license_blocked,
        vector_queries: metrics.vector_queries,
        vector_hits: metrics.vector_hits,
        vector_fallbacks: metrics.vector_fallbacks,
    };
    Ok(Json(view).into_response())
}

/// Surface vector backend status/config for the current node.
pub async fn vector_status(
    State(state): State<ApiState>,
) -> Result<axum::response::Response, ApiError> {
    if let Some(ctx) = state.plane.vector_context() {
        let cfg = ctx.config();
        let view = VectorStatusView {
            backend: format!("{:?}", cfg.backend),
            namespace: cfg.namespace.clone(),
            enabled: true,
        };
        Ok(Json(view).into_response())
    } else {
        Ok(Json(VectorStatusView {
            backend: "mock".into(),
            namespace: "n/a".into(),
            enabled: false,
        })
        .into_response())
    }
}

/// Report datamart health with minimal detail.
pub async fn datamart_health(
    State(state): State<ApiState>,
) -> Result<axum::response::Response, ApiError> {
    let datamart = state.plane.datamart();
    let status = match datamart.ncit_summary().await {
        Ok(_) => DatamartHealthView {
            status: "ok".into(),
            detail: None,
        },
        Err(DatamartError::Disabled) => DatamartHealthView {
            status: "disabled".into(),
            detail: Some("Datamart disabled".into()),
        },
        Err(err) => DatamartHealthView {
            status: "error".into(),
            detail: Some(err.to_string()),
        },
    };
    Ok(Json(status).into_response())
}

/// Return recent admin events captured in process memory.
pub async fn admin_events(
    State(state): State<ApiState>,
) -> Result<axum::response::Response, ApiError> {
    let events = state.admin_events_snapshot().await;
    Ok(Json(events).into_response())
}

/// Describe feature toggles (vector/mesh) for the current node.
pub async fn feature_toggles(
    State(state): State<ApiState>,
) -> Result<axum::response::Response, ApiError> {
    let vector_enabled = state.plane.vector_context().is_some();
    let view = FeatureTogglesView {
        vector_enabled,
        mesh_node_id: state.plane.node_id().to_string(),
    };
    let mesh_view = MeshFeatureTogglesView {
        vector: view.clone(),
        mesh: MeshTogglesView {
            mesh_enabled: true,
            hub_enabled: true,
            hub_url: std::env::var("refractive_swan_API_BASE_URL").ok(),
            node_id: Some(view.mesh_node_id.clone()),
            cache_backend: None,
            warehouse_backend: None,
        },
    };
    Ok(Json(mesh_view).into_response())
}
