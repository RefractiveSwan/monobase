use dfps_core::mapping::MappingState;
use dfps_observability::PipelineMetrics;
use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::view_model::{
    AlertKind, AlertMessage, CohortRowView, CohortView, MappingResultsView, PageContext,
};

pub fn render_page(ctx: &PageContext) -> String {
    html! {
        (DOCTYPE)
        html class="h-full bg-slate-100" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "DFPS Mapping Workbench" }
                script src="https://cdn.tailwindcss.com" {}
                script src="https://unpkg.com/htmx.org@1.9.12" {}
            }
            body class="min-h-screen bg-slate-100 text-slate-900" {
                main class="mx-auto max-w-6xl px-4 py-10 space-y-8" {
                    section class="bg-white shadow-sm rounded-xl p-6 space-y-4" {
                        h1 class="text-2xl font-semibold" { "FHIR + NCIt mapping workbench" }
                        p class="text-slate-600" {
                            "Paste a FHIR Bundle or upload JSON so the DFPS pipeline can flatten ServiceRequests into "
                            code { "stg_servicerequest_flat" }
                            " and "
                            code { "stg_sr_code_exploded" }
                            ", then emit "
                            code { "MappingResult" }
                            " rows."
                        }
                        div class="flex flex-wrap gap-4 text-sm" {
                            @if let Some(health) = &ctx.health {
                                span class={(format!("inline-flex items-center gap-2 rounded-full px-3 py-1 text-xs font-medium {}",
                                    if health.ok { "bg-emerald-100 text-emerald-800" } else { "bg-amber-100 text-amber-800" }
                                ))} {
                                    span class={(if health.ok { "h-2 w-2 rounded-full bg-emerald-500" } else { "h-2 w-2 rounded-full bg-amber-500" })} {}
                                    span { (format!("Backend health: {}", health.status)) }
                                }
                            } @else {
                                span class="inline-flex items-center rounded-full bg-amber-100 px-3 py-1 text-xs font-medium text-amber-800" {
                                    "Backend health unknown"
                                }
                            }
                            @if let Some(metrics) = &ctx.metrics {
                                span class="inline-flex items-center rounded-full bg-slate-100 px-3 py-1 text-xs text-slate-700" {
                                    (format!("Bundles processed: {} | AutoMapped: {} | NeedsReview: {} | NoMatch: {}",
                                        metrics.bundle_count,
                                        metrics.auto_mapped,
                                        metrics.needs_review,
                                        metrics.no_match
                                    ))
                                }
                            }
                        }
                        @if let Some(error) = &ctx.health_error {
                            div class="rounded-lg border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-900" {
                                strong class="font-semibold" { "Backend warning: " }
                                span { (error) }
                            }
                        }
                        div class="mt-2 grid gap-4 md:grid-cols-2 text-sm text-slate-600" {
                            div class="rounded-lg border border-slate-200 bg-slate-50 p-4 space-y-2" {
                                h3 class="text-base font-semibold text-slate-800" { "Mapping state glossary" }
                                ul class="space-y-2" {
                                    li {
                                        strong { "AutoMapped. " }
                                        "NCIt concept met lexical + semantic thresholds with no manual review."
                                    }
                                    li {
                                        strong { "NeedsReview. " }
                                        "MappingResult landed near the quality threshold and should be validated before surfacing."
                                    }
                                    li {
                                        strong { "NoMatch. " }
                                        "The NCIt + mock UMLS crosswalk could not resolve a concept for the ServiceRequest code."
                                    }
                                }
                            }
                            div class="rounded-lg border border-slate-200 bg-slate-50 p-4 space-y-2" {
                                h3 class="text-base font-semibold text-slate-800" { "How the mapping engine works" }
                                p {
                                    "Bundles are ingested, flattened into "
                                    code { "stg_servicerequest_flat" }
                                    " and "
                                    code { "stg_sr_code_exploded" }
                                    ", then cross-referenced against NCIt concepts plus mock UMLS xrefs."
                                }
                                p { "Each MappingResult links back to NCIt metadata so reviewers can track which concepts were used for AutoMapped rows." }
                            }
                        }
                    }
                    section class="grid gap-6 lg:grid-cols-2" {
                        div class="bg-white shadow-sm rounded-xl p-6" {
                            h2 class="text-lg font-semibold" { "Paste Bundle JSON" }
                            form hx-post="/map/paste" hx-target="#results" hx-swap="innerHTML" method="post" class="mt-4 space-y-4" {
                                label class="block text-sm font-medium text-slate-700" for="bundle_text" {
                                    "FHIR Bundle JSON"
                                }
                                textarea name="bundle_text" id="bundle_text" rows="10" class="w-full rounded-lg border border-slate-300 p-3 font-mono text-sm focus:border-emerald-500 focus:ring-emerald-200" placeholder="{ \"resourceType\": \"Bundle\", ... }" {}
                                button type="submit" class="inline-flex items-center rounded-lg bg-emerald-600 px-4 py-2 text-white font-medium hover:bg-emerald-700" {
                                    "Map bundle"
                                }
                            }
                        }
                        div class="bg-white shadow-sm rounded-xl p-6" {
                            h2 class="text-lg font-semibold" { "Upload Bundle JSON" }
                            form hx-post="/map/upload" hx-target="#results" hx-swap="innerHTML" method="post" enctype="multipart/form-data" class="mt-4 space-y-4" {
                                label class="block text-sm font-medium text-slate-700" for="bundle_file" {
                                    "JSON file"
                                }
                                input type="file" id="bundle_file" name="bundle_file" accept="application/json,.json,.ndjson" class="w-full rounded-lg border border-dashed border-slate-300 p-3 text-sm" {}
                                button type="submit" class="inline-flex items-center rounded-lg bg-slate-800 px-4 py-2 text-white font-medium hover:bg-slate-900" {
                                    "Upload & map"
                                }
                            }
                        }
                    }
                    section id="results" class="space-y-4" {
                        (render_results(ctx))
                    }
                    (render_metrics_dashboard(ctx.metrics.as_ref()))
                    (render_analytics_panels(ctx))
                    (render_eval_panel(ctx))
                    (render_no_match_explorer(ctx.results.as_ref()))
                }
            }
        }
    }
    .into_string()
}

