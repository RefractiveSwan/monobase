use actix_web::{HttpResponse, Result, web};
use serde::Deserialize;

use crate::views::layout::ViewChrome;
use crate::{
    client::BackendClient,
    state::AppState,
    views,
    views::models::{DEFAULT_EVAL_DATASET, EvalContext, PageContext},
};
use refractive_swan_eval::report;

/// Registers evaluation routes (page + HTMX fragments).
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/eval").route(web::get().to(eval_page)))
        .service(web::resource("/eval/report").route(web::get().to(eval_report)))
        .service(web::resource("/eval/run").route(web::post().to(eval_run)));
}

pub async fn eval_page(state: web::Data<AppState>) -> Result<HttpResponse> {
    let datasets = state.client.eval_datasets().await.unwrap_or_default();
    let selected = datasets
        .first()
        .map(|m| m.name.clone())
        .unwrap_or_else(|| DEFAULT_EVAL_DATASET.to_string());
    let eval = match state.client.eval_run(&selected, 1).await {
        Ok(run) => Some(EvalContext {
            dataset: selected.clone(),
            summary: run.summary,
        }),
        Err(_) => None,
    };
    let ctx = PageContext {
        datasets,
        selected_eval_dataset: selected,
        eval,
        chrome: ViewChrome::from(&state.config),
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
    let dataset = form.dataset.clone();
    match state.client.eval_run(&dataset, form.top_k).await {
        Ok(run) => Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(views::render_eval_fragment(&run))),
        Err(err) => Ok(HttpResponse::InternalServerError()
            .content_type("text/plain; charset=utf-8")
            .body(format!("Eval run error: {err}"))),
    }
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
