use maud::{Markup, html};

use crate::views::components::{badge::*, card::*, table::*};
use crate::views::layout::{Breadcrumb, PageCallout, PageShellProps, page_shell};
use crate::views::models::{MeshJobView, PageContext};

pub fn render_mesh_page(ctx: &PageContext) -> String {
    page_shell(PageShellProps {
        title: "Mesh readiness",
        chrome: &ctx.chrome,
        content: html! {
            (render_mesh_summary(ctx))
            (render_mesh_jobs(&ctx.mesh_jobs))
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

fn render_mesh_summary(ctx: &PageContext) -> Markup {
    card(html! {
        (card_header("Mesh nodes", Some(html! {
            span class="text-xs text-gray-500" { "Future mesh nodes + capabilities" }
        })))
        (card_body(html! {
            @if ctx.mesh_nodes.is_empty() {
                p class="text-sm text-gray-500 italic" { "No mesh nodes reported. This page uses mock data until mesh_node publishes capabilities." }
            } @else {
                (table_container(html! {
                    (table_header(&["Node", "Vector", "Warehouse", "Compliance", "Capacity", "Tags", "Status"]))
                    tbody class="bg-white divide-y divide-gray-200 text-sm" {
                        @for node in &ctx.mesh_nodes {
                            (table_row(html! {
                                (table_cell(html! { (&node.id) }))
                                (table_cell(html! { (&node.vector_backend) }))
                                (table_cell(html! { (&node.warehouse_backend) }))
                                (table_cell(html! { (&node.compliance_mode) }))
                                (table_cell_mono(html! { (node.max_dataset_size) }))
                                (table_cell(html! {
                                    span class="text-xs text-gray-600" { (node.tags.join(", ")) }
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