pub fn render_results_fragment(ctx: &PageContext) -> String {
    render_results(ctx).into_string()
}

fn render_results(ctx: &PageContext) -> Markup {
    html! {
        @if let Some(alert) = &ctx.alert {
            (render_alert(alert))
        }
        @if let Some(results) = &ctx.results {
            (render_results_panel(results))
        } @else {
            div class="bg-white rounded-xl border border-dashed border-slate-200 p-6 text-center text-slate-500" {
                p { "Results land here once the backend emits MappingResult rows." }
                p class="text-sm mt-2" { "Submit a Bundle (single object, array, or NDJSON) so ingestion can populate stg_servicerequest_flat and stg_sr_code_exploded." }
            }
        }
    }
}

fn render_metrics_dashboard(metrics: Option<&PipelineMetrics>) -> Markup {
    html! {
        section class="bg-white shadow-sm rounded-xl p-6 space-y-5" id="metrics-dashboard" {
            div class="flex items-center justify-between" {
                h2 class="text-xl font-semibold" { "Pipeline metrics" }
                span class="text-sm text-slate-500" {
                    @if let Some(mode) = metrics.and_then(|m| m.compliance_mode.as_deref()) {
                        (format!("Snapshot from GET /metrics/summary (compliance: {mode})"))
                    } @else {
                        "Snapshot from GET /metrics/summary"
                    }
                }
            }
            @if let Some(metrics) = metrics {
                div class="grid gap-4 md:grid-cols-3" {
                    (metric_card("Bundles processed", metrics.bundle_count, "Total number of Bundle mapping runs recorded.", "text-emerald-600"))
                    (metric_card("ServiceRequest flats", metrics.flats_count, "Flattened SR rows emitted by ingestion.", "text-slate-700"))
                    (metric_card("Mapping attempts", metrics.mapping_count, "Total MappingResult entries generated.", "text-slate-700"))
                }
                div class="grid gap-4 md:grid-cols-3" {
                    (state_metric_card("AutoMapped", metrics.auto_mapped, "bg-emerald-100 text-emerald-900", "Lexical matching cleared thresholds without reviewer help."))
                    (state_metric_card("Needs review", metrics.needs_review, "bg-amber-100 text-amber-900", "Score fell into the review band; confirm the NCIt suggestion manually."))
                    (state_metric_card("No match", metrics.no_match, "bg-rose-100 text-rose-900", "No NCIt concept resolved even after mock UMLS crosswalks."))
                }
                div class="grid gap-4 md:grid-cols-3" {
                    (metric_card("License blocked", metrics.license_blocked, "Mappings halted due to compliance mode tier restrictions.", "text-rose-700"))
                    (metric_card("Vector queries", metrics.vector_queries, "Vector search requests issued (if enabled).", "text-slate-700"))
                    (metric_card("Vector fallbacks", metrics.vector_fallbacks, "Times vector search was skipped/disabled.", "text-slate-700"))
                }
                div class="grid gap-4 md:grid-cols-3" {
                    (metric_card("Analytics requests", metrics.analytics_requests, "Count of calls to analytics endpoints.", "text-indigo-700"))
                    (metric_card("Cohort queries", metrics.cohort_queries, "Number of cohort filter requests processed.", "text-indigo-700"))
                    (metric_card_text("Avg cohort size", metrics.avg_cohort_size.map(|v| format!("{:.1}", v)).unwrap_or_else(|| "n/a".into()), "Mean rows returned per cohort query.", "text-indigo-700"))
                }
            } @else {
                p class="text-sm text-slate-500" {
                    "Run a mapping request to populate live metrics. The dashboard refreshes on each page load."
                }
            }
        }
    }
}

