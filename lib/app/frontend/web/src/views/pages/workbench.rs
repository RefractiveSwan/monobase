use maud::{Markup, html};
use refractive_swan_contracts::ValidationSeverity;

use crate::client::MetricsSnapshot;
use crate::templates;
use crate::vector::VectorMode;
use crate::views::components::{badge::*, button::*, card::*, input::*, typography::*};
use crate::views::layout::{Breadcrumb, PageCallout, PageShellProps, page_shell};
use crate::views::models::{
    AlertKind, AlertMessage, IngestionStatsView, MappingHistoryStatusView, PageContext,
    ValidationSummaryView, VectorConfigView,
};
use crate::views::pages::eval::render_no_match_explorer;
use crate::views::partials::fragments::{render_eval_panel, render_metrics_dashboard};
use crate::views::partials::results::render_results_panel;

pub fn render_workbench_page(ctx: &PageContext) -> String {
    page_shell(PageShellProps {
        title: "Mapping Workbench",
        chrome: &ctx.chrome,
        content: html! {
            (render_workbench_hero(ctx))
            (render_input_section(ctx))
            (render_history_container(ctx, false))
            (render_results_section(ctx))
            @if let Some(metrics) = &ctx.metrics {
                (render_metrics_dashboard(Some(metrics)))
            }
            (render_eval_panel(ctx))
        },
        breadcrumbs: vec![
            Breadcrumb::home(),
            Breadcrumb::new("Workbench", Some("/map")),
        ],
        callouts: vec![PageCallout::info(
            "CLI parity",
            "Paste/upload flows call the same `/api/map-bundles` endpoint that powers refractive_swan_cli."
        )],
    })
    .into_string()
}

fn render_workbench_hero(ctx: &PageContext) -> Markup {
    card(card_body(html! {
        div class="flex flex-col md:flex-row md:items-start md:justify-between gap-4" {
            (render_hero_content())
            (render_hero_status_badges(ctx))
        }
        (render_hero_error(ctx))
        (render_compliance_warning(ctx))
    }))
}

fn render_hero_content() -> Markup {
    html! {
        div class="space-y-3 max-w-3xl" {
            (section_heading("Clinical Mapping Pipeline"))
            p class="text-slate-600 leading-relaxed" {
                "Ingest FHIR Bundles to flatten ServiceRequests into "
                (code_badge("stg_servicerequest_flat"))
                " and "
                (code_badge("stg_sr_code_exploded"))
                ". The engine emits "
                (code_badge("MappingResult"))
                " rows cross-referenced against NCIt concepts."
            }
        }
    }
}

fn render_hero_status_badges(ctx: &PageContext) -> Markup {
    html! {
        div class="flex flex-col items-end gap-2" {
            @if let Some(health) = &ctx.health {
                (status_badge(&health.status))
            } @else {
                (status_badge("Unknown"))
            }
            @if let Some(metrics) = &ctx.metrics {
                (render_quick_metrics(metrics))
            }
        }
    }
}

fn render_quick_metrics(metrics: &MetricsSnapshot) -> Markup {
    html! {
        div class="text-xs text-slate-500 font-mono text-right" {
            div { (format!("Bundles: {}", metrics.metrics.bundle_count)) }
            div { (format!("Mapped: {}", metrics.metrics.auto_mapped)) }
        }
    }
}

fn render_hero_error(ctx: &PageContext) -> Markup {
    html! {
        @if let Some(error) = &ctx.health_error {
            div class="mt-4" {
                (alert(&AlertMessage { kind: AlertKind::Error, text: format!("System Warning: {}", error) }))
            }
        }
    }
}

fn render_compliance_warning(ctx: &PageContext) -> Markup {
    if let Some(metrics) = &ctx.metrics {
        if let Some(mode) = metrics.metrics.compliance_mode.as_deref() {
            let (message, class) = if metrics.metrics.license_blocked > 0 {
                (
                    format!(
                        "Compliance mode {} blocked {} mapping(s). Export limited to licensed tiers.",
                        mode, metrics.metrics.license_blocked
                    ),
                    "bg-rose-50 text-rose-900 border-rose-100",
                )
            } else {
                (
                    format!(
                        "Compliance mode {} active – exports filtered automatically.",
                        mode
                    ),
                    "bg-slate-50 text-slate-700 border-slate-100",
                )
            };
            return html! {
                div class=(format!("mt-4 rounded-md border px-3 py-2 text-xs font-medium {}", class)) {
                    (message)
                }
            };
        }
    }
    html! {}
}

