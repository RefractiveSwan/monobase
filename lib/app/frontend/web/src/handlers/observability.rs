use actix_web::{HttpResponse, Result, web};
use chrono::{DateTime, Utc};
use log::warn;
use refractive_swan_web_dto::{AdminEvent, AdminEventKind};

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
    let logs = collect_log_views(state.get_ref()).await;
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_observability_page(&ctx, &logs)))
}

/// Provides HTMX fragment for the live log panel.
pub async fn logs_fragment(state: web::Data<AppState>) -> Result<HttpResponse> {
    let logs = collect_log_views(state.get_ref()).await;
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_log_fragment(&logs)))
}

async fn collect_log_views(state: &AppState) -> Vec<LogEntryView> {
    let admin_events = match state.client.admin_events().await {
        Ok(events) => events,
        Err(err) => {
            warn!(
                target: "refractive_swan_web_frontend.observability",
                "failed to load admin events: {}",
                err
            );
            Vec::new()
        }
    };
    into_log_views(state.log_snapshot(), admin_events)
}

fn into_log_views(
    entries: Vec<crate::state::LogEntry>,
    admin_events: Vec<AdminEvent>,
) -> Vec<LogEntryView> {
    let mut merged: Vec<(DateTime<Utc>, LogEntryView)> = Vec::new();
    for entry in entries {
        let timestamp = entry.timestamp;
        merged.push((
            timestamp,
            LogEntryView {
                id: entry.id.to_string(),
                timestamp: timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                message: entry.message,
                kind: match entry.kind {
                    crate::state::LogKind::Info => LogEntryKindView::Info,
                    crate::state::LogKind::NoMatch => LogEntryKindView::NoMatch,
                    crate::state::LogKind::Error => LogEntryKindView::Error,
                },
            },
        ));
    }
    for event in admin_events {
        let timestamp = event.timestamp;
        merged.push((
            timestamp,
            LogEntryView {
                id: event.id.to_string(),
                timestamp: timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                message: format_admin_message(&event),
                kind: LogEntryKindView::Info,
            },
        ));
    }
    merged.sort_by(|a, b| b.0.cmp(&a.0));
    merged.into_iter().map(|(_, view)| view).collect()
}

fn format_admin_message(event: &AdminEvent) -> String {
    let kind = match event.kind {
        AdminEventKind::DatasetUpload => "dataset_upload",
        AdminEventKind::DatasetDelete => "dataset_delete",
        AdminEventKind::DatasetRefresh => "dataset_refresh",
        AdminEventKind::DatasetEnable => "dataset_enable",
        AdminEventKind::DatasetDisable => "dataset_disable",
        AdminEventKind::Compliance => "compliance",
        AdminEventKind::Vector => "vector",
        AdminEventKind::Datamart => "datamart",
        AdminEventKind::Ingestion => "ingestion",
        AdminEventKind::Toggle => "toggle",
    };
    let node = event
        .mesh_node_id
        .as_ref()
        .map(|id| format!(" node={}", id))
        .unwrap_or_default();
    format!("[admin:{kind}]{node} {}", event.message)
}