fn render_analytics_panels(ctx: &PageContext) -> Markup {
    html! {
        section class="grid gap-6 lg:grid-cols-2" id="analytics-overview" {
            div class="bg-white shadow-sm rounded-xl p-6 space-y-4" {
                div class="flex items-center justify-between" {
                    h2 class="text-xl font-semibold" { "Analytics overview" }
                    span class="text-sm text-slate-500" { "GET /analytics/ncit-summary" }
                }
                @if let Some(error) = &ctx.analytics_error {
                    (render_alert(&AlertMessage { kind: AlertKind::Error, text: error.clone() }))
                } @else if let Some(summary) = &ctx.analytics_summary {
                    (render_top_concepts(&summary.top_concepts))
                    (render_state_distribution(&summary.state_counts))
                    (render_time_buckets(&summary.time_buckets))
                } @else {
                    p class="text-sm text-slate-500" { "No analytics summary yet. Submit mapping runs or hit /analytics after sending Bundles." }
                }
            }
            div class="bg-white shadow-sm rounded-xl p-6 space-y-4" {
                div class="flex items-center justify-between" {
                    h2 class="text-xl font-semibold" { "Cohort exploration" }
                    span class="text-sm text-slate-500" { "GET /analytics/cohort" }
                }
                form method="get" action="/analytics" class="grid gap-3 md:grid-cols-2 text-sm" {
                    label class="flex flex-col gap-1" for="ncit_id" {
                        span class="text-slate-700" { "NCIt ID" }
                        input type="text" id="ncit_id" name="ncit_id" value=(ctx.cohort_filters.ncit_id.clone().unwrap_or_default()) placeholder="CXXXX" class="rounded-lg border border-slate-300 px-3 py-2" {}
                    }
                    label class="flex flex-col gap-1" for="status" {
                        span class="text-slate-700" { "Status" }
                        input type="text" id="status" name="status" value=(ctx.cohort_filters.status.clone().unwrap_or_default()) placeholder="active|completed" class="rounded-lg border border-slate-300 px-3 py-2" {}
                    }
                    label class="flex flex-col gap-1" for="date_from" {
                        span class="text-slate-700" { "Date from (YYYY-MM-DD)" }
                        input type="text" id="date_from" name="date_from" value=(ctx.cohort_filters.date_from.clone().unwrap_or_default()) class="rounded-lg border border-slate-300 px-3 py-2" {}
                    }
                    label class="flex flex-col gap-1" for="date_to" {
                        span class="text-slate-700" { "Date to (YYYY-MM-DD)" }
                        input type="text" id="date_to" name="date_to" value=(ctx.cohort_filters.date_to.clone().unwrap_or_default()) class="rounded-lg border border-slate-300 px-3 py-2" {}
                    }
                    button type="submit" class="inline-flex items-center justify-center rounded-lg bg-emerald-600 px-4 py-2 text-white font-medium hover:bg-emerald-700 md:col-span-2" {
                        "Apply cohort filters"
                    }
                }
                @if let Some(error) = &ctx.cohort_error {
                    (render_alert(&AlertMessage { kind: AlertKind::Error, text: error.clone() }))
                }
                @if let Some(cohort) = &ctx.cohort {
                    p class="text-sm text-slate-600" { (format!("Matched {} orders", cohort.total)) }
                    (render_cohort_table(cohort))
                } @else {
                    p class="text-sm text-slate-500" { "Run a cohort query to see resolved dim/fact rows." }
                }
            }
        }
    }
}

