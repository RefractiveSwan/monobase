use actix_web::{HttpResponse, Result, web};

use crate::{
    handlers::home,
    state::AppState,
    views,
    views::models::{DEFAULT_EVAL_DATASET, MeshJobView, MeshNodeView, PageContext},
};
use refractive_swan_contracts::{MeshJobStatus, MeshJobType, MeshNodeId, NodeCapabilities};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/mesh").route(web::get().to(mesh_page)))
        .service(web::resource("/mesh/hub/analytics").route(web::get().to(hub_analytics_fragment)))
        .service(web::resource("/mesh/hub/eval").route(web::get().to(hub_eval_fragment)))
        .service(web::resource("/mesh/admin-events").route(web::get().to(admin_events_fragment)));
}

pub async fn mesh_page(state: web::Data<AppState>) -> Result<HttpResponse> {
    let chrome = home::view_chrome(&state);
    let mut nodes = Vec::new();
    let mut jobs = Vec::new();
    let mut mesh_governance = mesh_toggles_summary(&state).await;
    let mut hub_analytics = None;
    let mut hub_eval = None;
    let mut mesh_alerts: Vec<String> = Vec::new();

    match mesh_nodes(&state).await {
        Ok(actual) => nodes = actual,
        Err(err) => mesh_alerts.push(err),
    };

    match hub_jobs(&state).await {
        Ok(mut hub) => {
            if nodes.is_empty() {
                nodes = hub.nodes;
            }
            jobs.append(&mut hub.jobs);
            if mesh_governance.is_none() {
                mesh_governance = hub.note;
            }
            hub_analytics = hub.analytics;
            hub_eval = hub.eval_summary;
        }
        Err(err) => {
            mesh_alerts.push(err);
        }
    }

    if nodes.is_empty() {
        nodes = sample_nodes(&state);
    }
    if jobs.is_empty() {
        jobs = sample_jobs();
    }

    let admin_events = state
        .client
        .admin_events()
        .await
        .unwrap_or_else(|_| Vec::new());
    let ctx = PageContext {
        mesh_nodes: nodes,
        mesh_jobs: jobs,
        mesh_governance,
        mesh_admin_events: admin_events,
        hub_analytics,
        hub_eval,
        mesh_alerts,
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
    let cache_backend = std::env::var("refractive_swan_CACHE_BACKEND").ok();
    let cache_status = cache_backend.as_ref().map(|_| "unknown".to_string());

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
        cache_backend: cache_backend.clone(),
        cache_status,
        last_seen_ms: None,
        compliance_mode,
        max_dataset_size: capabilities.max_dataset_size,
        tags: capabilities.tags,
        status: "available".into(),
        dp_budget_remaining: None,
        dp_budget_status: None,
        metrics: None,
    }]
}

async fn mesh_nodes(state: &AppState) -> Result<Vec<MeshNodeView>, String> {
    let capabilities = state
        .client
        .mesh_capabilities()
        .await
        .map_err(|err| err.user_message())?;
    let health = state
        .client
        .health()
        .await
        .map_err(|err| err.user_message())?;
    let cache_status = health
        .cache
        .as_ref()
        .and_then(|c| c.get("status"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let dp_budget_remaining = health.dp_budget_remaining.or_else(|| {
        health
            .cache
            .as_ref()
            .and_then(|c| c.get("dp_budget_remaining"))
            .and_then(|v| v.as_f64())
    });
    let dp_budget_status = health.dp_budget_status.or_else(|| {
        health
            .cache
            .as_ref()
            .and_then(|c| c.get("dp_budget_status"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    });
    Ok(vec![MeshNodeView {
        id: capabilities.node_id.to_string(),
        vector_backend: capabilities.vector_backend,
        warehouse_backend: capabilities.warehouse_backend,
        cache_backend: health.cache_backend.clone(),
        cache_status,
        last_seen_ms: None,
        compliance_mode: capabilities.compliance_mode,
        max_dataset_size: capabilities.max_dataset_size,
        tags: capabilities.tags,
        status: health.status,
        dp_budget_remaining,
        dp_budget_status,
        metrics: None,
    }])
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

async fn mesh_toggles_summary(state: &AppState) -> Option<String> {
    match state.client.node_toggles().await {
        Ok(toggles) => {
            let cache = toggles.mesh.cache_backend.as_deref().unwrap_or("disabled");
            let hub = toggles.mesh.hub_url.as_deref().unwrap_or("hub url not set");
            Some(format!(
                "Mesh enabled: {} (node_id={}); hub: enabled={} ({hub}); cache={cache}; warehouse={}",
                toggles.mesh.mesh_enabled,
                toggles.mesh.node_id.unwrap_or_else(|| "unknown".into()),
                toggles.mesh.hub_enabled,
                toggles
                    .mesh
                    .warehouse_backend
                    .unwrap_or_else(|| "unknown".into())
            ))
        }
        Err(err) => Some(format!("Mesh toggles unavailable: {}", err.user_message())),
    }
}

struct HubJobSnapshot {
    nodes: Vec<MeshNodeView>,
    jobs: Vec<MeshJobView>,
    note: Option<String>,
    analytics: Option<refractive_swan_web_dto::AnalyticsSummaryResponse>,
    eval_summary: Option<refractive_swan_web_dto::EvalSummary>,
}

pub async fn hub_analytics_fragment(state: web::Data<AppState>) -> Result<HttpResponse> {
    let mut alerts = Vec::new();
    let analytics = match state.client.hub_ncit_summary().await {
        Ok(summary) => Some(summary),
        Err(err) => {
            alerts.push(format!("Hub analytics failed: {}", err.user_message()));
            None
        }
    };
    let ctx = PageContext {
        hub_analytics: analytics,
        mesh_alerts: alerts,
        ..PageContext::default()
    };
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_hub_analytics_fragment(&ctx)))
}

pub async fn hub_eval_fragment(
    state: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse> {
    let dataset = query
        .get("dataset")
        .cloned()
        .unwrap_or_else(|| DEFAULT_EVAL_DATASET.to_string());
    let mut alerts = Vec::new();
    let eval = match state.client.run_federated_eval(&dataset).await {
        Ok(summary) => Some(summary.aggregated),
        Err(err) => {
            alerts.push(format!(
                "Hub federated eval failed for `{}`: {}",
                dataset,
                err.user_message()
            ));
            None
        }
    };
    let ctx = PageContext {
        hub_eval: eval,
        mesh_alerts: alerts,
        ..PageContext::default()
    };
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_hub_eval_fragment(&ctx, &dataset)))
}

pub async fn admin_events_fragment(state: web::Data<AppState>) -> Result<HttpResponse> {
    let events = state
        .client
        .admin_events()
        .await
        .unwrap_or_else(|_| Vec::new());
    let ctx = PageContext {
        mesh_admin_events: events,
        ..PageContext::default()
    };
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_admin_events_fragment(&ctx)))
}

