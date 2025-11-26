use actix_web::{HttpResponse, Result, web};

use crate::{
    handlers::home,
    state::AppState,
    views,
    views::models::{LogEntryKindView, LogEntryView},
};

/// Registers observability routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/observability").route(web::get().to(observability_page)))
        .service(web::resource("/logs/latest").route(web::get().to(logs_fragment)));
}

/// Shows vector/compliance/dataset health plus live logs.
pub async fn observability_page(state: web::Data<AppState>) -> Result<HttpResponse> {
    let ctx = home::build_base_context(&state).await;
    let logs = into_log_views(state.log_snapshot());
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_observability_page(&ctx, &logs)))
}

/// Provides HTMX fragment for the live log panel.
pub async fn logs_fragment(state: web::Data<AppState>) -> Result<HttpResponse> {
    let logs = into_log_views(state.log_snapshot());
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_log_fragment(&logs)))
}

fn into_log_views(entries: Vec<crate::state::LogEntry>) -> Vec<LogEntryView> {
    entries
        .into_iter()
        .map(|entry| LogEntryView {
            id: entry.id.to_string(),
            timestamp: entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
            message: entry.message,
            kind: match entry.kind {
                crate::state::LogKind::Info => LogEntryKindView::Info,
                crate::state::LogKind::NoMatch => LogEntryKindView::NoMatch,
                crate::state::LogKind::Error => LogEntryKindView::Error,
            },
        })
        .collect()
}