fn render_top_concepts(concepts: &[crate::view_model::AnalyticsConceptTile]) -> Markup {
    html! {
        div class="space-y-3" {
            h3 class="text-sm font-semibold text-slate-800" { "Top NCIt concepts" }
            @if concepts.is_empty() {
                p class="text-sm text-slate-500" { "No concepts yet." }
            } @else {
                div class="grid gap-3 sm:grid-cols-2" {
                    @for concept in concepts {
                        div class="rounded-lg border border-slate-200 p-3" {
                            p class="text-sm font-semibold text-slate-800" { (concept.preferred_name.clone()) }
                            p class="text-xs text-slate-500" { (concept.ncit_id.clone()) }
                            p class="text-xs text-slate-500 mt-1" { (format!("Count: {}", concept.total)) }
                        }
                    }
                }
            }
        }
    }
}

fn render_state_distribution(states: &[crate::view_model::CountStat]) -> Markup {
    html! {
        div class="space-y-2" {
            h3 class="text-sm font-semibold text-slate-800" { "Mapping states" }
            @for stat in states {
                div class="flex items-center justify-between rounded-lg bg-slate-50 px-3 py-2 text-sm text-slate-700" {
                    span { (stat.label.clone()) }
                    span class="font-semibold" { (stat.count) }
                }
            }
            @if states.is_empty() {
                p class="text-sm text-slate-500" { "No mappings observed yet." }
            }
        }
    }
}