async fn hub_jobs(state: &AppState) -> Result<HubJobSnapshot, String> {
    let hub_nodes = state
        .client
        .hub_nodes()
        .await
        .map_err(|err| err.user_message())?;

    let nodes = hub_nodes
        .nodes
        .into_iter()
        .map(|node| MeshNodeView {
            id: node.node_id.to_string(),
            vector_backend: node.vector_backend,
            warehouse_backend: node.warehouse_backend,
            cache_backend: node.cache_backend,
            cache_status: node.status.clone(),
            compliance_mode: node.compliance_mode,
            max_dataset_size: node
                .metrics
                .as_ref()
                .map(|m| m.bundle_count as u64)
                .unwrap_or(0),
            tags: node.tags,
            status: node.status.unwrap_or_else(|| "unknown".into()),
            last_seen_ms: node.last_seen_ms,
            dp_budget_remaining: None,
            dp_budget_status: None,
            metrics: node.metrics,
        })
        .collect();

    let mut jobs = Vec::new();
    let mut analytics: Option<refractive_swan_web_dto::AnalyticsSummaryResponse> = None;
    let mut eval_summary: Option<refractive_swan_web_dto::EvalSummary> = None;
    match state.client.hub_ncit_summary().await {
        Ok(summary) => {
            analytics = Some(summary.clone());
            jobs.push(MeshJobView {
                job_id: "hub-ncit-summary".into(),
                job_type: "AnalyticsQuery".into(),
                status: format!("{:?}", MeshJobStatus::Success),
                summary: format!("Aggregated {} NCIt summary rows", summary.rows.len()),
            });
        }
        Err(err) => jobs.push(MeshJobView {
            job_id: "hub-ncit-summary".into(),
            job_type: "AnalyticsQuery".into(),
            status: format!("{:?}", MeshJobStatus::Failed),
            summary: format!("Hub NCIt summary failed: {}", err.user_message()),
        }),
    }

    match state.client.run_federated_eval(DEFAULT_EVAL_DATASET).await {
        Ok(eval) => {
            jobs.push(MeshJobView {
                job_id: format!("hub-eval-{}", eval.dataset),
                job_type: "EvalDataset".into(),
                status: format!("{:?}", MeshJobStatus::Success),
                summary: format!(
                    "Federated eval `{}` aggregated {} cases across {} node(s)",
                    eval.dataset,
                    eval.aggregated.total_cases,
                    eval.per_node.len()
                ),
            });
            eval_summary = Some(eval.aggregated.clone());
        }
        Err(err) => jobs.push(MeshJobView {
            job_id: format!("hub-eval-{}", DEFAULT_EVAL_DATASET),
            job_type: "EvalDataset".into(),
            status: format!("{:?}", MeshJobStatus::Failed),
            summary: format!("Federated eval failed: {}", err.user_message()),
        }),
    }

    Ok(HubJobSnapshot {
        nodes,
        jobs,
        note: Some("Hub jobs executed against /hub/jobs/* endpoints".into()),
        analytics,
        eval_summary,
    })
}