fn render_input_section(ctx: &PageContext) -> Markup {
    html! {
        div class="grid gap-6 lg:grid-cols-3" {
            div class="lg:col-span-2 space-y-6" {
                (render_paste_input_card())
                (render_upload_input_card())
            }
            div class="space-y-6" {
                (render_validation_summary_container(ctx, false))
                (render_vector_mode_panel(&ctx.vector_config))
                @if let Some(stats) = &ctx.ingestion_stats {
                    (render_ingestion_stats_panel(stats))
                }
            }
        }
    }
}

fn render_paste_input_card() -> Markup {
    card(html! {
        (card_header("Input: Paste JSON", None))
        (card_body(html! {
            form hx-post="/map/paste" hx-target="#results" hx-swap="innerHTML" method="post" class="h-full flex flex-col space-y-4" {
                (render_template_picker())
                div id="bundle-text-container" {
                    (render_bundle_textarea_fragment(templates::load_template("blank")))
                }
                div class="flex justify-end" {
                    (primary_button(ButtonProps {
                        text: "Submit & Map",
                        type_: "submit",
                        icon: Some(html! {
                            svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 12L3.269 3.126A59.768 59.768 0 0121.485 12 59.77 59.77 0 013.27 20.876L5.999 12zm0 0h7.5" {}
                            }
                        }),
                        ..Default::default()
                    }))
                }
            }
        }))
    })
}

fn render_template_picker() -> Markup {
    html! {
        div class="flex items-center justify-between gap-3" {
            div class="flex flex-col gap-1" {
                (label_text("Sample data"))
                span class="text-[11px] text-gray-500" { "Prefill the textarea with vetted bundles from evaluation fixtures." }
            }
            select
                name="name"
                class="rounded-md border-gray-300 px-3 py-1.5 text-sm focus:border-navy-900 focus:ring-1 focus:ring-navy-900"
                hx-get="/map/template"
                hx-target="#bundle-text-container"
                hx-swap="innerHTML"
                hx-trigger="change" {
                @for template in crate::templates::TEMPLATES {
                    option value=(template.id) selected[(template.id == "blank")] {
                        (template.label)
                    }
                }
            }
        }
    }
}

pub(crate) fn render_bundle_textarea_fragment(value: &str) -> Markup {
    html! {
        label class="block" for="bundle_text" {
            (label_text("JSON Payload"))
            (helper_text("(Paste FHIR Bundle)"))
        }
        textarea
            id="bundle_text"
            name="bundle_text"
            spellcheck="false"
            rows="12"
            class="w-full rounded-md border-2 border-gray-300 p-3 font-mono text-xs focus:border-navy-900 focus:ring-2 focus:ring-navy-900 focus:ring-offset-2 transition-all" {
            (value)
        }
    }
}

fn render_upload_input_card() -> Markup {
    card(html! {
        (card_header("Input: Upload File", None))
        (card_body(html! {
            form hx-post="/map/upload" hx-target="#results" hx-swap="innerHTML" method="post" enctype="multipart/form-data" class="space-y-4" {
                label class="block" {
                    (label_text("JSON File"))
                    (helper_text("(Bundle or NDJSON)"))
                }
                (file_upload("bundle_file", "application/json,.json,.ndjson"))
                div class="flex justify-end" {
                    (primary_button(ButtonProps {
                        text: "Upload & Map",
                        type_: "submit",
                        icon: Some(html! {
                            svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" {}
                            }
                        }),
                        ..Default::default()
                    }))
                }
            }
        }))
    })
}

fn render_vector_mode_panel(config: &VectorConfigView) -> Markup {
    html! {
        div id="vector-mode-panel" {
            (card(html! {
                (card_header("Mapping mode", Some(html! {
                    span class="text-xs text-gray-500" { "Toggle lexical-only vs vector-backed runs" }
                })))
                (card_body(html! {
                    div class="space-y-2 text-xs text-gray-600" {
                        p class="font-semibold text-navy-900" {
                            (format!("Active mode: {}", config.mode.as_str()))
                        }
                        p { (format!("Env default: {}", if config.env_enabled { "enabled" } else { "disabled" })) }
                        @if let Some(backend) = &config.backend {
                            p { (format!("Backend: {}", backend)) }
                        }
                        @if let Some(namespace) = &config.namespace {
                            p { (format!("Namespace: {}", namespace)) }
                        }
                    }
                    form hx-post="/settings/vector-mode" hx-target="#vector-mode-panel" hx-swap="outerHTML" class="mt-3 flex gap-3" {
                        button
                            type="submit"
                            name="mode"
                            value="enabled"
                            class="flex-1 rounded-md border border-navy-200 px-3 py-2 text-xs font-semibold text-navy-900 shadow-sm hover:bg-navy-50 disabled:opacity-50"
                            disabled[config.mode == VectorMode::Enabled] {
                            "Vector enabled"
                        }
                        button
                            type="submit"
                            name="mode"
                            value="disabled"
                            class="flex-1 rounded-md border border-navy-200 px-3 py-2 text-xs font-semibold text-navy-900 shadow-sm hover:bg-navy-50 disabled:opacity-50"
                            disabled[config.mode == VectorMode::Disabled] {
                            "Lexical only"
                        }
                    }
                }))
            }))
        }
    }
}

