use maud::{Markup, html};

use crate::client::CohortFilters;
use crate::views::components::{badge::*, card::*, input::*, layout::*, table::*, typography::*};
use crate::views::layout::{Breadcrumb, PageCallout, PageShellProps, page_shell};
use crate::views::models::{
    self, AlertKind, AlertMessage, AnalyticsSummaryView, CohortView, PageContext,
};

pub fn render_analytics_page(ctx: &PageContext) -> String {
    page_shell(PageShellProps {
        title: "Analytics & Observability",
        chrome: &ctx.chrome,
        content: html! {
            (section_heading("Analytics Dashboard"))
            (render_analytics_panels(ctx))
        },
        breadcrumbs: vec![
            Breadcrumb::home(),
            Breadcrumb::new("Analytics", Some("/analytics")),
        ],
        callouts: vec![PageCallout::info(
            "Observability mode",
            "Analytics and cohort panels read from `/analytics/*` endpoints and the observability metrics cache."
        )],
    })
    .into_string()
}

pub(crate) fn render_analytics_panels(ctx: &PageContext) -> Markup {
    html! {
        (grid_section(html! {
            (card(html! {
                (card_header("Analytics Summary", None))
                (card_body(html! {
                    div class="space-y-4" {
                        (render_summary_filters())
                        div id="analytics-summary-fragment" class="space-y-6" {
                            @if let Some(error) = &ctx.analytics_error {
                                (alert(&AlertMessage { kind: AlertKind::Error, text: error.clone() }))
                            } @else if let Some(summary) = &ctx.analytics_summary {
                                (render_summary_content(summary))
                            } @else {
                                p class="text-sm text-gray-500 italic" { "No analytics data available." }
                            }
                        }
                    }
                }))
            }))

            (card(html! {
                (card_header("Cohort Explorer", None))
                (card_body(html! {
                    div class="space-y-4" {
                        form method="get" action="/analytics" class="grid gap-4 md:grid-cols-2 text-sm" {
                            label class="flex flex-col gap-1" {
                                (label_text("NCIt ID"))
                                (text_input("ncit_id", ctx.cohort_filters.ncit_id.as_deref(), "CXXXX"))
                            }
                            label class="flex flex-col gap-1" {
                                (label_text("Status"))
                                (text_input("status", ctx.cohort_filters.status.as_deref(), "active"))
                            }
                            label class="flex flex-col gap-1" {
                                (label_text("Date From"))
                                (text_input("date_from", ctx.cohort_filters.date_from.as_deref(), "YYYY-MM-DD"))
                            }
                            label class="flex flex-col gap-1" {
                                (label_text("Date To"))
                                (text_input("date_to", ctx.cohort_filters.date_to.as_deref(), "YYYY-MM-DD"))
                            }
                            div class="md:col-span-2 flex justify-end" {
                                button type="submit" class="inline-flex items-center rounded-md bg-white border border-gray-300 px-4 py-1.5 text-sm font-medium text-navy-900 hover:bg-gray-50" {
                                    "Apply Filters"
                                }
                            }
                        }

                        @if let Some(error) = &ctx.cohort_error {
                            (alert(&AlertMessage { kind: AlertKind::Error, text: error.clone() }))
                        }
                        @if let Some(cohort) = &ctx.cohort {
                            div class="mt-4" {
                                p class="text-xs text-gray-500 mb-2" { (format!("Found {} matching records", cohort.total)) }
                                (render_cohort_export_link(&ctx.cohort_filters))
                                (render_cohort_table(cohort))
                            }
                        } @else {
                            p class="text-sm text-gray-500 italic" { "Run a cohort query to see results." }
                        }
                    }
                }))
            }))
        }))
    }
}

fn render_cohort_export_link(filters: &CohortFilters) -> Markup {
    let query = filters.to_query_string();
    let href = if query.is_empty() {
        "/analytics/cohort/export".to_string()
    } else {
        format!("/analytics/cohort/export?{}", query)
    };
    html! {
        div class="mb-3 flex justify-end" {
            a href=(href) class="inline-flex items-center gap-1 rounded border border-gray-200 bg-white px-3 py-1 text-xs font-semibold text-navy-900 hover:bg-gray-50" {
                svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4h16v16H4z M8 4v16 M16 4v16 M4 10h16 M4 14h16" {}
                }
                "Download CSV"
            }
        }
    }
}

