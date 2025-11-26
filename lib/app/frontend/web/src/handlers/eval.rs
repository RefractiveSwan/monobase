use actix_web::{HttpResponse, Result, web};
use serde::Deserialize;

use crate::{
    client::BackendClient,
    handlers::home,
    state::AppState,
    views,
    views::models::{DEFAULT_EVAL_DATASET, EvalContext, EvalJobView, PageContext},
};
use refractive_swan_eval::report;

/// Registers evaluation routes (page + HTMX fragments).
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/eval").route(web::get().to(eval_page)))
        .service(web::resource("/eval/report").route(web::get().to(eval_report)))
        .service(web::resource("/eval/run").route(web::post().to(eval_run)))
        .service(web::resource("/eval/jobs").route(web::get().to(eval_jobs_fragment)))
        .service(web::resource("/eval/calibration").route(web::get().to(eval_calibration_fragment)))
        .service(
            web::resource("/eval/compare")
                .route(web::get().to(eval_compare_fragment))
                .route(web::post().to(eval_compare_submit)),
        );
}

pub async fn eval_page(state: web::Data<AppState>) -> Result<HttpResponse> {
    let datasets = state.client.eval_datasets().await.unwrap_or_default();
    let selected = datasets
        .iter()
        .find(|entry| !entry.disabled)
        .map(|m| m.manifest.name.clone())
        .unwrap_or_else(|| DEFAULT_EVAL_DATASET.to_string());
    let eval = match state.client.eval_run(&selected, 1).await {
        Ok(run) => Some(EvalContext {
            dataset: selected.clone(),
            summary: run.summary,
        }),
        Err(_) => None,
    };
    let eval_jobs = hydrate_eval_jobs(state.get_ref());
    let ctx = PageContext {
        datasets,
        selected_eval_dataset: selected,
        eval,
        eval_jobs,
        chrome: home::view_chrome(&state),
        ..PageContext::default()
    };
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_eval_page(&ctx)))
}

#[derive(Deserialize)]
pub struct EvalReportQuery {
    pub dataset: Option<String>,
}

#[derive(Deserialize)]
pub struct EvalRunForm {
    pub dataset: String,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

fn default_top_k() -> usize {
    1
}

/// Returns the HTMX fragment for the eval report panel.
pub async fn eval_report(
    state: web::Data<AppState>,
    query: web::Query<EvalReportQuery>,
) -> Result<HttpResponse> {
    let dataset = query
        .dataset
        .as_deref()
        .unwrap_or(DEFAULT_EVAL_DATASET)
        .to_string();
    match render_eval_report_fragment(&state.client, state.dataset_store.as_ref(), &dataset).await {
        Ok(html) => Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)),
        Err(err) => Ok(HttpResponse::InternalServerError()
            .content_type("text/plain; charset=utf-8")
            .body(format!("Eval report error: {err}"))),
    }
}

/// Handles eval run submissions triggered by the HTMX eval form.
pub async fn eval_run(
    state: web::Data<AppState>,
    form: web::Form<EvalRunForm>,
) -> Result<HttpResponse> {
    state.enqueue_eval_job(form.dataset.clone(), form.top_k);
    let jobs = hydrate_eval_jobs(state.get_ref());
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_eval_jobs_fragment(&jobs)))
}

/// Builds the eval report fragment via refractive_swan_eval helpers.
pub(crate) async fn render_eval_report_fragment(
    client: &BackendClient,
    store: &(dyn refractive_swan_eval::DatasetStore + Send + Sync),
    dataset: &str,
) -> Result<String, String> {
    let summary = client
        .eval_summary(dataset)
        .await
        .map_err(|err| format!("Backend eval error: {err}"))?;
    let baseline = match report::load_baseline_snapshot_from(store.data_root(), dataset) {
        Ok(snapshot) => Some(snapshot),
        Err(err) => {
            eprintln!("warning: baseline load failed for {dataset}: {err}");
            None
        }
    };
    let html = report::render_html(
        &summary,
        baseline.as_ref().map(|snapshot| &snapshot.summary),
    );
    Ok(html)
}

pub async fn eval_jobs_fragment(state: web::Data<AppState>) -> Result<HttpResponse> {
    let jobs = hydrate_eval_jobs(state.get_ref());
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_eval_jobs_fragment(&jobs)))
}

pub async fn eval_calibration_fragment(state: web::Data<AppState>) -> Result<HttpResponse> {
    let jobs = hydrate_eval_jobs(state.get_ref());
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_eval_calibration_fragment(&jobs)))
}

#[derive(Deserialize, Default)]
pub struct EvalCompareForm {
    #[serde(default)]
    pub job_ids: Vec<String>,
}

pub async fn eval_compare_fragment(state: web::Data<AppState>) -> Result<HttpResponse> {
    let jobs = hydrate_eval_jobs(state.get_ref());
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_eval_compare_fragment(&jobs, None)))
}

pub async fn eval_compare_submit(
    state: web::Data<AppState>,
    form: web::Form<EvalCompareForm>,
) -> Result<HttpResponse> {
    let jobs = hydrate_eval_jobs(state.get_ref());
    let selection = form.job_ids.iter().take(2).cloned().collect::<Vec<_>>();
    let selection = if selection.len() == 2 {
        Some([selection[0].clone(), selection[1].clone()])
    } else {
        None
    };
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_eval_compare_fragment(&jobs, selection)))
}

fn hydrate_eval_jobs(state: &AppState) -> Vec<EvalJobView> {
    state
        .eval_jobs_snapshot()
        .into_iter()
        .map(|entry| EvalJobView {
            id: entry.id.to_string(),
            dataset: entry.dataset,
            submitted_at: entry.submitted_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            status: match entry.status {
                crate::state::EvalJobStatus::Queued => {
                    crate::views::models::EvalJobStatusView::Queued
                }
                crate::state::EvalJobStatus::Running => {
                    crate::views::models::EvalJobStatusView::Running
                }
                crate::state::EvalJobStatus::Completed => {
                    crate::views::models::EvalJobStatusView::Completed
                }
                crate::state::EvalJobStatus::Failed => {
                    crate::views::models::EvalJobStatusView::Failed
                }
            },
            summary: entry.summary.as_ref().map(|summary| summary.into()),
            error: entry.error,
            top_k: entry.top_k,
        })
        .collect()
}
