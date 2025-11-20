use dfps_contracts::{PipelineMetrics, eval::EvalSummary, pipeline::MappingState};
use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::components::{
    badge::*, button::*, card::*, input::*, layout::*, table::*, typography::*,
};
use crate::view_model::{AlertKind, AlertMessage, CohortView, MappingResultsView, PageContext};

/// The base layout for all pages, including header, footer, and common scripts/styles.
fn base_layout(content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html class="h-full bg-[#f8f9fa]" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "DFPS Mapping Workbench" }

                // Typography: Merriweather (Serif headings), Inter (Sans body), Roboto Mono (Code)
                link rel="preconnect" href="https://fonts.googleapis.com";
                link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="";
                link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600&family=Merriweather:ital,wght@0,300;0,400;0,700;1,300;1,400&family=Roboto+Mono:wght@400;500&display=swap" rel="stylesheet";

                script src="https://cdn.tailwindcss.com" {}
                script src="https://unpkg.com/htmx.org@1.9.12" {}

                // HTMX Loading Indicator Styles
                style {
                    (PreEscaped(r#"
                        .htmx-indicator {
                            display: none;
                        }
                        .htmx-request .htmx-indicator {
                            display: inline-block;
                        }
                        .htmx-request.htmx-indicator {
                            display: inline-block;
                        }
                        @keyframes spin {
                            to { transform: rotate(360deg); }
                        }
                        .animate-spin {
                            animation: spin 1s linear infinite;
                        }
                    "#))
                }

                // Academic Theme Configuration - Enhanced for Accessibility
                script {
                    (PreEscaped(r#"
                        tailwind.config = {
                            theme: {
                                extend: {
                                    fontFamily: {
                                        sans: ['Inter', 'sans-serif'],
                                        serif: ['Merriweather', 'serif'],
                                        mono: ['Roboto Mono', 'monospace'],
                                    },
                                    colors: {
                                        navy: {
                                            50: '#f0f4f8',
                                            100: '#d9e2ec',
                                            600: '#334155',  // Better contrast for secondary text
                                            700: '#1e293b',  // Better contrast for body text
                                            800: '#0f172a',
                                            900: '#020617',  // Deeper for maximum contrast
                                        },
                                        gold: {
                                            50: '#fefce8',
                                            100: '#fbf3db',
                                            500: '#a78b4a',  // Adjusted for better contrast
                                            600: '#8b7239',
                                        },
                                        paper: '#fafafa',  // Slightly lighter for better contrast
                                    },
                                    boxShadow: {
                                        'academic': '0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px 0 rgba(0, 0, 0, 0.06)',
                                        'academic-lg': '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
                                    },
                                    spacing: {
                                        '18': '4.5rem',
                                        '22': '5.5rem',
                                    }
                                }
                            }
                        }
                    "#))
                }
            }
            body class="min-h-screen bg-paper text-navy-900 font-sans antialiased" {
                // Header: Professional Navy Bar
                header class="bg-navy-900 text-white shadow-md" {
                    div class="mx-auto max-w-7xl px-6 py-4 flex items-center justify-between" {
                        div class="flex items-center gap-3" {
                            div class="h-8 w-8 rounded bg-gold-500 flex items-center justify-center text-navy-900 font-bold font-serif text-sm" { "D" }
                            h1 class="text-xl font-serif font-bold tracking-wide" { "DFPS Workbench" }
                        }
                        nav class="flex items-center gap-6 text-sm font-medium" {
                            a href="/map" class="flex items-center gap-1.5 text-gray-300 hover:text-white hover:underline underline-offset-4 transition-all duration-200" {
                                // Map icon (Heroicons: map)
                                svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 6.75V15m6-6v8.25m.503 3.498l4.875-2.437c.381-.19.622-.58.622-1.006V4.82c0-.836-.88-1.38-1.628-1.006l-3.869 1.934c-.317.159-.69.159-1.006 0L9.503 3.252a1.125 1.125 0 00-1.006 0L3.622 5.689C3.24 5.88 3 6.27 3 6.695V19.18c0 .836.88 1.38 1.628 1.006l3.869-1.934c.317-.159.69-.159 1.006 0l4.994 2.497c.317.158.69.158 1.006 0z" {}
                                }
                                span { "Mapping" }
                            }
                            a href="/analytics" class="flex items-center gap-1.5 text-gray-300 hover:text-white hover:underline underline-offset-4 transition-all duration-200" {
                                // Chart bar icon (Heroicons: chart-bar)
                                svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v6.75C7.5 20.496 6.996 21 6.375 21h-2.25A1.125 1.125 0 013 19.875v-6.75zM9.75 8.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v11.25c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V8.625zM16.5 4.125c0-.621.504-1.125 1.125-1.125h2.25C20.496 3 21 3.504 21 4.125v15.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V4.125z" {}
                                }
                                span { "Analytics" }
                            }
                            a href="/eval" class="flex items-center gap-1.5 text-gray-300 hover:text-white hover:underline underline-offset-4 transition-all duration-200" {
                                // Beaker icon (Heroicons: beaker)
                                svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 3.104v5.714a2.25 2.25 0 01-.659 1.591L5 14.5M9.75 3.104c-.251.023-.501.05-.75.082m.75-.082a24.301 24.301 0 014.5 0m0 0v5.714c0 .597.237 1.17.659 1.591L19.8 15.3M14.25 3.104c.251.023.501.05.75.082M19.8 15.3l-1.57.393A9.065 9.065 0 0112 15a9.065 9.065 0 00-6.23-.693L5 14.5m14.8.8l1.402 1.402c1.232 1.232.65 3.318-1.067 3.611A48.309 48.309 0 0112 21c-2.773 0-5.491-.235-8.135-.687-1.718-.293-2.3-2.379-1.067-3.61L5 14.5" {}
                                }
                                span { "Evaluation" }
                            }
                        }
                    }
                }

                (content)

                footer class="bg-white border-t border-gray-200 mt-12" {
                    div class="mx-auto max-w-7xl px-6 py-8" {
                        p class="text-center text-xs text-gray-500 font-serif italic" {
                            "DFPS Clinical Model • Project Hierophancy"
                        }
                    }
                }
            }
        }
    }
}