fn render_summary_filters() -> Markup {
    html! {
        form
            class="flex flex-wrap gap-3 text-xs"
            hx-get="/analytics/summary/fragment"
            hx-target="#analytics-summary-fragment"
            hx-trigger="change"
        {
            label class="flex flex-col gap-1" {
                (label_text("State filter"))
                select name="state_filter" class="rounded-md border-gray-300 px-3 py-1.5 text-xs" {
                    option value="" { "All states" }
                    option value="auto_mapped" { "AutoMapped" }
                    option value="needs_review" { "Needs review" }
                    option value="no_match" { "NoMatch" }
                }
            }
            label class="flex flex-col gap-1" {
                (label_text("Time range"))
                select name="range_days" class="rounded-md border-gray-300 px-3 py-1.5 text-xs" {
                    option value="" { "All time" }
                    option value="7" { "Last 7 days" }
                    option value="14" { "Last 14 days" }
                    option value="30" { "Last 30 days" }
                }
            }
        }
    }
}

pub fn render_summary_fragment(summary: &AnalyticsSummaryView) -> String {
    render_summary_content(summary).into_string()
}

fn render_summary_content(summary: &AnalyticsSummaryView) -> Markup {
    html! {
        (render_top_concepts(&summary.top_concepts))
        div class="border-t border-gray-100 pt-4" {
            (render_state_distribution(&summary.state_counts))
        }
        div class="border-t border-gray-100 pt-4" {
            (render_time_chart(&summary.time_buckets))
        }
    }
}

fn render_top_concepts(concepts: &[models::AnalyticsConceptTile]) -> Markup {
    html! {
        div class="space-y-3" {
            (subsection_heading("Top Concepts"))
            div class="space-y-2" {
                @for concept in concepts {
                    div class="flex items-center justify-between p-2 rounded bg-gray-50 border border-gray-100" {
                        div {
                            p class="text-sm font-medium text-navy-900" { (concept.preferred_name.clone()) }
                            p class="text-xs font-mono text-gray-500" { (concept.ncit_id.clone()) }
                        }
                        span class="text-xs font-bold bg-white px-2 py-1 rounded border border-gray-200" { (concept.total) }
                    }
                }
                @if concepts.is_empty() {
                    p class="text-sm text-gray-500" { "No concepts found." }
                }
            }
        }
    }
}

fn render_state_distribution(states: &[models::CountStat]) -> Markup {
    html! {
        div class="space-y-3" {
            (subsection_heading("State Distribution"))
            div class="space-y-2" {
                @for stat in states {
                    div class="flex items-center justify-between p-2 rounded bg-gray-50 text-sm" {
                        span class="text-gray-600" { (stat.label.clone()) }
                        span class="font-mono font-medium" { (stat.count) }
                    }
                }
                @if states.is_empty() {
                    p class="text-sm text-gray-500" { "No mappings observed yet." }
                }
            }
        }
    }
}

fn render_cohort_table(cohort: &CohortView) -> Markup {
    html! {
        (table_container(html! {
            (table_header(&["SR ID", "Patient", "Encounter", "NCIt", "Status", "Intent", "Ordered At", "State"]))
            tbody class="bg-white divide-y divide-gray-200" {
                @for row in &cohort.rows {
                    (table_row(html! {
                        (table_cell_mono(html! { (row.sr_id.clone()) }))
                        (table_cell(html! { (row.patient_id.clone()) }))
                        (table_cell(html! { (row.encounter_id.clone()) }))
                        (table_cell_mono(html! { (row.ncit_id.clone()) }))
                        (table_cell(html! { (row.status.clone()) }))
                        (table_cell(html! { (row.intent.clone()) }))
                        (table_cell_mono(html! { (row.ordered_at.clone()) }))
                        (table_cell(state_chip(&row.mapping_state)))
                    }))
                }
            }
        }))
    }
}

fn render_time_chart(buckets: &[models::AnalyticsTimeBucket]) -> Markup {
    if buckets.is_empty() {
        return html! { p class="text-sm text-gray-500" { "No ordered_at timestamps available." } };
    }
    let max_total = buckets
        .iter()
        .map(|bucket| {
            bucket
                .state_counts
                .iter()
                .map(|stat| stat.count)
                .sum::<usize>()
        })
        .max()
        .unwrap_or(1) as f64;
    html! {
        div class="space-y-2" {
            (subsection_heading("Time Distribution"))
            @for bucket in buckets {
                @let total: usize = bucket.state_counts.iter().map(|stat| stat.count).sum();
                @let width = ((total as f64 / max_total) * 100.0).max(5.0);
                div class="flex items-center gap-3" {
                    span class="w-24 text-xs font-mono text-gray-500" { (bucket.bucket.clone()) }
                    div class="flex-1 h-3 rounded bg-gray-100" {
                        div class="h-3 rounded bg-navy-700" style=(format!("width: {:.2}%", width)) {}
                    }
                    span class="text-xs font-semibold text-navy-900" { (total) }
                }
            }
        }
    }
}
