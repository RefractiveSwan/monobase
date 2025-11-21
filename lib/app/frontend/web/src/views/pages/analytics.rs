use maud::{Markup, html};

use crate::views::components::{badge::*, card::*, input::*, layout::*, table::*, typography::*};
use crate::views::layout::base_layout;
use crate::views::models::{AlertKind, AlertMessage, CohortView, PageContext};

pub fn render_analytics_page(ctx: &PageContext) -> String {
    base_layout(page_container(html! {
        (section_heading("Analytics Dashboard"))
        (render_analytics_panels(ctx))
    }))
    .into_string()
}

pub(crate) fn render_analytics_panels(ctx: &PageContext) -> Markup {
    html! {
        (grid_section(html! {
            (card(html! {
                (card_header("Analytics Summary", None))
                (card_body(html! {
                    div class="space-y-6" {
                        @if let Some(error) = &ctx.analytics_error {
                            (alert(&AlertMessage { kind: AlertKind::Error, text: error.clone() }))
                        } @else if let Some(summary) = &ctx.analytics_summary {
                            (render_top_concepts(&summary.top_concepts))
                            div class="border-t border-gray-100 pt-4" {
                                (render_state_distribution(&summary.state_counts))
                            }
                            div class="border-t border-gray-100 pt-4" {
                                (render_time_buckets(&summary.time_buckets))
                            }
                        } @else {
                            p class="text-sm text-gray-500 italic" { "No analytics data available." }
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

fn render_top_concepts(concepts: &[crate::view_model::AnalyticsConceptTile]) -> Markup {
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

fn render_state_distribution(states: &[crate::view_model::CountStat]) -> Markup {
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

fn render_time_buckets(buckets: &[crate::view_model::AnalyticsTimeBucket]) -> Markup {
    html! {
        div class="space-y-3" {
            (subsection_heading("Time Buckets"))
            @if buckets.is_empty() {
                p class="text-sm text-gray-500" { "No ordered_at timestamps available." }
            } @else {
                (table_container(html! {
                    (table_header(&["Date", "States"]))
                    tbody class="bg-white divide-y divide-gray-200" {
                        @for bucket in buckets {
                            (table_row(html! {
                                (table_cell_mono(html! { (bucket.bucket.clone()) }))
                                (table_cell(html! {
                                    @for stat in &bucket.state_counts {
                                        span class="inline-flex items-center rounded bg-gray-100 px-2 py-0.5 text-xs mr-2" {
                                            (format!("{}: {}", stat.label, stat.count))
                                        }
                                    }
                                }))
                            }))
                        }
                    }
                }))
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