pub fn render_landing_page() -> String {
    base_layout(html! {
        // Hero Section
        div class="relative isolate overflow-hidden bg-navy-900 py-24 sm:py-32" {
            div class="mx-auto max-w-7xl px-6 lg:px-8" {
                div class="mx-auto max-w-2xl lg:mx-0" {
                    h1 class="text-4xl font-serif font-bold tracking-tight text-white sm:text-6xl" { "Precision Clinical Mapping" }
                    p class="mt-6 text-lg leading-8 text-gray-300" {
                        "High-performance, open-source FHIR terminology services. Powered by Rust for unmatched speed and accuracy in clinical data standardization."
                    }
                    div class="mt-10 flex items-center gap-x-6" {
                        a href="/map" class="rounded-md bg-gold-500 px-3.5 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-gold-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-gold-500 transition-colors" { "Launch Workbench" }
                        a href="https://github.com/refractive-swan" target="_blank" class="text-sm font-semibold leading-6 text-white" { "View on GitHub <span aria-hidden=\"true\">→</span>" }
                    }
                }
            }
        }

        // Feature Grid
        div class="mx-auto max-w-7xl px-6 lg:px-8 py-24 sm:py-32" {
            div class="mx-auto max-w-2xl lg:text-center" {
                h2 class="text-base font-semibold leading-7 text-gold-600" { "Why Refractive Swan?" }
                p class="mt-2 text-3xl font-bold tracking-tight text-navy-900 sm:text-4xl font-serif" { "Built for Modern Healthcare Data" }
            }
            div class="mx-auto mt-16 max-w-2xl sm:mt-20 lg:mt-24 lg:max-w-none" {
                dl class="grid max-w-xl grid-cols-1 gap-x-8 gap-y-16 lg:max-w-none lg:grid-cols-3" {
                    div class="flex flex-col" {
                        dt class="flex items-center gap-x-3 text-base font-semibold leading-7 text-navy-900" {
                            // Bolt Icon
                            svg class="h-5 w-5 flex-none text-gold-600" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" {
                                path stroke-linecap="round" stroke-linejoin="round" d="M3.75 13.5l10.5-11.25L12 10.5h8.25L9.75 21.75 12 13.5H3.75z" {}
                            }
                            "Blazing Fast"
                        }
                        dd class="mt-4 flex flex-auto flex-col text-base leading-7 text-gray-600" {
                            p class="flex-auto" { "Engineered in Rust for millisecond-latency vector search and terminology resolution." }
                        }
                    }
                    div class="flex flex-col" {
                        dt class="flex items-center gap-x-3 text-base font-semibold leading-7 text-navy-900" {
                            // Scale Icon
                            svg class="h-5 w-5 flex-none text-gold-600" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" {
                                path stroke-linecap="round" stroke-linejoin="round" d="M12 3v17.25m0 0c-1.472 0-2.882.265-4.185.75M12 20.25c1.472 0 2.882.265 4.185.75M18.75 4.97A48.416 48.416 0 0012 4.5c-2.291 0-4.545.16-6.75.47m13.5 0c1.01.143 2.01.317 3 .52m-3-.52l2.62 10.726c.122.499-.106 1.028-.589 1.202a5.988 5.988 0 01-2.031.352 5.988 5.988 0 01-2.031-.352c-.483-.174-.711-.703-.59-1.202L18.75 4.971zm-16.5.52c.99-.203 1.99-.377 3-.52m0 0l2.62 10.726c.122.499-.106 1.028-.589 1.202a5.988 5.988 0 01-2.031.352 5.988 5.988 0 01-2.031-.352c-.483-.174-.711-.703-.59-1.202L5.25 4.971z" {}
                            }
                            "Clinically Accurate"
                        }
                        dd class="mt-4 flex flex-auto flex-col text-base leading-7 text-gray-600" {
                            p class="flex-auto" { "Validated against NCIt ontologies with transparent matching logic and confidence scoring." }
                        }
                    }
                    div class="flex flex-col" {
                        dt class="flex items-center gap-x-3 text-base font-semibold leading-7 text-navy-900" {
                            // Open Source Icon (Code brackets)
                            svg class="h-5 w-5 flex-none text-gold-600" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" {
                                path stroke-linecap="round" stroke-linejoin="round" d="M17.25 6.75L22.5 12l-5.25 5.25m-10.5 0L1.5 12l5.25-5.25m7.5-3l-4.5 18" {}
                            }
                            "Open Source"
                        }
                        dd class="mt-4 flex flex-auto flex-col text-base leading-7 text-gray-600" {
                            p class="flex-auto" { "Fully open source. Inspect the code, contribute to the community, and deploy anywhere." }
                        }
                    }
                }
            }
        }
    }).into_string()
}

