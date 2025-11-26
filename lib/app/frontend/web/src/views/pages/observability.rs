use maud::{Markup, PreEscaped, html};

use crate::views::components::{card::*, table::*};
use crate::views::layout::{Breadcrumb, PageCallout, PageShellProps, page_shell};
use crate::views::models::{LogEntryKindView, LogEntryView, PageContext};

pub fn render_observability_page(ctx: &PageContext, logs: &[LogEntryView]) -> String {
    page_shell(PageShellProps {
        title: "Observability",
        chrome: &ctx.chrome,
        content: html! {
            (render_observability_cards(ctx))
            (render_log_panel(logs, false))
        },
        breadcrumbs: vec![
            Breadcrumb::home(),
            Breadcrumb::new("Observability", Some("/observability")),
        ],
        callouts: vec![PageCallout::info(
            "Live signals",
            "Vector usage, compliance state, dataset health, and NoMatch/API error logs update in real time.",
        )],
    })
    .into_string()
}

pub fn render_log_fragment(logs: &[LogEntryView]) -> String {
    html! {
        div id="log-panel" hx-get="/logs/latest" hx-trigger="load, every 8s" hx-target="#log-panel" hx-swap="outerHTML" {
            (render_log_entries(logs))
        }
    }
    .into_string()
}

fn render_observability_cards(ctx: &PageContext) -> Markup {
    html! {
        div class="grid gap-6 lg:grid-cols-3" {
            (render_vector_card(ctx))
            (render_compliance_card(ctx))
            (render_dataset_card(ctx))
        }
        div class="mt-6 grid gap-6 lg:grid-cols-3" {
            (render_terminology_card(ctx))
            (render_ingestion_card(ctx))
            div {}
        }
    }
}

fn render_vector_card(ctx: &PageContext) -> Markup {
    card(html! {
        (card_header("Vector usage", Some(html! {
            span class="text-xs text-gray-500" { "from /metrics/summary" }
        })))
        (card_body(html! {
            @if let Some(metrics) = &ctx.metrics {
                    div class="grid gap-3 text-sm" {
                        (metric_row("Queries", metrics.vector_queries))
                        (metric_row("Hits", metrics.vector_hits))
                        (metric_row("Fallbacks", metrics.vector_fallbacks))
                        div class="rounded-md border border-gray-100 bg-gray-50 p-3 text-xs text-gray-600 space-y-1" {
                            p { (format!("Latency p95: {} ms", metrics.vector_latency_ms_p95.map(|v| v.to_string()).unwrap_or_else(|| "n/a".into()))) }
                        }
                    }
            } @else {
                p class="text-sm text-gray-500 italic" { "Metrics unavailable. Submit a bundle to populate vector usage stats." }
            }
        }))
    })
}

fn render_compliance_card(ctx: &PageContext) -> Markup {
    card(html! {
        (card_header("Compliance policy", None))
        (card_body(html! {
            @if let Some(policy) = &ctx.compliance_policy {
                div class="space-y-3 text-sm text-gray-600" {
                    p class="font-semibold text-navy-900" { (format!("Mode: {}", policy.mode)) }
                    ul class="space-y-2 text-xs" {
                        @for action in &policy.actions {
                            li class="flex items-center justify-between rounded border border-gray-100 bg-gray-50 px-2 py-1" {
                                span class="font-semibold text-navy-900" { (&action.action) }
                                span class=(if action.allowed { "text-emerald-700" } else { "text-rose-700" }) {
                                    @if action.allowed { "allowed" } @else { "blocked" }
                                }
                            }
                        p class="text-[11px] text-gray-500 mt-1" {
                            ({
                                let tier_text = if action.tiers.is_empty() {
                                    "default".to_string()
                                } else {
                                    action.tiers.join(", ")
                                };
                                format!("Tiers: {}", tier_text)
                            })
                        }
                    }
                }
                }
            } @else {
                p class="text-sm text-gray-500 italic" { "Compliance metrics not yet available." }
            }
        }))
    })
}

