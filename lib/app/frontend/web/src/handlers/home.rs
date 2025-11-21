use actix_web::{HttpResponse, Result, web};

use crate::{
    client::BackendClient,
    handlers::eval::render_eval_report_fragment,
    state::AppState,
    view_model::{DEFAULT_EVAL_DATASET, HealthOverview, PageContext},
    views,
};

/// Register landing page and workbench routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(landing_page)));
    cfg.service(web::resource("/map").route(web::get().to(workbench)));
}

/// Landing page handler.
pub async fn landing_page() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_landing_page()))
}

/// Workbench page handler.
pub async fn workbench(state: web::Data<AppState>) -> Result<HttpResponse> {
    let ctx = build_base_context(&state.client, state.dataset_store.as_ref()).await;
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_workbench_page(&ctx)))
}

/// Shared context builder reused across feature handlers so everything pulls from the same backend calls.
pub(crate) async fn build_base_context(
    client: &BackendClient,
    store: &(dyn refractive_swan_eval::DatasetStore + Send + Sync),
) -> PageContext {
    let datasets = client.eval_datasets().await.unwrap_or_default();
    let selected_dataset = datasets
        .first()
        .map(|m| m.name.clone())
        .unwrap_or_else(|| DEFAULT_EVAL_DATASET.to_string());
    let metrics = client.metrics_summary().await.ok();

    let (health, health_error) = match client.health().await {
        Ok(resp) => {
            let status = resp.status;
            let ok = status == "ok";
            let mut error = None;
            if !ok {
                error = Some(format!(
                    "Health endpoint returned status '{}'. See backend logs for details.",
                    status
                ));
            }
            (Some(HealthOverview { status, ok }), error)
        }
        Err(err) => (
            None,
            Some(format!(
                "Health endpoint unreachable: {}",
                err.user_message()
            )),
        ),
    };

    let (eval_report_html, eval_panel_error) =
        match render_eval_report_fragment(client, store, selected_dataset.as_str()).await {
            Ok(html) => (Some(html), None),
            Err(err) => (None, Some(err)),
        };

    PageContext {
        datasets,
        metrics,
        health,
        health_error,
        eval_report_html,
        eval_panel_error,
        selected_eval_dataset: selected_dataset,
        ..PageContext::default()
    }
}