pub fn render_workbench_page(ctx: &PageContext) -> String {
    base_layout(page_container(html! {
        // Hero / Intro Section
        (card(card_body(html! {
            div class="flex flex-col md:flex-row md:items-start md:justify-between gap-4" {
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

                div class="flex flex-col items-end gap-2" {
                    @if let Some(health) = &ctx.health {
                        (status_badge(health.ok, &health.status))
                    } @else {
                        (status_badge(false, "Unknown"))
                    }

                    @if let Some(metrics) = &ctx.metrics {
                        div class="text-xs text-slate-500 font-mono text-right" {
                            div { (format!("Bundles: {}", metrics.bundle_count)) }
                            div { (format!("Mapped: {}", metrics.auto_mapped)) }
                        }
                    }
                }
            }

            @if let Some(error) = &ctx.health_error {
                div class="mt-4" {
                    (alert(&AlertMessage { kind: AlertKind::Error, text: format!("System Warning: {}", error) }))
                }
            }
        })))

        // Input Section
        (grid_section(html! {
            (card(html! {
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
            }))

            (card(html! {
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
            }))
        }))

        // Results Section
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

        // Metrics Dashboard (Compact)
        @if let Some(metrics) = &ctx.metrics {
            (render_metrics_dashboard(Some(metrics)))
        }
        (render_eval_panel(ctx))
    })).into_string()
}

