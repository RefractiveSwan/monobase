use maud::{Markup, html};
use refractive_swan_contracts::eval::EvalSummary;

use crate::view_model::{MappingResultsView, PageContext};
use crate::views::components::{button::*, card::*, layout::*, table::*, typography::*};
use crate::views::layout::base_layout;

pub fn render_eval_page(ctx: &PageContext) -> String {
    base_layout(page_container(html! {
        (render_eval_section(ctx))
    }))
    .into_string()
}

pub fn render_eval_fragment(run: &crate::client::EvalRunResponse) -> String {
    render_eval_summary(&run.summary, &run.dataset).into_string()
}

pub(crate) fn render_no_match_explorer(results: Option<&MappingResultsView>) -> Markup {
    html! {
        (card(html! {
            (card_header("NoMatch Explorer", None))
            (card_body(html! {
                @if let Some(view) = results {
                    @if view.no_matches.is_empty() {
                        p class="text-sm text-emerald-700 font-medium" { "✓ No unmapped codes found in this run." }
                    } @else {
                        (table_container(html! {
                            (table_header(&["ServiceRequest", "Code", "Reason"]))
                            tbody class="bg-white divide-y divide-gray-200" {
                                @for row in &view.no_matches {
                                    (table_row(html! {
                                        (table_cell(html! {
                                            p class="font-mono text-xs font-medium" { (&row.sr_id) }
                                            p class="text-xs text-gray-500" { (&row.system) }
                                        }))
                                        (table_cell(html! {
                                            p class="font-mono text-xs font-semibold" { (&row.code) }
                                            p class="text-xs text-gray-500" { (&row.display) }
                                        }))
                                        (table_cell(html! {
                                            span class="inline-flex rounded bg-rose-50 px-2 py-0.5 text-xs font-medium text-rose-800 border border-rose-100" {
                                                (row.reason.as_deref().unwrap_or("unknown"))
                                            }
                                        }))
                                    }))
                                }
                            }
                        }))
                    }
                } @else {
                    div class="text-center py-12" {
                        // Magnifying glass icon (Heroicons: magnifying-glass)
                        svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z" {}
                        }
                        h3 class="mt-4 text-sm font-semibold text-gray-900" { "No Data Yet" }
                        p class="mt-2 text-sm text-gray-500 max-w-sm mx-auto" { "Submit a FHIR Bundle to identify codes that couldn't be mapped to NCIt concepts." }
                    }
                }
            }))
        }))
    }
}

fn render_eval_section(ctx: &PageContext) -> Markup {
    html! {
        (card(html! {
            (card_header("Evaluation", Some(html! {
                span class="text-sm text-gray-500" { "Refractive Swan mapping eval datasets" }
            })))
            (card_body(html! {
                 form hx-post="/eval/run" hx-target="#eval-fragment" hx-swap="innerHTML" class="flex flex-wrap gap-3 items-center text-sm" {
                    (label_text("Dataset"))
                    select id="dataset" name="dataset" class="rounded-md border-gray-300 px-3 py-1.5 text-sm" {
                        @for ds in &ctx.datasets {
                            option value=(ds.name) selected[(ctx.selected_eval_dataset == ds.name)] { (format!("{} ({} rows)", ds.name, ds.n_cases)) }
                        }
                    }
                    (label_text("Top K"))
                    input type="number" id="top_k" name="top_k" value="1" min="1" max="5" class="w-16 rounded-md border-gray-300 px-2 py-1 text-sm" {}
                    (primary_button(ButtonProps {
                        text: "Run eval",
                        type_: "submit",
                        ..ButtonProps::default()
                    }))
                }
                div id="eval-fragment" class="mt-4" {
                    @if let Some(eval) = &ctx.eval {
                        (render_eval_summary(&eval.summary, &eval.dataset))
                    } @else {
                        p class="text-sm text-gray-500" { "No eval summary available yet." }
                    }
                }
            }))
        }))
    }
}

fn render_eval_summary(summary: &EvalSummary, dataset: &str) -> Markup {
    html! {
        div class="space-y-6" {
            div class="flex items-center justify-between" {
                (subsection_heading(&format!("Dataset: {}", dataset)))
                span class="text-sm text-gray-500" { (format!("Total cases: {}", summary.total_cases)) }
            }
            div class="grid gap-4 md:grid-cols-3" {
                (metric_card("Precision", &format!("{:.1}%", summary.precision * 100.0), None))
                (metric_card("Recall", &format!("{:.1}%", summary.recall * 100.0), None))
                (metric_card("Coverage", &format!("{:.1}%", summary.coverage * 100.0), None))
            }
            div class="grid gap-4 md:grid-cols-3" {
                (metric_card("Top1 accuracy", &format!("{:.1}%", summary.top1_accuracy * 100.0), None))
                (metric_card("Top3 accuracy", &format!("{:.1}%", summary.top3_accuracy * 100.0), None))
                (metric_card("AutoMapped precision", &format!("{:.1}%", summary.auto_mapped_precision * 100.0), None))
            }
            div class="bg-gray-50 rounded-md border border-gray-200 p-4" {
                h4 class="text-xs font-bold text-gray-500 uppercase tracking-wide mb-2" { "State counts" }
                ul class="text-sm text-gray-600 space-y-1 font-mono" {
                    @for (state, count) in &summary.state_counts {
                        li { (format!("{state}: {count}")) }
                    }
                }
            }
            div class="bg-gray-50 rounded-md border border-gray-200 p-4" {
                h4 class="text-xs font-bold text-gray-500 uppercase tracking-wide mb-2" { "Top NoMatch reasons" }
                ul class="text-sm text-gray-600 space-y-1 font-mono" {
                    @for (reason, count) in &summary.reason_counts {
                        li { (format!("{reason}: {count}")) }
                    }
                }
            }
        }
    }
}
