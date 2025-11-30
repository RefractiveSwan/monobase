use maud::{Markup, html};

use crate::views::components::{badge::*, card::*, table::*};
use crate::views::layout::{Breadcrumb, PageCallout, PageShellProps, page_shell};
use crate::views::models::{MeshJobView, PageContext};
use refractive_swan_web_dto::{AnalyticsSummaryResponse, EvalSummary};

pub fn render_mesh_page(ctx: &PageContext) -> String {
    page_shell(PageShellProps {
        title: "Mesh readiness",
        chrome: &ctx.chrome,
        content: html! {
            (render_mesh_alerts(&ctx.mesh_alerts))
            (render_mesh_summary(ctx))
            (render_node_details(ctx))
            (render_mesh_jobs(&ctx.mesh_jobs))
            (render_hub_panels(ctx))
            (render_admin_events(ctx))
            (render_governance(ctx))
        },
        breadcrumbs: vec![
            Breadcrumb::home(),
            Breadcrumb::new("Mesh", Some("/mesh")),
        ],
        callouts: vec![PageCallout::info(
            "Single-node fallback",
            "Workbench runs in single-node mode; mesh API hooks are stubbed until mesh_node endpoints ship.",
        )],
    })
    .into_string()
}

pub fn render_hub_analytics_fragment(ctx: &PageContext) -> String {
    render_hub_analytics(ctx.hub_analytics.as_ref(), &ctx.mesh_alerts).into_string()
}

pub fn render_hub_eval_fragment(ctx: &PageContext, dataset: &str) -> String {
    render_hub_eval(ctx.hub_eval.as_ref(), &ctx.mesh_alerts, Some(dataset)).into_string()
}

pub fn render_admin_events_fragment(ctx: &PageContext) -> String {
    render_admin_events(ctx).into_string()
}

fn render_mesh_alerts(alerts: &[String]) -> Markup {
    if alerts.is_empty() {
        return html! {};
    }
    card(html! {
        (card_body(html! {
            @for alert in alerts {
                div class="rounded border border-amber-200 bg-amber-50 px-3 py-2 text-xs text-amber-800 mb-2" {
                    (alert)
                }
            }
        }))
    })
}

fn render_mesh_summary(ctx: &PageContext) -> Markup {
    card(html! {
        (card_header("Mesh nodes", Some(html! {
            span class="text-xs text-gray-500" { "Future mesh nodes + capabilities" }
        })))
        (card_body(html! {
            @if let Some(note) = &ctx.mesh_governance {
                div class="mb-3 rounded border border-amber-200 bg-amber-50 px-3 py-2 text-xs text-amber-800" {
                    (note)
                }
            }
            @if ctx.mesh_nodes.is_empty() {
                p class="text-sm text-gray-500 italic" { "No mesh nodes reported. This page uses mock data until mesh_node publishes capabilities." }
            } @else {
                (table_container(html! {
                    (table_header(&["Node", "Vector", "Warehouse", "Cache", "DP budget", "Compliance", "Capacity", "Tags", "Last seen", "Status"]))
                    tbody class="bg-white divide-y divide-gray-200 text-sm" {
                        @for node in &ctx.mesh_nodes {
                            (table_row(html! {
                                (table_cell(html! { (&node.id) }))
                                (table_cell(html! { (&node.vector_backend) }))
                                (table_cell(html! { (&node.warehouse_backend) }))
                                (table_cell(html! {
                                    span class="inline-flex items-center gap-1" {
                                        span { (node.cache_backend.as_deref().unwrap_or("disabled")) }
                                        @if let Some(status) = &node.cache_status {
                                            span class="text-[11px] text-gray-500" { (format!("({})", status)) }
                                        }
                                    }
                                }))
                                (table_cell(html! {
                                    @match (&node.dp_budget_remaining, &node.dp_budget_status) {
                                        (Some(rem), Some(status)) => {
                                            span class="inline-flex items-center gap-1" {
                                                span { (format!("{rem:.2}")) }
                                                span class="text-[11px] text-gray-500" { (format!("({status})")) }
                                            }
                                        },
                                        (Some(rem), None) => {
                                            span class="text-xs text-gray-700" { (format!("{rem:.2}")) }
                                        },
                                        _ => {
                                            span class="text-xs text-gray-500 italic" { "n/a" }
                                        }
                                    }
                                }))
                                (table_cell(html! { (&node.compliance_mode) }))
                                (table_cell_mono(html! { (node.max_dataset_size) }))
                                (table_cell(html! {
                                    span class="text-xs text-gray-600" { (node.tags.join(", ")) }
                                }))
                                (table_cell(html! {
                                    @if let Some(ts) = node.last_seen_ms {
                                        (format!("{} ms ago", ts))
                                    } @else {
                                        span class="text-xs text-gray-500 italic" { "n/a" }
                                    }
                                }))
                                (table_cell(html! { (status_badge(&node.status)) }))
                            }))
                        }
                    }
                }))
            }
        }))
    })
}