fn render_time_buckets(buckets: &[crate::view_model::AnalyticsTimeBucket]) -> Markup {
    html! {
        div class="space-y-2" {
            h3 class="text-sm font-semibold text-slate-800" { "Counts by time bucket" }
            @if buckets.is_empty() {
                p class="text-sm text-slate-500" { "No ordered_at timestamps available." }
            } @else {
                table class="min-w-full border border-slate-200 rounded-lg text-sm" {
                    thead class="bg-slate-50" {
                        tr {
                            th class="px-3 py-2 text-left text-slate-600" { "Date" }
                            th class="px-3 py-2 text-left text-slate-600" { "State counts" }
                        }
                    }
                    tbody {
                        @for bucket in buckets {
                            tr class="border-t border-slate-200" {
                                td class="px-3 py-2" { (bucket.bucket.clone()) }
                                td class="px-3 py-2 space-x-2" {
                                    @for stat in &bucket.state_counts {
                                        span class="inline-flex items-center rounded-full bg-slate-100 px-2 py-1 text-xs text-slate-700" {
                                            (format!("{}: {}", stat.label, stat.count))
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn render_cohort_table(cohort: &CohortView) -> Markup {
    html! {
        div class="overflow-x-auto" {
            table class="min-w-full border border-slate-200 rounded-lg text-sm" {
                thead class="bg-slate-50" {
                    tr {
                        th class="px-3 py-2 text-left text-slate-600" { "SR ID" }
                        th class="px-3 py-2 text-left text-slate-600" { "Patient" }
                        th class="px-3 py-2 text-left text-slate-600" { "Encounter" }
                        th class="px-3 py-2 text-left text-slate-600" { "NCIt" }
                        th class="px-3 py-2 text-left text-slate-600" { "Status" }
                        th class="px-3 py-2 text-left text-slate-600" { "Intent" }
                        th class="px-3 py-2 text-left text-slate-600" { "Ordered at" }
                        th class="px-3 py-2 text-left text-slate-600" { "State" }
                    }
                }
                tbody {
                    @for row in &cohort.rows {
                        (render_cohort_row(row))
                    }
                }
            }
        }
    }
}

fn render_cohort_row(row: &CohortRowView) -> Markup {
    html! {
        tr class="border-t border-slate-200" {
            td class="px-3 py-2 font-medium text-slate-800" { (row.sr_id.clone()) }
            td class="px-3 py-2" { (row.patient_id.clone()) }
            td class="px-3 py-2" { (row.encounter_id.clone()) }
            td class="px-3 py-2" { (row.ncit_id.clone()) }
            td class="px-3 py-2" { (row.status.clone()) }
            td class="px-3 py-2" { (row.intent.clone()) }
            td class="px-3 py-2" { (row.ordered_at.clone()) }
            td class="px-3 py-2" { (row.mapping_state.clone()) }
        }
    }
}

fn render_eval_panel(ctx: &PageContext) -> Markup {
    html! {
        section class="bg-white shadow-sm rounded-xl p-6 space-y-4" id="eval-panel" {
            div class="flex items-center justify-between" {
                h2 class="text-xl font-semibold" { "Mapping evaluation snapshot" }
                span class="text-sm text-slate-500" { "HTMX fragment from dfps_eval::report" }
            }
            div class="flex flex-wrap items-center gap-3 text-sm" {
                label class="text-slate-600" for="eval-dataset" { "Dataset" }
                select id="eval-dataset" name="dataset" class="rounded-lg border border-slate-300 px-3 py-2 text-sm"
                    hx-get="/eval/report"
                    hx-target="#eval-report-fragment"
                    hx-swap="innerHTML"
                    hx-trigger="change" {
                    @if !ctx.datasets.is_empty() {
                        @for dataset in &ctx.datasets {
                            option value=(dataset.name) selected[(ctx.selected_eval_dataset == dataset.name)] { (dataset.name.clone()) }
                        }
                    } @else {
                        // Fallback if dataset list failed to load.
                        option value=(ctx.selected_eval_dataset) { (ctx.selected_eval_dataset.clone()) }
                    }
                }
            }
            div id="eval-report-fragment" class="rounded-lg border border-slate-200 bg-slate-50 p-4" {
                @if let Some(html) = &ctx.eval_report_html {
                    (PreEscaped(html))
                } @else if let Some(err) = &ctx.eval_panel_error {
                    p class="text-sm text-rose-700" { (err) }
                } @else {
                    p class="text-sm text-slate-500" { "Run dfps_cli eval_mapping --out-dir to populate eval artifacts, then reload this page." }
                }
            }
        }
    }
}

fn render_no_match_explorer(results: Option<&MappingResultsView>) -> Markup {
    html! {
        section class="bg-white shadow-sm rounded-xl p-6 space-y-4" id="no-match-explorer" {
            div class="flex items-center justify-between" {
                h2 class="text-xl font-semibold" { "NoMatch explorer" }
                span class="text-sm text-slate-500" { "Codes that need NCIt follow-up" }
            }
            @if let Some(view) = results {
                @if view.no_matches.is_empty() {
                    div class="rounded-lg border border-slate-200 bg-slate-50 p-5 text-sm text-slate-600" {
                        "Great news--your latest mapping run did not emit any MappingState::NoMatch rows."
                    }
                } @else {
                    div class="overflow-x-auto" {
                        table class="min-w-full divide-y divide-slate-200" {
                            thead class="bg-slate-50" {
                                tr {
                                    th class="px-4 py-2 text-left text-xs font-semibold uppercase tracking-wide text-slate-600" { "ServiceRequest" }
                                    th class="px-4 py-2 text-left text-xs font-semibold uppercase tracking-wide text-slate-600" { "Code" }
                                    th class="px-4 py-2 text-left text-xs font-semibold uppercase tracking-wide text-slate-600" { "Reason" }
                                }
                            }
                            tbody class="divide-y divide-slate-100 bg-white" {
                                @for row in &view.no_matches {
                                    tr {
                                        td class="px-4 py-3 align-top" {
                                            p class="font-medium" { (&row.sr_id) }
                                            p class="text-sm text-slate-500" { (&row.system) }
                                        }
                                        td class="px-4 py-3 align-top" {
                                            p class="font-semibold" { (&row.code) }
                                            p class="text-sm text-slate-500" { (&row.display) }
                                        }
                                        td class="px-4 py-3 align-top" {
                                            span class="inline-flex rounded-full bg-rose-100 px-2.5 py-1 text-xs font-semibold text-rose-900" {
                                                (row.reason.as_deref().unwrap_or("unknown_reason"))
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } @else {
                div class="rounded-lg border border-dashed border-slate-200 p-5 text-sm text-slate-600" {
                    "Upload a Bundle or paste JSON to seed the explorer with actionable NoMatch rows."
                }
            }
        }
    }
}

pub fn render_eval_page(ctx: &PageContext) -> String {
    html! {
        (DOCTYPE)
        html class="h-full bg-slate-100" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "DFPS Eval" }
                script src="https://cdn.tailwindcss.com" {}
                script src="https://unpkg.com/htmx.org@1.9.12" {}
            }
            body class="min-h-screen bg-slate-100 text-slate-900" {
                main class="mx-auto max-w-5xl px-4 py-10 space-y-6" {
                    (render_eval_section(ctx))
                }
            }
        }
    }
    .into_string()
}

pub fn render_eval_fragment(run: &crate::client::EvalRunResponse) -> String {
    render_eval_summary(&run.summary, &run.dataset).into_string()
}

fn render_eval_section(ctx: &PageContext) -> Markup {
    html! {
        section class="bg-white shadow-sm rounded-xl p-6 space-y-4" {
            div class="flex items-center justify-between" {
                h2 class="text-xl font-semibold" { "Evaluation" }
                span class="text-sm text-slate-500" { "DFPS mapping eval datasets" }
            }
            form hx-post="/eval/run" hx-target="#eval-fragment" hx-swap="innerHTML" class="flex flex-wrap gap-3 items-center text-sm" {
                label for="dataset" { "Dataset" }
                select id="dataset" name="dataset" class="rounded-lg border border-slate-300 px-3 py-2 text-sm" {
                    @for ds in &ctx.datasets {
                        option value=(ds.name) selected[(ctx.selected_eval_dataset == ds.name)] { (format!("{} ({} rows)", ds.name, ds.n_cases)) }
                    }
                }
                label for="top_k" { "Top K" }
                input type="number" id="top_k" name="top_k" value="1" min="1" max="5" class="w-16 rounded-lg border border-slate-300 px-2 py-1 text-sm" {}
                button type="submit" class="inline-flex items-center rounded-lg bg-emerald-600 px-4 py-2 text-white font-medium hover:bg-emerald-700" { "Run eval" }
            }
            div id="eval-fragment" {
                @if let Some(eval) = &ctx.eval {
                    (render_eval_summary(&eval.summary, &eval.dataset))
                } @else {
                    p class="text-sm text-slate-500" { "No eval summary available yet." }
                }
            }
        }
    }
}

fn render_eval_summary(summary: &dfps_eval::EvalSummary, dataset: &str) -> Markup {
    html! {
        div class="space-y-4" {
            div class="flex items-center justify-between" {
                h3 class="text-lg font-semibold" { (format!("Dataset: {}", dataset)) }
                span class="text-sm text-slate-500" { (format!("Total cases: {}", summary.total_cases)) }
            }
            div class="grid gap-4 md:grid-cols-3" {
                (metric_card("Precision", (summary.precision * 100.0) as usize, "%", "text-emerald-600"))
                (metric_card("Recall", (summary.recall * 100.0) as usize, "%", "text-emerald-600"))
                (metric_card("Coverage", (summary.coverage * 100.0) as usize, "%", "text-emerald-600"))
            }
            div class="grid gap-4 md:grid-cols-3" {
                (metric_card("Top1 accuracy", (summary.top1_accuracy * 100.0) as usize, "%", "text-slate-700"))
                (metric_card("Top3 accuracy", (summary.top3_accuracy * 100.0) as usize, "%", "text-slate-700"))
                (metric_card("AutoMapped precision", (summary.auto_mapped_precision * 100.0) as usize, "%", "text-slate-700"))
            }
            div class="bg-slate-50 rounded-lg border border-slate-200 p-4" {
                h4 class="text-sm font-semibold text-slate-800 mb-2" { "State counts" }
                ul class="text-sm text-slate-600 space-y-1" {
                    @for (state, count) in &summary.state_counts {
                        li { (format!("{state}: {count}")) }
                    }
                }
            }
            div class="bg-slate-50 rounded-lg border border-slate-200 p-4" {
                h4 class="text-sm font-semibold text-slate-800 mb-2" { "Top NoMatch reasons" }
                ul class="text-sm text-slate-600 space-y-1" {
                    @for (reason, count) in &summary.reason_counts {
                        li { (format!("{reason}: {count}")) }
                    }
                }
            }
        }
    }
}

fn render_alert(alert: &AlertMessage) -> Markup {
    let (bg, text) = match alert.kind {
        AlertKind::Info => ("bg-emerald-50 text-emerald-900", "Info"),
        AlertKind::Error => ("bg-rose-50 text-rose-900", "Error"),
    };
    html! {
        div class={(format!("rounded-lg px-4 py-3 text-sm font-medium {bg}"))} {
            strong class="mr-2" { (text) ":" }
            span { (&alert.text) }
        }
    }
}

fn render_results_panel(results: &MappingResultsView) -> Markup {
    html! {
        div class="bg-white shadow rounded-xl p-6 space-y-6" {
            h2 class="text-xl font-semibold" { "MappingResult rows" }
            div class="grid gap-4 md:grid-cols-3" {
                div class="rounded-lg border border-slate-200 p-4" {
                    p class="text-sm font-mono text-slate-500" { "stg_servicerequest_flat" }
                    p class="text-2xl font-semibold" { (results.request_summary.total) }
                    p class="text-xs text-slate-500 mt-1" { "ServiceRequest rows produced by ingestion." }
                }
                div class="rounded-lg border border-slate-200 p-4" {
                    p class="text-sm font-semibold text-slate-600" { "ServiceRequest.status" }
                    @if results.request_summary.statuses.is_empty() {
                        p class="text-sm text-slate-500" { "No status values reported." }
                    } @else {
                        ul class="mt-2 space-y-1" {
                            @for stat in &results.request_summary.statuses {
                                li class="flex justify-between text-sm" {
                                    span { (&stat.label) }
                                    span class="font-medium" { (&stat.count) }
                                }
                            }
                        }
                    }
                }
                div class="rounded-lg border border-slate-200 p-4" {
                    p class="text-sm font-semibold text-slate-600" { "ServiceRequest.intent" }
                    @if results.request_summary.intents.is_empty() {
                        p class="text-sm text-slate-500" { "No intents reported." }
                    } @else {
                        ul class="mt-2 space-y-1" {
                            @for stat in &results.request_summary.intents {
                                li class="flex justify-between text-sm" {
                                    span { (&stat.label) }
                                    span class="font-medium" { (&stat.count) }
                                }
                            }
                        }
                    }
                }
            }
            div class="overflow-x-auto" {
                table class="min-w-full divide-y divide-slate-200" {
                    thead class="bg-slate-50" {
                        tr {
                            th class="px-4 py-2 text-left text-xs font-semibold uppercase tracking-wide text-slate-600" { "ServiceRequest (sr_id)" }
                            th class="px-4 py-2 text-left text-xs font-semibold uppercase tracking-wide text-slate-600" { "Code element" }
                            th class="px-4 py-2 text-left text-xs font-semibold uppercase tracking-wide text-slate-600" { "NCIt concept" }
                            th class="px-4 py-2 text-left text-xs font-semibold uppercase tracking-wide text-slate-600" { "Mapping state" }
                        }
                    }
                    tbody class="divide-y divide-slate-100 bg-white" {
                        @if results.rows.is_empty() {
                            tr {
                                td colspan="4" class="px-4 py-6 text-center text-slate-500" {
                                    "Backend returned 0 MappingResult rows. Confirm that codes landed in stg_sr_code_exploded."
                                }
                            }
                        } @else {
                            @for row in &results.rows {
                                tr {
                                    td class="px-4 py-3 align-top" {
                                        p class="font-medium" { (&row.sr_id) }
                                        p class="text-sm text-slate-500" { (&row.system) }
                                    }
                                    td class="px-4 py-3 align-top" {
                                        p class="font-semibold" { (&row.code) }
                                        p class="text-sm text-slate-500" { (&row.display) }
                                    }
                                    td class="px-4 py-3 align-top" {
                                        @if let Some(id) = &row.ncit_id {
                                            p class="font-medium" { (id) }
                                            @if let Some(label) = &row.ncit_label {
                                                p class="text-sm text-slate-500" { (label) }
                                            }
                                        } @else {
                                            span class="text-slate-400" { "--" }
                                        }
                                    }
                                    td class="px-4 py-3 align-top" {
                                        (state_chip(row.state))
                                        @if let Some(reason) = &row.reason {
                                            p class="mt-1 text-xs text-slate-500" {
                                                "MappingResult.reason: " (reason)
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
fn state_chip(state: MappingState) -> Markup {
    let (label, classes, tooltip) = match state {
        MappingState::AutoMapped => (
            "AutoMapped",
            "bg-emerald-100 text-emerald-900 ring-emerald-200",
            "Met lexical + semantic thresholds using NCIt and mock UMLS xrefs.",
        ),
        MappingState::NeedsReview => (
            "Needs review",
            "bg-amber-100 text-amber-900 ring-amber-200",
            "Below hard threshold; requires human validation before promoting.",
        ),
        MappingState::NoMatch => (
            "No match",
            "bg-rose-100 text-rose-900 ring-rose-200",
            "Pipeline could not locate an NCIt concept for the supplied code.",
        ),
    };
    html! {
        span title=(tooltip) class={(format!("inline-flex items-center rounded-full px-3 py-1 text-xs font-semibold ring-1 ring-inset {classes}"))} {
            (label)
        }
    }
}

fn metric_card(title: &str, value: usize, description: &str, accent: &str) -> Markup {
    html! {
        div class="rounded-lg border border-slate-200 p-4 shadow-sm" {
            p class="text-sm text-slate-500" { (title) }
            p class={(format!("text-3xl font-semibold {}", accent))} { (value) }
            p class="text-xs text-slate-500 mt-1" { (description) }
        }
    }
}

fn metric_card_text(title: &str, value: String, description: &str, accent: &str) -> Markup {
    html! {
        div class="rounded-lg border border-slate-200 p-4 shadow-sm" {
            p class="text-sm text-slate-500" { (title) }
            p class={(format!("text-3xl font-semibold {}", accent))} { (value) }
            p class="text-xs text-slate-500 mt-1" { (description) }
        }
    }
}

fn state_metric_card(title: &str, value: usize, classes: &str, tooltip: &str) -> Markup {
    html! {
        div class="rounded-lg border border-slate-200 p-4" {
            p class="text-sm text-slate-500" { (title) }
            div title=(tooltip) class={(format!("mt-2 inline-flex items-center rounded-full px-3 py-1 text-sm font-semibold {}", classes))} {
                (value) " entries"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_model::{
        CountStat, MappingResultsView, MappingRowView, NoMatchRowView, PageContext,
        ServiceRequestSummary,
    };

    #[test]
    fn render_page_shows_metrics_and_no_match_details() {
        let mut metrics = PipelineMetrics::default();
        metrics.bundle_count = 3;
        metrics.flats_count = 4;
        metrics.mapping_count = 5;
        metrics.auto_mapped = 2;
        metrics.needs_review = 1;
        metrics.no_match = 2;

        let results = MappingResultsView {
            request_summary: ServiceRequestSummary {
                total: 2,
                statuses: vec![CountStat {
                    label: "active".into(),
                    count: 2,
                }],
                intents: vec![CountStat {
                    label: "order".into(),
                    count: 2,
                }],
            },
            rows: vec![
                MappingRowView {
                    sr_id: "SR-1".into(),
                    system: "http://loinc.org".into(),
                    code: "24606-6".into(),
                    display: "FDG uptake".into(),
                    ncit_id: Some("C1234".into()),
                    ncit_label: Some("FDG Uptake".into()),
                    state: MappingState::AutoMapped,
                    reason: None,
                },
                MappingRowView {
                    sr_id: "SR-2".into(),
                    system: "http://loinc.org".into(),
                    code: "99999-9".into(),
                    display: "Unknown code".into(),
                    ncit_id: None,
                    ncit_label: None,
                    state: MappingState::NoMatch,
                    reason: Some("missing_system_or_code".into()),
                },
            ],
            no_matches: vec![NoMatchRowView {
                sr_id: "SR-2".into(),
                system: "http://loinc.org".into(),
                code: "99999-9".into(),
                display: "Unknown code".into(),
                reason: Some("missing_system_or_code".into()),
            }],
        };

        let mut ctx = PageContext::default();
        ctx.health = None;
        ctx.health_error = Some("Health endpoint unreachable: test".into());
        ctx.metrics = Some(metrics);
        ctx.results = Some(results);
        ctx.eval_report_html = Some("<div>Eval report</div>".into());

        let html = render_page(&ctx);
        assert!(html.contains("Pipeline metrics"));
        assert!(html.contains("NoMatch explorer"));
        assert!(html.contains("MappingResult.reason"));
        assert!(html.contains("missing_system_or_code"));
        assert!(html.contains("Backend warning"));
        assert!(html.contains("Mapping evaluation snapshot"));
        assert!(html.contains("Eval report"));
    }

    #[test]
    fn render_eval_page_shows_dataset_picker_and_metrics() {
        let mut ctx = PageContext::default();
        ctx.datasets = vec![dfps_eval::DatasetManifest {
            name: "pet_ct_small".into(),
            version: "20240601".into(),
            license: Some("test-license".into()),
            source: Some("test-source".into()),
            n_cases: 3,
            sha256: "abc123".into(),
            notes: None,
        }];
        ctx.selected_eval_dataset = "pet_ct_small".into();
        ctx.eval = Some(super::super::view_model::EvalContext {
            dataset: "pet_ct_small".into(),
            summary: dfps_eval::EvalSummary {
                total_cases: 3,
                precision: 0.97,
                recall: 0.97,
                coverage: 1.0,
                top1_accuracy: 0.97,
                top3_accuracy: 0.97,
                auto_mapped_precision: 0.98,
                state_counts: [("auto_mapped".into(), 3)].into_iter().collect(),
                reason_counts: [("missing_system_or_code".into(), 1)].into_iter().collect(),
                ..dfps_eval::EvalSummary::default()
            },
        });

        let html = render_eval_page(&ctx);
        assert!(html.contains("Dataset: pet_ct_small"));
        assert!(html.contains("Run eval"));
        assert!(html.contains("Top1 accuracy"));
        assert!(html.contains("missing_system_or_code"));
    }
}