fn render_ingestion_stats_panel(stats: &IngestionStatsView) -> Markup {
    card(html! {
        (card_header("Validation stats", Some(html! {
            span class="text-xs text-gray-500" { "Aggregated from upload history" }
        })))
        (card_body(html! {
            div class="grid grid-cols-3 gap-2" {
                (stat_chip("Errors", stats.errors, "text-rose-700"))
                (stat_chip("Warnings", stats.warnings, "text-amber-700"))
                (stat_chip("Info", stats.info, "text-slate-600"))
            }
            p class="mt-3 text-xs text-gray-500" {
                (format!("{} run(s) tracked · Updated {}", stats.total_runs, stats.last_updated.clone().unwrap_or_else(|| "n/a".into())))
            }
        }))
    })
}

pub fn render_vector_mode_fragment(config: &VectorConfigView) -> String {
    render_vector_mode_panel(config).into_string()
}

fn stat_chip(label: &str, value: usize, class: &str) -> Markup {
    html! {
        div class=(format!("rounded-md border border-gray-100 bg-gray-50 px-3 py-2 text-center {}", class)) {
            p class="text-[10px] uppercase tracking-wide text-gray-500" { (label) }
            p class="text-lg font-semibold text-navy-900" { (value) }
        }
    }
}

pub(crate) fn render_validation_summary_container(ctx: &PageContext, oob: bool) -> Markup {
    if oob {
        html! {
            div id="validation-summary" hx-swap-oob="innerHTML" {
                (render_validation_summary_card(ctx))
            }
        }
    } else {
        html! {
            div id="validation-summary" {
                (render_validation_summary_card(ctx))
            }
        }
    }
}

fn render_validation_summary_card(ctx: &PageContext) -> Markup {
    card(html! {
        (card_header("Validation summary", None))
        (card_body(html! {
            @if let Some(summary) = &ctx.validation_summary {
                div class="flex items-center justify-between mb-4" {
                    @if summary.has_errors {
                        span class="text-sm font-semibold text-rose-700" { "Errors detected" }
                    } @else {
                        span class="text-sm font-semibold text-emerald-700" { "All checks passed" }
                    }
                    span class="text-xs text-gray-500 font-mono" { (format!("{} issues", summary.total)) }
                }
                div class="grid gap-3 md:grid-cols-3 text-center text-xs font-semibold" {
                    div class="rounded-md border border-rose-100 bg-rose-50 py-3" {
                        p class="text-rose-700" { "Errors" }
                        p class="text-rose-900 text-xl" { (summary.errors) }
                    }
                    div class="rounded-md border border-amber-100 bg-amber-50 py-3" {
                        p class="text-amber-700" { "Warnings" }
                        p class="text-amber-900 text-xl" { (summary.warnings) }
                    }
                    div class="rounded-md border border-slate-100 bg-slate-50 py-3" {
                        p class="text-slate-600" { "Info" }
                        p class="text-slate-900 text-xl" { (summary.info) }
                    }
                }
                (render_validation_issue_list(summary))
            } @else {
                div class="text-sm text-gray-500 space-y-2" {
                    p { "No validation report yet. Submit a bundle to view strict/lenient findings." }
                    p class="text-xs" { "Strict ingestion mode surfaces errors and warnings aligned with docs/system-design/clinical/fhir requirements." }
                }
            }
        }))
    })
}

fn render_validation_issue_list(summary: &ValidationSummaryView) -> Markup {
    if summary.issues.is_empty() {
        return html! {
            p class="mt-4 text-xs text-gray-500 italic" { "No detailed validation issues reported." }
        };
    }
    let preview = summary.issues.iter().take(4);
    html! {
        div class="mt-4 space-y-2" {
            @for issue in preview {
                div class="rounded-md border border-gray-100 bg-gray-50 px-3 py-2 text-xs" {
                    div class="flex items-center justify-between gap-3" {
                        span class="font-semibold text-gray-700" { (&issue.bundle_label) }
                        span class="font-mono text-[10px] text-gray-500" { (&issue.requirement) }
                    }
                    div class="mt-1 flex items-center gap-2" {
                        (severity_badge(issue.severity))
                        span class="text-gray-800" { (&issue.message) }
                    }
                }
            }
            @if summary.issues.len() > 4 {
                p class="text-[10px] text-gray-500 uppercase tracking-wide" { (format!("+{} more issue(s)", summary.issues.len() - 4)) }
            }
        }
    }
}

