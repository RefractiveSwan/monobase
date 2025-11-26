use maud::{Markup, html};
use refractive_swan_contracts::PipelineMetrics;

use crate::views::components::{card::*, typography::*};
use crate::views::models::{AlertMessage, PageContext};
use crate::views::pages::workbench::{
    render_history_container, render_validation_summary_container,
};

use super::results::render_results_panel;

/// HTMX fragment returned to `/map/paste` and `/map/upload` handlers.
pub fn render_results_fragment(ctx: &PageContext) -> String {
    html! {
        (render_results(ctx))
        (render_validation_summary_container(ctx, true))
        (render_history_container(ctx, true))
    }
    .into_string()
}

fn render_results(ctx: &PageContext) -> Markup {
    html! {
        @if let Some(alert) = &ctx.alert {
            (render_alert(alert))
        }
        @if let Some(results) = &ctx.results {
            (render_results_panel(results))
        } @else {
            div class="bg-white rounded-md border border-dashed border-gray-300 p-12 text-center" {
                h3 class="mt-2 text-sm font-medium text-gray-900" { "No results generated" }
                p class="mt-1 text-sm text-gray-500" { "Submit a Bundle to see mapping analysis." }
            }
        }
    }
}

pub(crate) fn render_metrics_dashboard(metrics: Option<&PipelineMetrics>) -> Markup {
    html! {
        (card(html! {
            (card_header("Pipeline Metrics", Some(html! {
                span class="text-xs font-mono text-gray-500" {
                    @if let Some(mode) = metrics.and_then(|m| m.compliance_mode.as_deref()) {
                        (format!("Compliance Mode: {}", mode))
                    } @else {
                        "Live Snapshot"
                    }
                }
            })))

            @if let Some(metrics) = metrics {
                (card_body(html! {
                    div class="space-y-6" {
                        div class="grid gap-4 md:grid-cols-3" {
                            (metric_card("Bundles Processed", &metrics.bundle_count.to_string(), None))
                            (metric_card("Flattened Rows", &metrics.flats_count.to_string(), None))
                            (metric_card("Mapping Attempts", &metrics.mapping_count.to_string(), None))
                        }

                        div class="grid gap-4 md:grid-cols-3" {
                            (state_metric_card("AutoMapped", metrics.auto_mapped, "bg-emerald-50 text-emerald-800 border-emerald-100", "High confidence matches"))
                            (state_metric_card("Needs Review", metrics.needs_review, "bg-amber-50 text-amber-800 border-amber-100", "Requires validation"))
                            (state_metric_card("No Match", metrics.no_match, "bg-rose-50 text-rose-800 border-rose-100", "Unresolved concepts"))
                        }

                        div class="grid gap-4 md:grid-cols-4 pt-4 border-t border-gray-100" {
                            (secondary_metric("License Blocked", metrics.license_blocked, "text-rose-700"))
                            (secondary_metric("Vector Queries", metrics.vector_queries, "text-gray-700"))
                            (secondary_metric("Cohort Queries", metrics.cohort_queries, "text-gray-700"))
                            (secondary_metric_avg("Avg Cohort Size", metrics.avg_cohort_size.map(|v| v as f64), "text-gray-700"))
                        }
                    }
                }))
            } @else {
                div class="p-6 text-center text-sm text-gray-500 italic" {
                    "Metrics will populate after the first mapping run."
                }
            }
        }))
    }
}

pub(crate) fn render_eval_panel(ctx: &PageContext) -> Markup {
    html! {
        (card(html! {
            (card_header("Evaluation Report", Some(html! {
                span class="text-xs text-gray-500" { "Gold Standard Comparison" }
            })))
            (card_body(html! {
                div class="space-y-4" {
                    div class="flex flex-wrap items-center gap-3 text-sm" {
                        (label_text("Dataset"))
                        select id="eval-dataset" name="dataset" class="rounded-md border-gray-300 px-3 py-1.5 text-sm focus:border-navy-900 focus:ring-1 focus:ring-navy-900"
                            hx-get="/eval/report"
                            hx-target="#eval-report-fragment"
                            hx-swap="innerHTML"
                            hx-trigger="change" {
                            @if !ctx.datasets.is_empty() {
                                @for dataset in &ctx.datasets {
                                    option value=(dataset.name) selected[(ctx.selected_eval_dataset == dataset.name)] { (dataset.name.clone()) }
                                }
                            } @else {
                                option value=(ctx.selected_eval_dataset) { (ctx.selected_eval_dataset.clone()) }
                            }
                        }
                    }

                    div id="eval-report-fragment" class="mt-4 rounded-md border border-gray-200 bg-gray-50 p-4" {
                        @if let Some(html) = &ctx.eval_report_html {
                            (maud::PreEscaped(html))
                        } @else if let Some(err) = &ctx.eval_panel_error {
                            p class="text-sm text-rose-700" { (err) }
                        } @else {
                            p class="text-sm text-gray-500 italic" { "Select a dataset to view evaluation metrics." }
                        }
                    }
                }
            }))
        }))
    }
}

fn render_alert(alert: &AlertMessage) -> Markup {
    crate::views::components::badge::alert(alert)
}
