use dfps_contracts::{PipelineMetrics, eval::EvalSummary, pipeline::MappingState};
use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::view_model::{AlertKind, AlertMessage, CohortView, MappingResultsView, PageContext};

pub fn render_page(ctx: &PageContext) -> String {
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
                            a href="/" class="flex items-center gap-1.5 text-gray-300 hover:text-white hover:underline underline-offset-4 transition-all duration-200" {
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

                main class="mx-auto max-w-7xl px-6 lg:px-8 py-10 space-y-10" {
                    // Hero / Intro Section
                    section class="bg-white shadow-academic rounded-md border border-gray-200 p-6" {
                        div class="flex flex-col md:flex-row md:items-start md:justify-between gap-4" {
                            div class="space-y-3 max-w-3xl" {
                                h2 class="text-3xl font-serif font-bold text-navy-900 leading-tight" { "Clinical Mapping Pipeline" }
                                p class="text-slate-600 leading-relaxed" {
                                    "Ingest FHIR Bundles to flatten ServiceRequests into "
                                    code class="font-mono text-xs bg-navy-50 px-1 py-0.5 rounded text-navy-800" { "stg_servicerequest_flat" }
                                    " and "
                                    code class="font-mono text-xs bg-navy-50 px-1 py-0.5 rounded text-navy-800" { "stg_sr_code_exploded" }
                                    ". The engine emits "
                                    code class="font-mono text-xs bg-navy-50 px-1 py-0.5 rounded text-navy-800" { "MappingResult" }
                                    " rows cross-referenced against NCIt concepts."
                                }
                            }

                            div class="flex flex-col items-end gap-2" {
                                @if let Some(health) = &ctx.health {
                                    div class={(format!("inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-medium border {}",
                                        if health.ok { "bg-emerald-50 text-emerald-800 border-emerald-200" } else { "bg-amber-50 text-amber-800 border-amber-200" }
                                    ))} {
                                        span class={(if health.ok { "h-1.5 w-1.5 rounded-full bg-emerald-600" } else { "h-1.5 w-1.5 rounded-full bg-amber-600" })} {}
                                        span { (format!("System Status: {}", health.status)) }
                                    }
                                } @else {
                                    span class="inline-flex items-center px-3 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-600 border border-gray-200" {
                                        "System Status: Unknown"
                                    }
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
                            div class="mt-4 rounded-md border-l-4 border-rose-600 bg-rose-50 px-4 py-3 text-sm text-rose-900 shadow-sm" {
                                strong class="font-bold font-serif" { "System Warning: " }
                                span { (error) }
                            }
                        }
                    }

                    // Input Section
                    section class="grid gap-6 lg:grid-cols-2" {
                        div class="bg-white shadow-academic rounded-md border border-gray-200 flex flex-col" {
                            div class="bg-navy-50 px-6 py-3 border-b border-gray-200" {
                                h3 class="text-sm font-bold text-navy-900 uppercase tracking-wider" { "Input: Paste JSON" }
                            }
                            div class="p-6 flex-1" {
                                form hx-post="/map/paste" hx-target="#results" hx-swap="innerHTML" method="post" class="h-full flex flex-col space-y-4" {
                                    label class="block" for="bundle_text" {
                                        span class="text-xs font-semibold text-gray-700 uppercase tracking-wide" { "JSON Payload" }
                                        span class="text-xs text-gray-500 ml-2" { "(Paste FHIR Bundle)" }
                                    }
                                    textarea id="bundle_text" name="bundle_text" rows="8" placeholder="{\"resourceType\": \"Bundle\", \"type\": \"collection\", ...}" class="w-full rounded-md border-2 border-gray-300 p-3 font-mono text-xs focus:border-navy-900 focus:ring-2 focus:ring-navy-900 focus:ring-offset-2 transition-all" {}
                                    div class="flex justify-end" {
                                        button type="submit" class="inline-flex items-center gap-2 rounded-md bg-navy-900 px-6 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-navy-800 hover:scale-[1.02] active:scale-[0.98] focus:ring-2 focus:ring-navy-900 focus:ring-offset-2 transition-all duration-200" {
                                        // Paper airplane icon (Heroicons: paper-airplane)
                                        svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 12L3.269 3.126A59.768 59.768 0 0121.485 12 59.77 59.77 0 013.27 20.876L5.999 12zm0 0h7.5" {}
                                        }
                                        span { "Submit & Map" }
                                        // Loading spinner
                                        svg class="htmx-indicator w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24" {
                                            circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" {}
                                            path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" {}
                                        }
                                        }
                                    }
                                }
                            }
                        }
                        div class="bg-white shadow-academic rounded-md border border-gray-200 flex flex-col" {
                            div class="bg-navy-50 px-6 py-3 border-b border-gray-200" {
                                h3 class="text-sm font-bold text-navy-900 uppercase tracking-wider" { "Input: Upload File" }
                            }
                            div class="p-6 flex-1" {
                                form hx-post="/map/upload" hx-target="#results" hx-swap="innerHTML" method="post" enctype="multipart/form-data" class="space-y-4" {
                                    label class="block" {
                                        span class="text-xs font-semibold text-gray-700 uppercase tracking-wide" { "JSON File" }
                                        span class="text-xs text-gray-500 ml-2" { "(Bundle or NDJSON)" }
                                    }
                                    input type="file" id="bundle_file" name="bundle_file" accept="application/json,.json,.ndjson" class="w-full rounded-md border-2 border-gray-300 p-3 text-sm focus:border-navy-900 focus:ring-2 focus:ring-navy-900 focus:ring-offset-2 transition-all" {}
                                    div class="flex justify-end" {
                                        button type="submit" class="inline-flex items-center gap-2 rounded-md bg-navy-900 px-6 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-navy-800 hover:scale-[1.02] active:scale-[0.98] focus:ring-2 focus:ring-navy-900 focus:ring-offset-2 transition-all duration-200" {
                                        // Upload icon (Heroicons: arrow-up-tray)
                                        svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5m-13.5-9L12 3m0 0l4.5 4.5M12 3v13.5" {}
                                        }
                                        span { "Upload & Map" }
                                        // Loading spinner
                                        svg class="htmx-indicator w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24" {
                                            circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" {}
                                            path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" {}
                                        }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    section id="results" class="space-y-6" {
                        (render_results(ctx))
                    }

                    (render_metrics_dashboard(ctx.metrics.as_ref()))
                    (render_analytics_panels(ctx))
                    (render_eval_panel(ctx))
                    (render_no_match_explorer(ctx.results.as_ref()))
                }

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
        section class="bg-white shadow-academic rounded-md border border-gray-200 overflow-hidden" id="metrics-dashboard" {
            div class="bg-navy-50 px-6 py-4 border-b border-gray-200 flex items-center justify-between" {
                h2 class="text-lg font-serif font-bold text-navy-900" { "Pipeline Metrics" }
                span class="text-xs font-mono text-gray-500" {
                    @if let Some(mode) = metrics.and_then(|m| m.compliance_mode.as_deref()) {
                        (format!("Compliance Mode: {}", mode))
                    } @else {
                        "Live Snapshot"
                    }
                }
            }

            @if let Some(metrics) = metrics {
                div class="p-6 space-y-6" {
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
            } @else {
                div class="p-6 text-center text-sm text-gray-500 italic" {
                    "Metrics will populate after the first mapping run."
                }
            }
        }
    }
}

fn render_analytics_panels(ctx: &PageContext) -> Markup {
    html! {
        section class="grid gap-6 lg:grid-cols-2" id="analytics-overview" {
            div class="bg-white shadow-academic rounded-md border border-gray-200 flex flex-col" {
                div class="bg-navy-50 px-6 py-3 border-b border-gray-200" {
                    h2 class="text-sm font-bold text-navy-900 uppercase tracking-wider" { "Analytics Summary" }
                }
                div class="p-6 flex-1 space-y-6" {
                    @if let Some(error) = &ctx.analytics_error {
                        (render_alert(&AlertMessage { kind: AlertKind::Error, text: error.clone() }))
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
            }

            div class="bg-white shadow-academic rounded-md border border-gray-200 flex flex-col" {
                div class="bg-navy-50 px-6 py-3 border-b border-gray-200" {
                    h2 class="text-sm font-bold text-navy-900 uppercase tracking-wider" { "Cohort Explorer" }
                }
                div class="p-6 flex-1 space-y-4" {
                    form method="get" action="/analytics" class="grid gap-4 md:grid-cols-2 text-sm" {
                        label class="flex flex-col gap-1" {
                            span class="text-xs font-semibold text-gray-600 uppercase" { "NCIt ID" }
                            input type="text" name="ncit_id" value=(ctx.cohort_filters.ncit_id.clone().unwrap_or_default()) placeholder="CXXXX" class="rounded-md border-gray-300 px-3 py-1.5 text-sm focus:border-navy-900 focus:ring-1 focus:ring-navy-900" {}
                        }
                        label class="flex flex-col gap-1" {
                            span class="text-xs font-semibold text-gray-600 uppercase" { "Status" }
                            input type="text" name="status" value=(ctx.cohort_filters.status.clone().unwrap_or_default()) placeholder="active" class="rounded-md border-gray-300 px-3 py-1.5 text-sm focus:border-navy-900 focus:ring-1 focus:ring-navy-900" {}
                        }
                        label class="flex flex-col gap-1" {
                            span class="text-xs font-semibold text-gray-600 uppercase" { "Date From" }
                            input type="text" name="date_from" value=(ctx.cohort_filters.date_from.clone().unwrap_or_default()) placeholder="YYYY-MM-DD" class="rounded-md border-gray-300 px-3 py-1.5 text-sm focus:border-navy-900 focus:ring-1 focus:ring-navy-900" {}
                        }
                        label class="flex flex-col gap-1" {
                            span class="text-xs font-semibold text-gray-600 uppercase" { "Date To" }
                            input type="text" name="date_to" value=(ctx.cohort_filters.date_to.clone().unwrap_or_default()) placeholder="YYYY-MM-DD" class="rounded-md border-gray-300 px-3 py-1.5 text-sm focus:border-navy-900 focus:ring-1 focus:ring-navy-900" {}
                        }
                        div class="md:col-span-2 flex justify-end" {
                            button type="submit" class="inline-flex items-center rounded-md bg-white border border-gray-300 px-4 py-1.5 text-sm font-medium text-navy-900 hover:bg-gray-50" {
                                "Apply Filters"
                            }
                        }
                    }

                    @if let Some(error) = &ctx.cohort_error {
                        (render_alert(&AlertMessage { kind: AlertKind::Error, text: error.clone() }))
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
            }
        }
    }
}

fn render_top_concepts(concepts: &[crate::view_model::AnalyticsConceptTile]) -> Markup {
    html! {
        div class="space-y-3" {
            h3 class="text-xs font-bold text-gray-500 uppercase tracking-wide" { "Top Concepts" }
            @if concepts.is_empty() {
                p class="text-sm text-gray-500" { "No concepts recorded." }
            } @else {
                div class="grid gap-2" {
                    @for concept in concepts {
                        div class="flex items-center justify-between p-2 rounded bg-gray-50 border border-gray-100" {
                            div {
                                p class="text-sm font-medium text-navy-900" { (concept.preferred_name.clone()) }
                                p class="text-xs font-mono text-gray-500" { (concept.ncit_id.clone()) }
                            }
                            span class="text-xs font-bold bg-white px-2 py-1 rounded border border-gray-200" { (concept.total) }
                        }
                    }
                }
            }
        }
    }
}

fn render_state_distribution(states: &[crate::view_model::CountStat]) -> Markup {
    html! {
        div class="space-y-3" {
            h3 class="text-xs font-bold text-gray-500 uppercase tracking-wide" { "State Distribution" }
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
            h3 class="text-xs font-bold text-gray-500 uppercase tracking-wide" { "Time Buckets" }
            @if buckets.is_empty() {
                p class="text-sm text-gray-500" { "No ordered_at timestamps available." }
            } @else {
                div class="overflow-x-auto border border-gray-200 rounded-md" {
                    table class="min-w-full text-xs" {
                        thead class="bg-gray-50" {
                            tr {
                                th class="px-3 py-2 text-left font-bold text-gray-500" { "Date" }
                                th class="px-3 py-2 text-left font-bold text-gray-500" { "States" }
                            }
                        }
                        tbody class="bg-white divide-y divide-gray-200" {
                            @for bucket in buckets {
                                tr {
                                    td class="px-3 py-2 font-mono" { (bucket.bucket.clone()) }
                                    td class="px-3 py-2 space-x-2" {
                                        @for stat in &bucket.state_counts {
                                            span class="inline-flex items-center rounded bg-gray-100 px-2 py-0.5 text-xs" {
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
}

fn render_cohort_table(cohort: &CohortView) -> Markup {
    html! {
        div class="overflow-x-auto border border-gray-200 rounded-md" {
            table class="min-w-full divide-y divide-gray-200 text-sm" {
                thead class="bg-gray-50" {
                    tr {
                        th class="px-3 py-2 text-left text-xs font-bold text-gray-500 uppercase tracking-wider" { "SR ID" }
                        th class="px-3 py-2 text-left text-xs font-bold text-gray-500 uppercase tracking-wider" { "Patient" }
                        th class="px-3 py-2 text-left text-xs font-bold text-gray-500 uppercase tracking-wider" { "Encounter" }
                        th class="px-3 py-2 text-left text-xs font-bold text-gray-500 uppercase tracking-wider" { "NCIt" }
                        th class="px-3 py-2 text-left text-xs font-bold text-gray-500 uppercase tracking-wider" { "Status" }
                        th class="px-3 py-2 text-left text-xs font-bold text-gray-500 uppercase tracking-wider" { "Intent" }
                        th class="px-3 py-2 text-left text-xs font-bold text-gray-500 uppercase tracking-wider" { "Ordered At" }
                        th class="px-3 py-2 text-left text-xs font-bold text-gray-500 uppercase tracking-wider" { "State" }
                    }
                }
                tbody class="bg-white divide-y divide-gray-200" {
                    @for row in &cohort.rows {
                        tr class="hover:bg-gray-50 hover:border-l-4 hover:border-l-navy-900 transition-all duration-150 cursor-pointer" {
                            td class="px-3 py-2 font-mono text-xs text-navy-900" { (row.sr_id.clone()) }
                            td class="px-3 py-2 text-xs text-gray-600" { (row.patient_id.clone()) }
                            td class="px-3 py-2 text-xs text-gray-600" { (row.encounter_id.clone()) }
                            td class="px-3 py-2 font-mono text-xs text-gray-600" { (row.ncit_id.clone()) }
                            td class="px-3 py-2 text-xs text-gray-600" { (row.status.clone()) }
                            td class="px-3 py-2 text-xs text-gray-600" { (row.intent.clone()) }
                            td class="px-3 py-2 font-mono text-xs text-gray-600" { (row.ordered_at.clone()) }
                            td class="px-3 py-2" { (state_chip_compact(row.mapping_state.clone())) }
                        }
                    }
                }
            }
        }
    }
}

fn render_eval_panel(ctx: &PageContext) -> Markup {
    html! {
        section class="bg-white shadow-academic rounded-md border border-gray-200" id="eval-panel" {
            div class="bg-navy-50 px-6 py-3 border-b border-gray-200 flex items-center justify-between" {
                h2 class="text-sm font-bold text-navy-900 uppercase tracking-wider" { "Evaluation Report" }
                span class="text-xs text-gray-500" { "Gold Standard Comparison" }
            }
            div class="p-6 space-y-4" {
                div class="flex flex-wrap items-center gap-3 text-sm" {
                    label class="text-xs font-semibold text-gray-600 uppercase" for="eval-dataset" { "Dataset" }
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
        }
    }
}

fn render_no_match_explorer(results: Option<&MappingResultsView>) -> Markup {
    html! {
        section class="bg-white shadow-academic rounded-md border border-gray-200" id="no-match-explorer" {
            div class="bg-navy-50 px-6 py-3 border-b border-gray-200" {
                h2 class="text-sm font-bold text-navy-900 uppercase tracking-wider" { "NoMatch Explorer" }
            }
            div class="p-6" {
                @if let Some(view) = results {
                    @if view.no_matches.is_empty() {
                        p class="text-sm text-emerald-700 font-medium" { "✓ No unmapped codes found in this run." }
                    } @else {
                        div class="overflow-x-auto border border-gray-200 rounded-md" {
                            table class="min-w-full divide-y divide-gray-200 text-sm" {
                                thead class="bg-gray-50" {
                                    tr {
                                        th class="px-4 py-2 text-left text-xs font-bold text-gray-500 uppercase tracking-wider" { "ServiceRequest" }
                                        th class="px-4 py-2 text-left text-xs font-bold text-gray-500 uppercase tracking-wider" { "Code" }
                                        th class="px-4 py-2 text-left text-xs font-bold text-gray-500 uppercase tracking-wider" { "Reason" }
                                    }
                                }
                                tbody class="bg-white divide-y divide-gray-200" {
                                    @for row in &view.no_matches {
                                        tr {
                                            td class="px-4 py-2 align-top" {
                                                p class="font-mono text-xs font-medium" { (&row.sr_id) }
                                                p class="text-xs text-gray-500" { (&row.system) }
                                            }
                                            td class="px-4 py-2 align-top" {
                                                p class="font-mono text-xs font-semibold" { (&row.code) }
                                                p class="text-xs text-gray-500" { (&row.display) }
                                            }
                                            td class="px-4 py-2 align-top" {
                                                span class="inline-flex rounded bg-rose-50 px-2 py-0.5 text-xs font-medium text-rose-800 border border-rose-100" {
                                                    (row.reason.as_deref().unwrap_or("unknown"))
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
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
            }
        }
    }
}

pub fn render_eval_page(ctx: &PageContext) -> String {
    html! {
        (DOCTYPE)
        html class="h-full bg-paper" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "DFPS Eval" }

                link rel="preconnect" href="https://fonts.googleapis.com";
                link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="";
                link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600&family=Merriweather:ital,wght@0,300;0,400;0,700;1,300;1,400&family=Roboto+Mono:wght@400;500&display=swap" rel="stylesheet";

                script src="https://cdn.tailwindcss.com" {}
                script src="https://unpkg.com/htmx.org@1.9.12" {}

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
                                            800: '#1e293b',
                                            900: '#0f172a',
                                        },
                                        gold: {
                                            100: '#fbf3db',
                                            500: '#b49b57',
                                            600: '#967d3f',
                                        },
                                        paper: '#f8f9fa',
                                    }
                                }
                            }
                        }
                    "#))
                }
            }
            body class="min-h-screen bg-paper text-navy-900 font-sans" {
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
        section class="bg-white shadow-academic rounded-md border border-gray-200 p-6 space-y-4" {
            div class="flex items-center justify-between" {
                h2 class="text-xl font-serif font-bold text-navy-900" { "Evaluation" }
                span class="text-sm text-gray-500" { "DFPS mapping eval datasets" }
            }
            form hx-post="/eval/run" hx-target="#eval-fragment" hx-swap="innerHTML" class="flex flex-wrap gap-3 items-center text-sm" {
                label for="dataset" { "Dataset" }
                select id="dataset" name="dataset" class="rounded-md border-gray-300 px-3 py-1.5 text-sm" {
                    @for ds in &ctx.datasets {
                        option value=(ds.name) selected[(ctx.selected_eval_dataset == ds.name)] { (format!("{} ({} rows)", ds.name, ds.n_cases)) }
                    }
                }
                label for="top_k" { "Top K" }
                input type="number" id="top_k" name="top_k" value="1" min="1" max="5" class="w-16 rounded-md border-gray-300 px-2 py-1 text-sm" {}
                button type="submit" class="inline-flex items-center rounded-md bg-navy-900 px-4 py-1.5 text-white font-medium hover:bg-navy-800" { "Run eval" }
            }
            div id="eval-fragment" {
                @if let Some(eval) = &ctx.eval {
                    (render_eval_summary(&eval.summary, &eval.dataset))
                } @else {
                    p class="text-sm text-gray-500" { "No eval summary available yet." }
                }
            }
        }
    }
}

fn render_eval_summary(summary: &EvalSummary, dataset: &str) -> Markup {
    html! {
        div class="space-y-6" {
            div class="flex items-center justify-between" {
                h3 class="text-lg font-serif font-semibold text-navy-900" { (format!("Dataset: {}", dataset)) }
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
    let (bg, border, text_color, icon) = match alert.kind {
        AlertKind::Info => ("bg-blue-50", "border-blue-200", "text-blue-800", "ℹ️"),
        AlertKind::Error => ("bg-rose-50", "border-rose-200", "text-rose-800", "⚠️"),
    };
    html! {
        div class={(format!("rounded-md border px-4 py-3 text-sm font-medium flex items-start gap-2 {} {} {}", bg, border, text_color))} {
            span { (icon) }
            span { (&alert.text) }
        }
    }
}

fn render_results_panel(results: &MappingResultsView) -> Markup {
    html! {
        div class="bg-white shadow-academic rounded-md border border-gray-200 overflow-hidden" {
            div class="bg-navy-50 px-6 py-3 border-b border-gray-200 flex items-center justify-between" {
                h2 class="text-sm font-bold text-navy-900 uppercase tracking-wider" { "Mapping Results" }
                span class="text-xs font-mono text-gray-500" { (format!("Total: {}", results.request_summary.total)) }
            }

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
                    thead class="bg-gray-50" {
                        tr {
                            th class="px-6 py-3 text-left text-xs font-bold text-gray-500 uppercase tracking-wider" { "ServiceRequest" }
                            th class="px-6 py-3 text-left text-xs font-bold text-gray-500 uppercase tracking-wider" { "Code Element" }
                            th class="px-6 py-3 text-left text-xs font-bold text-gray-500 uppercase tracking-wider" { "NCIt Concept" }
                            th class="px-6 py-3 text-left text-xs font-bold text-gray-500 uppercase tracking-wider" { "State" }
                        }
                    }
                    tbody class="bg-white divide-y divide-gray-200" {
                        @if results.rows.is_empty() {
                            tr {
                                td colspan="4" class="px-6 py-8 text-center text-gray-500 italic" {
                                    "No mapping rows generated."
                                }
                            }
                        } @else {
                            @for row in &results.rows {
                                tr class="hover:bg-gray-50 hover:border-l-4 hover:border-l-navy-900 transition-all duration-150 cursor-pointer" {
                                    td class="px-6 py-4 align-top" {
                                        div class="font-mono text-xs font-medium text-navy-900" { (&row.sr_id) }
                                        div class="text-xs text-gray-500 mt-0.5" { (&row.system) }
                                    }
                                    td class="px-6 py-4 align-top" {
                                        div class="font-mono text-xs font-bold text-navy-800" { (&row.code) }
                                        div class="text-xs text-gray-600 mt-0.5" { (&row.display) }
                                    }
                                    td class="px-6 py-4 align-top" {
                                        @if let Some(id) = &row.ncit_id {
                                            div class="font-mono text-xs font-medium text-navy-900 bg-gray-100 px-1.5 py-0.5 rounded inline-block" { (id) }
                                            @if let Some(label) = &row.ncit_label {
                                                div class="text-xs text-gray-600 mt-0.5" { (label) }
                                            }
                                        } @else {
                                            span class="text-gray-300" { "—" }
                                        }
                                    }
                                    td class="px-6 py-4 align-top" {
                                        (state_chip(row.state))
                                        @if let Some(reason) = &row.reason {
                                            div class="mt-1 text-xs text-rose-600 font-medium" { (reason) }
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
    let (label, classes) = match state {
        MappingState::AutoMapped => (
            "AutoMapped",
            "bg-emerald-50 text-emerald-800 border-emerald-200",
        ),
        MappingState::NeedsReview => (
            "Needs Review",
            "bg-amber-50 text-amber-800 border-amber-200",
        ),
        MappingState::NoMatch => ("No Match", "bg-rose-50 text-rose-800 border-rose-200"),
    };
    html! {
        span class={(format!("inline-flex items-center rounded px-2.5 py-0.5 text-xs font-medium border {}", classes))} {
            (label)
        }
    }
}

fn state_chip_compact(state: String) -> Markup {
    let classes = match state.as_str() {
        "AutoMapped" => "text-emerald-700 bg-emerald-50",
        "NeedsReview" => "text-amber-700 bg-amber-50",
        "NoMatch" => "text-rose-700 bg-rose-50",
        _ => "text-gray-700 bg-gray-50",
    };
    html! {
        span class={(format!("inline-flex rounded px-2 py-0.5 text-xs font-medium {}", classes))} {
            (state)
        }
    }
}

fn metric_card(title: &str, value: usize, suffix: &str, text_class: &str) -> Markup {
    // Determine if this is a percentage metric
    let is_percentage = suffix == "%";
    let progress_color = if is_percentage {
        if value >= 90 {
            "bg-emerald-500"
        } else if value >= 70 {
            "bg-amber-500"
        } else {
            "bg-rose-500"
        }
    } else {
        "bg-navy-900"
    };

    html! {
        div class="rounded-md border border-gray-200 p-4 bg-white hover:border-gray-300 hover:shadow-md transition-all" {
            p class="text-xs font-bold text-gray-500 uppercase tracking-wide" { (title) }
            div class="mt-2 flex items-baseline gap-1" {
                span class={(format!("text-2xl font-serif font-bold {}", text_class))} { (value) }
                span class="text-sm text-gray-400" { (suffix) }
            }
            @if is_percentage {
                div class="mt-3 w-full bg-gray-200 rounded-full h-2 overflow-hidden" {
                    div class={(format!("h-full rounded-full transition-all duration-500 {}", progress_color))} style={(format!("width: {}%", value))} {}
                }
            }
        }
    }
}

fn state_metric_card(title: &str, value: usize, classes: &str, tooltip: &str) -> Markup {
    html! {
        div class={(format!("rounded-md border p-4 {}", classes))} title=(tooltip) {
            p class="text-xs font-bold opacity-80 uppercase tracking-wide" { (title) }
            p class="mt-1 text-2xl font-serif font-bold" { (value) }
        }
    }
}

fn secondary_metric(title: &str, value: usize, text_class: &str) -> Markup {
    html! {
        div class="text-center" {
            p class="text-xs text-gray-500" { (title) }
            p class={(format!("text-lg font-mono font-bold {}", text_class))} { (value) }
        }
    }
}

fn secondary_metric_avg(title: &str, value: Option<f64>, text_class: &str) -> Markup {
    html! {
        div class="text-center" {
            p class="text-xs text-gray-500" { (title) }
            p class={(format!("text-lg font-mono font-bold {}", text_class))} {
                @if let Some(v) = value {
                    (format!("{:.1}", v))
                } @else {
                    "n/a"
                }
            }
        }
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
    fn render_page_shows_metrics_and_no_match_details() {
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

        let html = render_page(&ctx);
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
    fn render_page_snapshot() {
        let ctx = PageContext::default();
        assert_snapshot!("render_page_full", render_page(&ctx));
    }
}