fn severity_badge(severity: ValidationSeverity) -> Markup {
    let (label, class) = match severity {
        ValidationSeverity::Error => ("Error", "bg-rose-100 text-rose-800"),
        ValidationSeverity::Warning => ("Warning", "bg-amber-100 text-amber-800"),
        ValidationSeverity::Info => ("Info", "bg-slate-100 text-slate-700"),
    };
    html! {
        span class=(format!("px-2 py-0.5 rounded-md text-[10px] font-semibold {}", class)) { (label) }
    }
}

pub(crate) fn render_history_container(ctx: &PageContext, oob: bool) -> Markup {
    if oob {
        html! {
            div id="mapping-history" hx-swap-oob="innerHTML" {
                (render_history_panel(ctx))
            }
        }
    } else {
        html! {
            div id="mapping-history" {
                (render_history_panel(ctx))
            }
        }
    }
}

fn render_history_panel(ctx: &PageContext) -> Markup {
    card(html! {
        (card_header("Upload history", Some(html! {
            span class="text-xs text-gray-500 font-medium" { "Most recent 5 runs" }
        })))
        (card_body(html! {
            @if ctx.mapping_history.is_empty() {
                p class="text-sm text-gray-500" { "No mapping runs yet. Submit a bundle to populate history." }
            } @else {
                ul class="space-y-3 text-sm" {
                    @for entry in &ctx.mapping_history {
                        li class="rounded-md border border-gray-100 p-3 bg-white shadow-sm" {
                            div class="flex items-center justify-between" {
                                (history_status_badge(entry.status))
                                span class="text-xs text-gray-400 font-mono" { (&entry.submitted_at) }
                            }
                            div class="mt-2 text-xs text-gray-600 flex flex-wrap gap-3" {
                                @if let Some(mapped) = entry.mapped {
                                    span { (format!("Mapped: {}", mapped)) }
                                }
                                span { (format!("Duration: {}", format_duration(entry.duration_ms))) }
                                @if entry.validation_errors > 0 {
                                    span class="text-rose-600 font-semibold" { (format!("Validation errors: {}", entry.validation_errors)) }
                                }
                            }
                            div class="mt-3 flex items-center justify-between gap-3 text-xs" {
                                @if let Some(error) = &entry.error {
                                    span class="text-rose-700 font-medium" { (error) }
                                } @else {
                                    span class="text-gray-500" { "Cached response available." }
                                }
                                @if entry.error.is_none() {
                                    button
                                        class="inline-flex items-center gap-1 rounded-md border border-gray-200 px-3 py-1 text-xs font-medium text-navy-900 hover:bg-gray-50"
                                        hx-get=(format!("/map/history/{}", entry.id))
                                        hx-target="#results"
                                        hx-swap="innerHTML"
                                    {
                                        "View results"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }))
    })
}

fn history_status_badge(status: MappingHistoryStatusView) -> Markup {
    match status {
        MappingHistoryStatusView::Success => {
            html! { span class="text-xs font-semibold text-emerald-700" { "Success" } }
        }
        MappingHistoryStatusView::Error => {
            html! { span class="text-xs font-semibold text-rose-700" { "Error" } }
        }
    }
}

fn format_duration(ms: u64) -> String {
    if ms < 1000 {
        format!("{ms} ms")
    } else {
        format!("{:.2} s", ms as f64 / 1000.0)
    }
}

fn render_results_section(ctx: &PageContext) -> Markup {
    html! {
        div id="results" {
            @if let Some(results) = &ctx.results {
                (render_results_panel(results))
                div class="mt-8" {
                    (render_no_match_explorer(Some(results)))
                }
                div class="mt-4 flex justify-end" {
                    a
                        href="/map/download/latest"
                        class="inline-flex items-center gap-1 rounded-md border border-gray-200 bg-white px-4 py-2 text-sm font-semibold text-navy-900 hover:bg-gray-50"
                    {
                        svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v16c0 .552.448 1 1 1h14a1 1 0 001-1V4m-5 6l-3 3-3-3m3 3V3" {}
                        }
                        "Download NDJSON"
                    }
                }
            } @else {
                (render_no_match_explorer(None))
            }
        }
    }
}
