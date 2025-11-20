use actix_web::{HttpResponse, Result, web};

use crate::{
    client::BackendClient,
    handlers::eval::render_eval_report_fragment,
    state::AppState,
    view_model::{DEFAULT_EVAL_DATASET, HealthOverview, PageContext},
    views,
};

/// Register landing page route.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(index)));
}

/// Landing page handler that renders the workbench.
pub async fn index(state: web::Data<AppState>) -> Result<HttpResponse> {
    let ctx = build_base_context(&state.client, &state.dataset_store).await;
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_page(&ctx)))
}

/// Shared context builder reused across feature handlers so everything pulls from the same backend calls.
pub(crate) async fn build_base_context(
    client: &BackendClient,
    store: &dfps_eval::FileDatasetStore,
) -> PageContext {
    let mut ctx = PageContext::default();
    ctx.datasets = client.eval_datasets().await.unwrap_or_default();
    if let Some(first) = ctx.datasets.first() {
        ctx.selected_eval_dataset = first.name.clone();
    }

    ctx.metrics = client.metrics_summary().await.ok();

    match client.health().await {
        Ok(resp) => {
            let status = resp.status;
            let ok = status == "ok";
            ctx.health = Some(HealthOverview {
                status: status.clone(),
                ok,
            });
            if !ok {
                ctx.health_error = Some(format!(
                    "Health endpoint returned status '{status}'. See backend logs for details."
                ));
            }
        }
        Err(err) => {
            ctx.health_error = Some(format!(
                "Health endpoint unreachable: {}",
                err.user_message()
            ));
        }
    }

    let selected_dataset = ctx
        .datasets
        .first()
        .map(|m| m.name.as_str())
        .unwrap_or(DEFAULT_EVAL_DATASET);

    match render_eval_report_fragment(client, store, selected_dataset).await {
        Ok(html) => {
            ctx.eval_report_html = Some(html);
            ctx.selected_eval_dataset = selected_dataset.to_string();
        }
        Err(err) => {
            ctx.eval_panel_error = Some(err);
        }
    }

    ctx
}