fn render_mesh_jobs(jobs: &[MeshJobView]) -> Markup {
    card(html! {
        (card_header("Mesh job hooks", Some(html! {
            span class="text-xs text-gray-500" { "EvalDataset / AnalyticsQuery stubs" }
        })))
        (card_body(html! {
            p class="text-xs text-gray-600 mb-3" { "Jobs below are placeholders; once the mesh_node API exposes job submission, HTMX forms will target those endpoints." }
            @if jobs.is_empty() {
                p class="text-sm text-gray-500 italic" { "No mock jobs queued." }
            } @else {
                div class="space-y-2" {
                    @for job in jobs {
                        div class="rounded border border-gray-100 bg-gray-50 px-3 py-2 text-xs text-gray-600" {
                            div class="flex items-center justify-between" {
                                span class="font-semibold text-navy-900" { (&job.job_id) }
                                (status_badge(&job.status))
                            }
                            div class="text-[11px] text-gray-500" { (format!("Type: {}", job.job_type)) }
                            div class="text-[11px] text-gray-700" { (&job.summary) }
                        }
                    }
                }
            }
        }))
    })
}

fn render_node_details(ctx: &PageContext) -> Markup {
    if ctx.mesh_nodes.is_empty() {
        return html! {};
    }
    card(html! {
        (card_header("Node details", Some(html! {
            span class="text-xs text-gray-500" { "Metrics and admin signals (per node)" }
        })))
        (card_body(html! {
            div class="grid gap-3 md:grid-cols-2" {
                @for node in &ctx.mesh_nodes {
                    div class="rounded border border-gray-100 bg-white p-3 text-xs text-gray-700 space-y-1" {
                        div class="flex items-center justify-between" {
                            span class="font-semibold text-navy-900" { (&node.id) }
                            (status_badge(&node.status))
                        }
                        div class="text-[11px] text-gray-500" { (format!("Vector: {} | Warehouse: {}", node.vector_backend, node.warehouse_backend)) }
                        div class="text-[11px] text-gray-500" { (format!("Compliance: {}", node.compliance_mode)) }
                        @if let Some(rem) = node.dp_budget_remaining {
                            div class="text-[11px] text-emerald-700" { (format!("DP budget remaining: {rem:.2} {}", node.dp_budget_status.as_deref().unwrap_or(""))) }
                        }
                        @if let Some(metrics) = &node.metrics {
                            div class="grid grid-cols-3 gap-2 text-[11px]" {
                                span { (format!("Bundles: {}", metrics.bundle_count)) }
                                span { (format!("Mappings: {}", metrics.mapping_count)) }
                                span { (format!("Vector queries: {}", metrics.vector_queries)) }
                            }
                        } @else {
                            div class="text-[11px] text-gray-500 italic" { "Metrics unavailable." }
                        }
                    }
                }
            }
        }))
    })
}

fn render_hub_panels(ctx: &PageContext) -> Markup {
    html! {
        div class="grid gap-4 md:grid-cols-2" {
            (render_hub_analytics(ctx.hub_analytics.as_ref(), &ctx.mesh_alerts))
            (render_hub_eval(ctx.hub_eval.as_ref(), &ctx.mesh_alerts, None))
        }
    }
}

fn render_hub_analytics(summary: Option<&AnalyticsSummaryResponse>, alerts: &[String]) -> Markup {
    let max = summary
        .map(|s| s.rows.iter().map(|r| r.count).max().unwrap_or(0))
        .unwrap_or(0)
        .max(1);
    let rows: Vec<_> = summary
        .map(|s| {
            let mut rows = s.rows.clone();
            rows.sort_by_key(|r| std::cmp::Reverse(r.count));
            rows.truncate(5);
            rows
        })
        .unwrap_or_default();
    card(html! {
        (card_header("Hub NCIt summary", Some(html! {
            span class="text-xs text-gray-500" { "Aggregated via /hub/jobs/analytics/ncit-summary" }
        })))
        (card_body(html! {
            div id="hub-analytics-panel" hx-get="/mesh/hub/analytics" hx-trigger="load, every 10s" hx-swap="outerHTML" {
                @if !alerts.is_empty() && summary.is_none() {
                    @for alert in alerts {
                        div class="rounded border border-amber-200 bg-amber-50 px-3 py-2 text-[11px] text-amber-800 mb-2" {
                            (alert)
                        }
                    }
                }
                @if rows.is_empty() {
                    p class="text-xs text-gray-500 italic" { "No hub analytics available yet." }
                } @else {
                    div class="space-y-2" {
                        @for row in rows {
                            @let pct = ((row.count as f32 / max as f32) * 100.0).min(100.0);
                            div class="space-y-1" {
                                div class="flex justify-between text-[11px] text-gray-600" {
                                    span { (row.ncit_id.as_str()) " – " (row.preferred_name.as_deref().unwrap_or("unknown")) }
                                    span class="font-semibold text-navy-900" { (row.count) }
                                }
                                div class="h-2 w-full rounded bg-gray-100 overflow-hidden" {
                                    div class="h-2 bg-sky-500 rounded" style=(format!("width: {pct}%;")) {}
                                }
                            }
                        }
                    }
                }
            }
        }))
    })
}