fn render_dataset_card(ctx: &PageContext) -> Markup {
    card(html! {
        (card_header("Dataset store health", None))
        (card_body(html! {
            @if ctx.datasets.is_empty() {
                p class="text-sm text-gray-500 italic" { "Dataset store did not return manifests. Check refractive_swan_eval config." }
            } @else {
                p class="text-xs text-gray-500 mb-3" { (format!("{} dataset(s) available", ctx.datasets.len())) }
                (table_container(html! {
                    (table_header(&["Dataset", "Cases"]))
                    tbody class="bg-white divide-y divide-gray-200 text-xs" {
                        @for dataset in &ctx.datasets {
                            (table_row(html! {
                                (table_cell(html! { (dataset.name.clone()) }))
                                (table_cell_mono(html! { (dataset.n_cases) }))
                            }))
                        }
                    }
                }))
            }
        }))
    })
}

fn render_terminology_card(ctx: &PageContext) -> Markup {
    card(html! {
        (card_header("Terminology insights", None))
        (card_body(html! {
            @if let Some(insights) = &ctx.terminology_insights {
                div class="flex items-center justify-between text-sm text-gray-600" {
                    span { (format!("Systems: {}", insights.total_systems)) }
                    span { (format!("Licensed/Open: {}/{}", insights.licensed, insights.open)) }
                }
                ul class="mt-3 space-y-2 text-xs text-gray-600" {
                    @for system in insights.code_systems.iter().take(3) {
                        li class="flex items-center justify-between rounded border border-gray-100 bg-gray-50 px-2 py-1" {
                            span class="font-semibold text-navy-900" { (&system.name) }
                            span { (system.license_tier.clone()) }
                        }
                    }
                }
            } @else {
                p class="text-sm text-gray-500 italic" { "Terminology registry metadata not loaded." }
            }
        }))
    })
}

fn render_ingestion_card(ctx: &PageContext) -> Markup {
    card(html! {
        (card_header("Ingestion validation", None))
        (card_body(html! {
            @if let Some(stats) = &ctx.ingestion_stats {
                div class="grid grid-cols-3 gap-2 text-center" {
                    (metric_row("Errors", stats.errors))
                    (metric_row("Warnings", stats.warnings))
                    (metric_row("Info", stats.info))
                }
                p class="mt-3 text-xs text-gray-500" {
                    (format!("{} run(s) tracked · Updated {}", stats.total_runs, stats.last_updated.clone().unwrap_or_else(|| "n/a".into())))
                }
            } @else {
                p class="text-sm text-gray-500 italic" { "Submit bundles to populate validation totals." }
            }
        }))
    })
}

fn metric_row(label: &str, value: usize) -> Markup {
    html! {
        div class="flex items-center justify-between rounded-md border border-gray-100 bg-gray-50 px-3 py-2" {
            span class="text-xs text-gray-500" { (label) }
            span class="text-sm font-semibold text-navy-900" { (value) }
        }
    }
}

fn render_log_panel(logs: &[LogEntryView], oob: bool) -> Markup {
    let panel = html! {
        (card(html! {
            (card_header("Live logs", Some(html! {
                span class="text-xs text-gray-500" { "NoMatch + error events" }
            })))
            (PreEscaped(render_log_fragment(logs)))
        }))
    };
    if oob {
        html! {
            div hx-swap-oob="outerHTML" id="log-panel-wrapper" {
                (panel)
            }
        }
    } else {
        panel
    }
}

fn render_log_entries(logs: &[LogEntryView]) -> Markup {
    if logs.is_empty() {
        return html! {
            div class="p-4 text-sm text-gray-500 italic" { "No events recorded yet." }
        };
    }
    html! {
        ul class="divide-y divide-gray-100" {
            @for entry in logs {
                li class="px-4 py-3 text-sm flex flex-col gap-1" {
                    div class="flex items-center justify-between text-xs text-gray-500" {
                        span { (&entry.timestamp) }
                        (log_badge(entry.kind))
                    }
                    p class="font-mono text-[11px] text-navy-900" { (&entry.message) }
                }
            }
        }
    }
}

fn log_badge(kind: LogEntryKindView) -> Markup {
    match kind {
        LogEntryKindView::Info => {
            html! { span class="rounded bg-sky-50 px-2 py-0.5 text-[10px] font-semibold text-sky-800 border border-sky-100" { "Info" } }
        }
        LogEntryKindView::NoMatch => {
            html! { span class="rounded bg-amber-50 px-2 py-0.5 text-[10px] font-semibold text-amber-800 border border-amber-100" { "NoMatch" } }
        }
        LogEntryKindView::Error => {
            html! { span class="rounded bg-rose-50 px-2 py-0.5 text-[10px] font-semibold text-rose-800 border border-rose-100" { "Error" } }
        }
    }
}
