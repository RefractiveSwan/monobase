use maud::{Markup, html};
use refractive_swan_contracts::pipeline::MappingState;

use crate::view_model::{MappingResultsView, ServiceRequestSummary};
use crate::views::components::{badge::*, card::*, table::*};

pub(crate) fn render_results_panel(results: &MappingResultsView) -> Markup {
    html! {
        (card(html! {
            (card_header("Mapping Results", Some(html! {
                span class="text-xs font-mono text-gray-500" { (format!("Total: {}", results.request_summary.total)) }
            })))
            (render_results_summary(&results.request_summary))
            (render_results_table(&results.rows))
        }))
    }
}

fn render_results_summary(summary: &ServiceRequestSummary) -> Markup {
    html! {
        div class="p-4 grid gap-3 md:grid-cols-3 border-b border-gray-100" {
            (render_summary_total_card(summary.total))
            (render_summary_stat_card("SR Status", &summary.statuses, "No status values"))
            (render_summary_stat_card("SR Intent", &summary.intents, "No intents"))
        }
    }
}

fn render_summary_total_card(total: usize) -> Markup {
    html! {
        div class="rounded-md border border-gray-200 p-3" {
            p class="text-xs font-mono text-gray-500 uppercase" { "stg_servicerequest_flat" }
            p class="text-xl font-serif font-bold mt-1" { (total) }
        }
    }
}

fn render_summary_stat_card(
    title: &str,
    stats: &[crate::view_model::CountStat],
    empty_msg: &str,
) -> Markup {
    html! {
        div class="rounded-md border border-gray-200 p-3" {
            p class="text-xs font-semibold text-gray-600 uppercase" { (title) }
            @if stats.is_empty() {
                p class="text-sm text-gray-500 mt-1" { (empty_msg) }
            } @else {
                ul class="mt-2 space-y-1 text-xs" {
                    @for stat in stats {
                        li class="flex justify-between" {
                            span { (&stat.label) }
                            span class="font-medium" { (&stat.count) }
                        }
                    }
                }
            }
        }
    }
}

fn render_results_table(rows: &[crate::view_model::MappingRowView]) -> Markup {
    html! {
        div class="overflow-x-auto" {
            table class="min-w-full divide-y divide-gray-200 text-sm" {
                (table_header(&["ServiceRequest", "Code Element", "NCIt Concept", "State"]))
                tbody class="bg-white divide-y divide-gray-200" {
                    @if rows.is_empty() {
                        (render_empty_results_row())
                    } @else {
                        @for row in rows {
                            (render_mapping_row(row))
                        }
                    }
                }
            }
        }
    }
}

fn render_empty_results_row() -> Markup {
    html! {
        tr {
            td colspan="4" class="px-6 py-8 text-center text-gray-500 italic" {
                "No mapping rows generated."
            }
        }
    }
}

fn render_mapping_row(row: &crate::view_model::MappingRowView) -> Markup {
    table_row(html! {
        (render_service_request_cell(&row.sr_id, &row.system))
        (render_code_element_cell(&row.code, &row.display))
        (render_ncit_concept_cell(row.ncit_id.as_ref(), row.ncit_label.as_ref()))
        (render_mapping_state_cell(row.state, row.reason.as_ref()))
    })
}

fn render_service_request_cell(sr_id: &str, system: &str) -> Markup {
    table_cell(html! {
        div class="font-mono text-xs font-medium text-navy-900" { (sr_id) }
        div class="text-xs text-gray-500" { (system) }
    })
}

fn render_code_element_cell(code: &str, display: &str) -> Markup {
    table_cell(html! {
        div class="font-mono text-xs font-semibold" { (code) }
        div class="text-xs text-gray-500" { (display) }
    })
}

fn render_ncit_concept_cell(ncit_id: Option<&String>, ncit_label: Option<&String>) -> Markup {
    table_cell(html! {
        @if let Some(id) = ncit_id {
            div class="font-mono text-xs font-medium text-navy-900" { (id) }
            @if let Some(label) = ncit_label {
                div class="text-xs text-gray-600" { (label) }
            }
        } @else {
            span class="text-gray-400 italic" { "-" }
        }
    })
}

fn render_mapping_state_cell(state: MappingState, reason: Option<&String>) -> Markup {
    table_cell(html! {
        (state_chip(match state {
            MappingState::AutoMapped => "auto_mapped",
            MappingState::NeedsReview => "needs_review",
            MappingState::NoMatch => "no_match",
        }))
        @if let Some(r) = reason {
            div class="mt-1 text-xs text-rose-600 font-medium" { (r) }
        }
    })
}