fn render_hub_eval(eval: Option<&EvalSummary>, alerts: &[String], dataset: Option<&str>) -> Markup {
    let dataset_label = dataset.unwrap_or("bronze_pet_ct_small");
    card(html! {
        (card_header("Hub federated eval", Some(html! {
            span class="text-xs text-gray-500" { "Aggregated via /hub/jobs/eval" }
        })))
        (card_body(html! {
            div id="hub-eval-panel" hx-get={(format!("/mesh/hub/eval?dataset={}", dataset_label))} hx-trigger="load, every 10s" hx-swap="outerHTML" {
                @if !alerts.is_empty() && eval.is_none() {
                    @for alert in alerts {
                        div class="rounded border border-amber-200 bg-amber-50 px-3 py-2 text-[11px] text-amber-800 mb-2" {
                            (alert)
                        }
                    }
                }
                @if let Some(summary) = eval {
                    div class="grid grid-cols-2 gap-3 text-xs text-gray-700" {
                        (metric_card("Total cases", summary.total_cases))
                        (metric_card("Correct", summary.correct))
                        (metric_card("Predicted", summary.predicted_cases))
                        (metric_card("Incorrect", summary.incorrect))
                    }
                    div class="grid grid-cols-3 gap-2 mt-3 text-[11px]" {
                        (score_bar("Precision", summary.precision))
                        (score_bar("Recall", summary.recall))
                        (score_bar("F1", summary.f1))
                    }
                } @else {
                    p class="text-xs text-gray-500 italic" { "No federated eval results yet." }
                }
            }
        }))
    })
}

fn metric_card(label: &str, value: usize) -> Markup {
    html! {
        div class="rounded border border-gray-100 bg-gray-50 px-3 py-2 flex flex-col gap-1" {
            span class="text-[11px] text-gray-500" { (label) }
            span class="text-sm font-semibold text-navy-900" { (value) }
        }
    }
}

fn score_bar(label: &str, score: f32) -> Markup {
    let pct = (score * 100.0).clamp(0.0, 100.0);
    html! {
        div class="space-y-1" {
            div class="flex justify-between" {
                span { (label) }
                span class="font-semibold text-navy-900" { (format!("{pct:.1}%")) }
            }
            div class="h-2 w-full rounded bg-gray-100 overflow-hidden" {
                div class="h-2 bg-emerald-500 rounded" style={(format!("width: {pct}%;"))} {}
            }
        }
    }
}

fn render_admin_events(ctx: &PageContext) -> Markup {
    card(html! {
        (card_header("Recent admin events", Some(html! {
            span class="text-xs text-gray-500" { "Updates via /admin/events" }
        })))
        (card_body(html! {
            div id="mesh-admin-events" hx-get="/mesh/admin-events" hx-trigger="load, every 8s" hx-swap="outerHTML" {
                @if ctx.mesh_admin_events.is_empty() {
                    p class="text-xs text-gray-500 italic" { "No admin events yet." }
                } @else {
                    ul class="space-y-2 text-xs text-gray-700" {
                        @for evt in &ctx.mesh_admin_events {
                            li class="rounded border border-gray-100 bg-gray-50 px-3 py-2" {
                                div class="flex items-center justify-between" {
                                    span class="font-semibold text-navy-900" { (format!("{:?}", evt.kind)) }
                                    span class="text-[11px] text-gray-500" { (evt.timestamp.format("%Y-%m-%d %H:%M:%S")) }
                                }
                                @if let Some(node) = &evt.mesh_node_id {
                                    div class="text-[11px] text-gray-500" { (format!("node: {}", node)) }
                                }
                                div class="text-[11px] text-gray-700" { (&evt.message) }
                            }
                        }
                    }
                }
            }
        }))
    })
}

fn render_governance(ctx: &PageContext) -> Markup {
    card(html! {
        (card_header("Governance & fallback", None))
        (card_body(html! {
            p class="text-sm text-gray-600" {
                (ctx.mesh_governance.as_deref().unwrap_or("Governance previews will surface policy decisions once dfps_mesh_governance is wired."))
            }
            ul class="mt-3 space-y-1 text-xs text-gray-600 list-disc list-inside" {
                li { "EvalDataset / AnalyticsQuery submissions will be routed to mesh_node when available." }
                li { "Vector controls stay node-local; per-node backend selection will mirror the workbench toggle." }
                li { "In single-node mode, this page reflects local capabilities only." }
            }
        }))
    })
}