pub fn render_analytics_page(ctx: &PageContext) -> String {
    base_layout(page_container(html! {
        (section_heading("Analytics Dashboard"))
        (render_analytics_panels(ctx))
    }))
    .into_string()
}

/// HTMX fragment returned to `/map/paste` and `/map/upload` handlers.
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
            div class="bg-white rounded-md border border-dashed border-gray-300 p-12 text-center" {
                h3 class="mt-2 text-sm font-medium text-gray-900" { "No results generated" }
                p class="mt-1 text-sm text-gray-500" { "Submit a Bundle to see mapping analysis." }
            }
        }
    }
}

fn render_metrics_dashboard(metrics: Option<&PipelineMetrics>) -> Markup {
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
                            (metric_card("Bundles Processed", metrics.bundle_count, "Total runs", "text-navy-900"))
                            (metric_card("Flattened Rows", metrics.flats_count, "SR flats emitted", "text-navy-900"))
                            (metric_card("Mapping Attempts", metrics.mapping_count, "Total results", "text-navy-900"))
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

fn render_analytics_panels(ctx: &PageContext) -> Markup {
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

fn render_eval_panel(ctx: &PageContext) -> Markup {
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
                            (PreEscaped(html))
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

fn render_no_match_explorer(results: Option<&MappingResultsView>) -> Markup {
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

pub fn render_eval_page(ctx: &PageContext) -> String {
    base_layout(page_container(html! {
        (render_eval_section(ctx))
    }))
    .into_string()
}

pub fn render_eval_fragment(run: &crate::client::EvalRunResponse) -> String {
    render_eval_summary(&run.summary, &run.dataset).into_string()
}

fn render_eval_section(ctx: &PageContext) -> Markup {
    html! {
        (card(html! {
            (card_header("Evaluation", Some(html! {
                span class="text-sm text-gray-500" { "DFPS mapping eval datasets" }
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
                (metric_card("Precision", (summary.precision * 100.0) as usize, "%", "text-emerald-700"))
                (metric_card("Recall", (summary.recall * 100.0) as usize, "%", "text-emerald-700"))
                (metric_card("Coverage", (summary.coverage * 100.0) as usize, "%", "text-emerald-700"))
            }
            div class="grid gap-4 md:grid-cols-3" {
                (metric_card("Top1 accuracy", (summary.top1_accuracy * 100.0) as usize, "%", "text-navy-700"))
                (metric_card("Top3 accuracy", (summary.top3_accuracy * 100.0) as usize, "%", "text-navy-700"))
                (metric_card("AutoMapped precision", (summary.auto_mapped_precision * 100.0) as usize, "%", "text-navy-700"))
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

fn render_alert(alert: &AlertMessage) -> Markup {
    crate::components::badge::alert(alert)
}

fn render_results_panel(results: &MappingResultsView) -> Markup {
    html! {
        (card(html! {
            (card_header("Mapping Results", Some(html! {
                span class="text-xs font-mono text-gray-500" { (format!("Total: {}", results.request_summary.total)) }
            })))

            div class="p-4 grid gap-3 md:grid-cols-3 border-b border-gray-100" {
                div class="rounded-md border border-gray-200 p-3" {
                    p class="text-xs font-mono text-gray-500 uppercase" { "stg_servicerequest_flat" }
                    p class="text-xl font-serif font-bold mt-1" { (results.request_summary.total) }
                }
                div class="rounded-md border border-gray-200 p-3" {
                    p class="text-xs font-semibold text-gray-600 uppercase" { "SR Status" }
                    @if results.request_summary.statuses.is_empty() {
                        p class="text-sm text-gray-500 mt-1" { "No status values" }
                    } @else {
                        ul class="mt-2 space-y-1 text-xs" {
                            @for stat in &results.request_summary.statuses {
                                li class="flex justify-between" {
                                    span { (&stat.label) }
                                    span class="font-medium" { (&stat.count) }
                                }
                            }
                        }
                    }
                }
                div class="rounded-md border border-gray-200 p-3" {
                    p class="text-xs font-semibold text-gray-600 uppercase" { "SR Intent" }
                    @if results.request_summary.intents.is_empty() {
                        p class="text-sm text-gray-500 mt-1" { "No intents" }
                    } @else {
                        ul class="mt-2 space-y-1 text-xs" {
                            @for stat in &results.request_summary.intents {
                                li class="flex justify-between" {
                                    span { (&stat.label) }
                                    span class="font-medium" { (&stat.count) }
                                }
                            }
                        }
                    }
                }
            }

            div class="overflow-x-auto" {
                table class="min-w-full divide-y divide-gray-200 text-sm" {
                    (table_header(&["ServiceRequest", "Code Element", "NCIt Concept", "State"]))
                    tbody class="bg-white divide-y divide-gray-200" {
                        @if results.rows.is_empty() {
                            tr {
                                td colspan="4" class="px-6 py-8 text-center text-gray-500 italic" {
                                    "No mapping rows generated."
                                }
                            }
                        } @else {
                            @for row in &results.rows {
                                (table_row(html! {
                                    (table_cell(html! {
                                        div class="font-mono text-xs font-medium text-navy-900" { (&row.sr_id) }
                                        div class="text-xs text-gray-500" { (&row.system) }
                                    }))
                                    (table_cell(html! {
                                        div class="font-mono text-xs font-semibold" { (&row.code) }
                                        div class="text-xs text-gray-500" { (&row.display) }
                                    }))
                                    (table_cell(html! {
                                        @if let Some(ncit_id) = &row.ncit_id {
                                            div class="font-mono text-xs font-medium text-navy-900" { (ncit_id) }
                                            @if let Some(label) = &row.ncit_label {
                                                div class="text-xs text-gray-600" { (label) }
                                            }
                                        } @else {
                                            span class="text-gray-400 italic" { "-" }
                                        }
                                    }))
                                    (table_cell(html! {
                                        (state_chip(match row.state {
                                            MappingState::AutoMapped => "auto_mapped",
                                            MappingState::NeedsReview => "needs_review",
                                            MappingState::NoMatch => "no_match",
                                        }))
                                        @if let Some(reason) = &row.reason {
                                            div class="mt-1 text-xs text-rose-600 font-medium" { (reason) }
                                        }
                                    }))
                                }))
                            }
                        }
                    }
                }
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_model::{
        AnalyticsConceptTile, AnalyticsSummaryView, CohortRowView, CohortView, CountStat,
        MappingResultsView, MappingRowView, NoMatchRowView, PageContext, ServiceRequestSummary,
    };
    use dfps_contracts::eval::DatasetManifest;
    use insta::assert_snapshot;

    fn sample_results_view() -> MappingResultsView {
        MappingResultsView {
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
        }
    }

    #[test]
    fn render_workbench_page_shows_metrics_and_no_match_details() {
        let metrics = PipelineMetrics {
            bundle_count: 3,
            flats_count: 4,
            mapping_count: 5,
            auto_mapped: 2,
            needs_review: 1,
            no_match: 2,
            ..PipelineMetrics::default()
        };

        let ctx = PageContext {
            health_error: Some("Health endpoint unreachable: test".into()),
            metrics: Some(metrics),
            results: Some(sample_results_view()),
            eval_report_html: Some("<div>Eval report</div>".into()),
            ..PageContext::default()
        };

        let html = render_workbench_page(&ctx);
        assert!(html.contains("Pipeline Metrics"));
        assert!(html.contains("NoMatch Explorer"));
        assert!(html.contains("missing_system_or_code"));
        assert!(html.contains("System Warning"));
        assert!(html.contains("Evaluation Report"));
        assert!(html.contains("Eval report"));
    }

    #[test]
    fn render_eval_page_shows_dataset_picker_and_metrics() {
        let ctx = PageContext {
            datasets: vec![DatasetManifest {
                name: "pet_ct_small".into(),
                version: "20240601".into(),
                license: Some("test-license".into()),
                source: Some("test-source".into()),
                n_cases: 3,
                sha256: "abc123".into(),
                notes: None,
            }],
            selected_eval_dataset: "pet_ct_small".into(),
            eval: Some(super::super::view_model::EvalContext {
                dataset: "pet_ct_small".into(),
                summary: EvalSummary {
                    total_cases: 3,
                    precision: 0.97,
                    recall: 0.97,
                    coverage: 1.0,
                    top1_accuracy: 0.97,
                    top3_accuracy: 0.97,
                    auto_mapped_precision: 0.98,
                    state_counts: [("auto_mapped".into(), 3)].into_iter().collect(),
                    reason_counts: [("missing_system_or_code".into(), 1)].into_iter().collect(),
                    ..EvalSummary::default()
                },
            }),
            ..PageContext::default()
        };

        let html = render_eval_page(&ctx);
        assert!(html.contains("Dataset: pet_ct_small"));
        assert!(html.contains("Run eval"));
        assert!(html.contains("Top1 accuracy"));
        assert!(html.contains("missing_system_or_code"));
    }

    #[test]
    fn mapping_results_fragment_snapshot() {
        let ctx = PageContext {
            results: Some(sample_results_view()),
            ..PageContext::default()
        };
        assert_snapshot!("mapping_results_fragment", render_results_fragment(&ctx));
    }

    #[test]
    fn no_match_explorer_snapshot() {
        let view = sample_results_view();
        assert_snapshot!(
            "no_match_explorer_fragment",
            render_no_match_explorer(Some(&view)).into_string()
        );
    }

    #[test]
    fn analytics_panels_snapshot() {
        let ctx = PageContext {
            analytics_summary: Some(AnalyticsSummaryView {
                top_concepts: vec![AnalyticsConceptTile {
                    ncit_id: "C1234".into(),
                    preferred_name: "FDG Uptake".into(),
                    total: 3,
                }],
                state_counts: vec![
                    CountStat {
                        label: "auto_mapped".into(),
                        count: 3,
                    },
                    CountStat {
                        label: "needs_review".into(),
                        count: 1,
                    },
                ],
                time_buckets: vec![],
            }),
            cohort: Some(CohortView {
                total: 1,
                rows: vec![CohortRowView {
                    sr_id: "SR-1".into(),
                    patient_id: "P1".into(),
                    encounter_id: "E1".into(),
                    ncit_id: "C1234".into(),
                    description: "FDG".into(),
                    status: "active".into(),
                    intent: "order".into(),
                    ordered_at: "2024-05-01T12:00:00Z".into(),
                    mapping_state: "auto_mapped".into(),
                }],
            }),
            ..PageContext::default()
        };
        assert_snapshot!(
            "analytics_panels_fragment",
            render_analytics_panels(&ctx).into_string()
        );
    }

    #[test]
    fn eval_panel_snapshot() {
        let ctx = PageContext {
            datasets: vec![DatasetManifest {
                name: "gold_pet_ct_small".into(),
                version: "20240501".into(),
                license: Some("test".into()),
                source: Some("demo".into()),
                n_cases: 3,
                sha256: "abc123".into(),
                notes: None,
            }],
            selected_eval_dataset: "gold_pet_ct_small".into(),
            eval_report_html: Some("<div>metrics</div>".into()),
            ..PageContext::default()
        };
        assert_snapshot!("eval_panel_fragment", render_eval_panel(&ctx).into_string());
    }

    #[test]
    fn render_workbench_page_snapshot() {
        let ctx = PageContext::default();
        assert_snapshot!("render_workbench_page_full", render_workbench_page(&ctx));
    }
}
