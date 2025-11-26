use actix_web::{HttpResponse, Result, web};

use crate::{
    handlers::home,
    state::AppState,
    views,
    views::models::{MeshJobView, MeshNodeView, PageContext},
};
use refractive_swan_contracts::{MeshJobStatus, MeshJobType, MeshNodeId, NodeCapabilities};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/mesh").route(web::get().to(mesh_page)));
}

pub async fn mesh_page(state: web::Data<AppState>) -> Result<HttpResponse> {
    let chrome = home::view_chrome(&state);
    let nodes = sample_nodes(&state);
    let jobs = sample_jobs();
    let ctx = PageContext {
        mesh_nodes: nodes,
        mesh_jobs: jobs,
        mesh_governance: Some(
            "Governance previews and mesh job submissions are mocked in workbench mode."
                .to_string(),
        ),
        chrome,
        ..PageContext::default()
    };
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_mesh_page(&ctx)))
}

fn sample_nodes(state: &AppState) -> Vec<MeshNodeView> {
    let compliance_mode = state
        .compliance_policy()
        .map(|p| p.mode.as_str().to_string())
        .unwrap_or_else(|| "unknown".into());

    let vector_backend =
        std::env::var("refractive_swan_VECTOR_BACKEND").unwrap_or_else(|_| "mock".to_string());
    let warehouse_backend = std::env::var("refractive_swan_WAREHOUSE_URL")
        .map(|url| {
            if url.starts_with("sqlite://") {
                "sqlite".to_string()
            } else if url.starts_with("postgres://") {
                "postgres".to_string()
            } else {
                "external".to_string()
            }
        })
        .unwrap_or_else(|_| "sqlite".to_string());

    let node_id = std::env::var("refractive_swan_MESH_NODE_ID")
        .map(MeshNodeId)
        .unwrap_or_else(|_| MeshNodeId("workbench-local".into()));
    let capabilities = NodeCapabilities {
        node_id,
        vector_backend: vector_backend.clone(),
        warehouse_backend: warehouse_backend.clone(),
        compliance_mode: compliance_mode.clone(),
        max_dataset_size: 50_000,
        tags: vec!["workbench".into(), "single-node".into()],
    };

    vec![MeshNodeView {
        id: capabilities.node_id.to_string(),
        vector_backend,
        warehouse_backend,
        compliance_mode,
        max_dataset_size: capabilities.max_dataset_size,
        tags: capabilities.tags,
        status: "available".into(),
    }]
}

fn sample_jobs() -> Vec<MeshJobView> {
    vec![
        MeshJobView {
            job_id: "mesh-job-01".into(),
            job_type: format!("{:?}", MeshJobType::EvalDataset),
            status: format!("{:?}", MeshJobStatus::InProgress),
            summary: "Queued eval against gold_pet_ct_small (mock)".into(),
        },
        MeshJobView {
            job_id: "mesh-job-02".into(),
            job_type: format!("{:?}", MeshJobType::AnalyticsQuery),
            status: format!("{:?}", MeshJobStatus::Success),
            summary: "NCIt summary completed on workbench-local".into(),
        },
    ]
}
