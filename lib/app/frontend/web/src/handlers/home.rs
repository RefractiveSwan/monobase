use actix_web::{HttpResponse, Result, web};

use crate::{
    handlers::eval::render_eval_report_fragment,
    state::{AppState, MappingHistoryEntry, MappingHistoryStatus},
    views,
    views::layout::{PageAnnouncement, ViewChrome},
    views::models::{
        ComplianceActionView, CompliancePolicyView, DEFAULT_EVAL_DATASET, HealthOverview,
        IngestionStatsView, MappingHistoryStatusView, MappingHistoryView, PageContext,
        TerminologyCodeSystemView, TerminologyInsightsView, TerminologyOntologyView,
        VectorConfigView,
    },
};
use refractive_swan_compliance::{ComplianceAction, Policy};
use refractive_swan_contracts::pipeline::ValidationSeverity;
use refractive_swan_terminology::{codesystem::LicenseTier, list_code_systems, list_ontologies};
use std::env;

/// Register landing page and workbench routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(landing_page)));
    cfg.service(web::resource("/map").route(web::get().to(workbench)));
}

/// Landing page handler.
pub async fn landing_page(state: web::Data<AppState>) -> Result<HttpResponse> {
    let chrome = view_chrome(&state);
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_landing_page(&chrome)))
}

/// Workbench page handler.
pub async fn workbench(state: web::Data<AppState>) -> Result<HttpResponse> {
    let ctx = build_base_context(&state).await;
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(views::render_workbench_page(&ctx)))
}

/// Shared context builder reused across feature handlers so everything pulls from the same backend calls.
pub(crate) async fn build_base_context(state: &AppState) -> PageContext {
    let client = &state.client;
    let store = state.dataset_store.as_ref();
    let datasets = client.eval_datasets().await.unwrap_or_default();
    let selected_dataset = datasets
        .iter()
        .find(|entry| !entry.disabled)
        .map(|m| m.manifest.name.clone())
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

    let history_entries = state.history_snapshot();
    let mapping_history = hydrate_mapping_history(&history_entries);
    let ingestion_stats = build_ingestion_stats(&history_entries);

    PageContext {
        datasets,
        metrics,
        health,
        health_error,
        eval_report_html,
        eval_panel_error,
        selected_eval_dataset: selected_dataset,
        chrome: view_chrome(state),
        mapping_history,
        terminology_insights: Some(build_terminology_insights()),
        compliance_policy: state
            .compliance_policy()
            .map(|policy| build_compliance_view(&policy)),
        ingestion_stats,
        vector_config: build_vector_config(state),
        ..PageContext::default()
    }
}

pub(crate) fn view_chrome(state: &AppState) -> ViewChrome {
    let announcements = vec![
        PageAnnouncement {
            text: "Upload history preview available – tap a recent run to reload results."
                .to_string(),
            href: state.config.docs_url.clone(),
        },
        PageAnnouncement {
            text: "Eval Control Center now includes tiered datasets + queue/compare panels."
                .to_string(),
            href: Some("/eval".to_string()),
        },
        PageAnnouncement {
            text: "Dataset admin page covers uploads, fixtures, regression smoke tests, and maintenance."
                .to_string(),
            href: Some("/admin/datasets".to_string()),
        },
    ];
    ViewChrome::from(&state.config).with_announcements(announcements)
}

pub(crate) fn hydrate_mapping_history(entries: &[MappingHistoryEntry]) -> Vec<MappingHistoryView> {
    entries
        .iter()
        .map(|entry| {
            let validation_errors = entry
                .validation_reports
                .iter()
                .map(|report| {
                    report
                        .issues
                        .iter()
                        .filter(|issue| matches!(issue.severity, ValidationSeverity::Error))
                        .count()
                })
                .sum();
            MappingHistoryView {
                id: entry.id.to_string(),
                submitted_at: entry.submitted_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                duration_ms: entry.duration_ms as u64,
                status: match entry.status {
                    MappingHistoryStatus::Success => MappingHistoryStatusView::Success,
                    MappingHistoryStatus::Error => MappingHistoryStatusView::Error,
                },
                mapped: entry.mapped,
                error: entry.error.clone(),
                validation_errors,
            }
        })
        .collect()
}

fn build_terminology_insights() -> TerminologyInsightsView {
    let systems = list_code_systems();
    let licensed = systems
        .iter()
        .filter(|meta| matches!(meta.license_tier, LicenseTier::Licensed))
        .count();
    let open = systems
        .iter()
        .filter(|meta| matches!(meta.license_tier, LicenseTier::Open))
        .count();
    let code_systems = systems
        .iter()
        .map(|meta| TerminologyCodeSystemView {
            name: meta.name.to_string(),
            url: meta.url.to_string(),
            license_tier: meta.license_tier.as_str().to_string(),
            source_kind: meta.source_kind.as_str().to_string(),
        })
        .collect();
    let ontologies = list_ontologies()
        .iter()
        .map(|ont| TerminologyOntologyView {
            id: ont.id.to_string(),
            name: ont.name.to_string(),
            iri: ont.iri.to_string(),
            description: ont.description.to_string(),
        })
        .collect();
    TerminologyInsightsView {
        total_systems: systems.len(),
        licensed,
        open,
        code_systems,
        ontologies,
    }
}

fn build_compliance_view(policy: &Policy) -> CompliancePolicyView {
    let actions = ComplianceAction::all()
        .into_iter()
        .map(|action| {
            let allowed = policy.is_action_allowed(action);
            let tiers = policy
                .allowed_tiers_for(action)
                .map(|set| {
                    set.iter()
                        .map(|tier| tier.as_str().to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            ComplianceActionView {
                action: compliance_action_label(action).to_string(),
                allowed,
                tiers,
            }
        })
        .collect();
    CompliancePolicyView {
        mode: policy.mode.as_str().to_string(),
        actions,
    }
}

fn compliance_action_label(action: ComplianceAction) -> &'static str {
    match action {
        ComplianceAction::Ingest => "Ingest",
        ComplianceAction::Map => "Map",
        ComplianceAction::Export => "Export",
    }
}

pub(crate) fn build_vector_config(state: &AppState) -> VectorConfigView {
    let env_enabled = env::var("refractive_swan_VECTOR_ENABLED")
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(true);
    VectorConfigView {
        mode: state.vector_mode(),
        backend: env::var("refractive_swan_VECTOR_BACKEND")
            .ok()
            .filter(|s| !s.is_empty()),
        namespace: env::var("refractive_swan_VECTOR_NAMESPACE")
            .ok()
            .filter(|s| !s.is_empty()),
        env_enabled,
    }
}

fn build_ingestion_stats(entries: &[MappingHistoryEntry]) -> Option<IngestionStatsView> {
    if entries.is_empty() {
        return None;
    }
    let mut errors = 0;
    let mut warnings = 0;
    let mut info_count = 0;
    for entry in entries {
        for report in &entry.validation_reports {
            for issue in &report.issues {
                match issue.severity {
                    ValidationSeverity::Error => errors += 1,
                    ValidationSeverity::Warning => warnings += 1,
                    ValidationSeverity::Info => info_count += 1,
                }
            }
        }
    }
    let last_updated = entries
        .first()
        .map(|entry| entry.submitted_at.format("%Y-%m-%d %H:%M:%S").to_string());
    Some(IngestionStatsView {
        total_runs: entries.len(),
        errors,
        warnings,
        info: info_count,
        last_updated,
    })
}
