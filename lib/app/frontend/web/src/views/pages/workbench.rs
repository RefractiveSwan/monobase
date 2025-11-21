use maud::{Markup, html};
use refractive_swan_contracts::PipelineMetrics;

use crate::view_model::{AlertKind, AlertMessage, PageContext};
use crate::views::components::{badge::*, button::*, card::*, input::*, layout::*, typography::*};
use crate::views::layout::base_layout;
use crate::views::pages::eval::render_no_match_explorer;
use crate::views::partials::fragments::{render_eval_panel, render_metrics_dashboard};
use crate::views::partials::results::render_results_panel;

pub fn render_workbench_page(ctx: &PageContext) -> String {
    base_layout(page_container(html! {
        (render_workbench_hero(ctx))
        (render_input_section())
        (render_results_section(ctx))
        @if let Some(metrics) = &ctx.metrics {
            (render_metrics_dashboard(Some(metrics)))
        }
        (render_eval_panel(ctx))
    }))
    .into_string()
}

fn render_workbench_hero(ctx: &PageContext) -> Markup {
    card(card_body(html! {
        div class="flex flex-col md:flex-row md:items-start md:justify-between gap-4" {
            (render_hero_content())
            (render_hero_status_badges(ctx))
        }
        (render_hero_error(ctx))
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

fn render_quick_metrics(metrics: &PipelineMetrics) -> Markup {
    html! {
        div class="text-xs text-slate-500 font-mono text-right" {
            div { (format!("Bundles: {}", metrics.bundle_count)) }
            div { (format!("Mapped: {}", metrics.auto_mapped)) }
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

fn render_input_section() -> Markup {
    grid_section(html! {
        (render_paste_input_card())
        (render_upload_input_card())
    })
}

fn render_paste_input_card() -> Markup {
    card(html! {
        (card_header("Input: Paste JSON", None))
        (card_body(html! {
            form hx-post="/map/paste" hx-target="#results" hx-swap="innerHTML" method="post" class="h-full flex flex-col space-y-4" {
                label class="block" for="bundle_text" {
                    (label_text("JSON Payload"))
                    (helper_text("(Paste FHIR Bundle)"))
                }
                (textarea("bundle_text", 8, "{\"resourceType\": \"Bundle\", \"type\": \"collection\", ...}"))
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

fn render_results_section(ctx: &PageContext) -> Markup {
    html! {
        div id="results" {
            @if let Some(results) = &ctx.results {
                (render_results_panel(results))
                div class="mt-8" {
                    (render_no_match_explorer(Some(results)))
                }
            } @else {
                (render_no_match_explorer(None))
            }
        }
    }
}
