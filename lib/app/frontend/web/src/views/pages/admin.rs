use maud::{Markup, html};

use crate::views::components::{badge::alert, button::*, card::*, input::*, table::*};
use crate::views::layout::{Breadcrumb, PageCallout, PageShellProps, page_shell};
use crate::views::models::{
    DatasetAdminEntryView, DatasetAdminView, PageContext, RegressionJobStatusView,
    RegressionJobView,
};

pub fn render_dataset_admin_page(ctx: &PageContext) -> String {
    page_shell(PageShellProps {
        title: "Dataset & Storage Admin",
        chrome: &ctx.chrome,
        content: render_dataset_admin_content(ctx),
        breadcrumbs: vec![
            Breadcrumb::home(),
            Breadcrumb::new("Dataset admin", Some("/admin/datasets")),
        ],
        callouts: vec![PageCallout::info(
            "Local dataset store",
            "Upload/refresh manifests under refractive_swan_eval::DatasetStore and manage regression smoke tests.",
        )],
    })
    .into_string()
}

fn render_dataset_admin_content(ctx: &PageContext) -> Markup {
    html! {
        @if let Some(message) = &ctx.maintenance_message {
            div class="mb-4" {
                (alert(&crate::views::models::AlertMessage {
                    kind: crate::views::models::AlertKind::Info,
                    text: message.clone(),
                }))
            }
        }
        @if let Some(admin) = &ctx.dataset_admin {
            div class="grid gap-6 lg:grid-cols-2" {
                (render_dataset_table(admin))
                (render_upload_form())
                (render_fixture_card(admin))
                (render_datamart_card(admin))
            }
        } @else {
            (card(html! {
                (card_body(html! {
                    p class="text-sm text-gray-500" { "Dataset store not configured." }
                }))
            }))
        }
        (render_regression_card(&ctx.regression_jobs))
        (render_maintenance_card())
    }
}

fn render_dataset_table(admin: &DatasetAdminView) -> Markup {
    card(html! {
        (card_header("Dataset catalog", Some(html! {
            span class="text-xs text-gray-500" { (format!("Root: {}", admin.root)) }
        })))
        (card_body(html! {
            (table_container(html! {
                (table_header(&["Dataset", "Tier", "Cases", "SHA", "Status", "Actions"]))
                tbody class="bg-white divide-y divide-gray-200 text-sm" {
                    @for entry in &admin.entries {
                        (table_row(render_dataset_row(entry)))
                    }
                }
            }))
        }))
    })
}

fn render_dataset_row(entry: &DatasetAdminEntryView) -> Markup {
    html! {
        (table_cell(html! {
            p class="font-semibold text-navy-900" { (&entry.name) }
            p class="text-xs text-gray-500" { (format!("Version {}", entry.version)) }
        }))
        (table_cell(html! { (&entry.tier) }))
        (table_cell_mono(html! { (entry.n_cases) }))
        (table_cell_mono(html! { (&entry.sha256[..std::cmp::min(8, entry.sha256.len())]) }))
        (table_cell(html! {
            @if entry.disabled {
                span class="text-xs font-semibold text-rose-700" { "Disabled" }
            } @else {
                span class="text-xs text-emerald-700 font-semibold" { "Enabled" }
            }
        }))
        (table_cell(html! {
            form method="post" action=(format!("/admin/datasets/{}/toggle", entry.name)) {
                input type="hidden" name="action" value=(if entry.disabled { "enable" } else { "disable" }) {}
                button type="submit" class="text-xs font-semibold text-navy-900 hover:underline" {
                    @if entry.disabled { "Enable" } @else { "Disable" }
                }
            }
        }))
    }
}

fn render_upload_form() -> Markup {
    card(html! {
        (card_header("Upload dataset", Some(html! {
            span class="text-xs text-gray-500" { "Manifest JSON + NDJSON data" }
        })))
        (card_body(html! {
            form method="post" action="/admin/datasets/upload" enctype="multipart/form-data" class="space-y-3 text-sm text-gray-600" {
                (file_upload("manifest", "application/json"))
                (file_upload("dataset", ".ndjson,application/x-ndjson"))
                (primary_button(ButtonProps {
                    text: "Upload",
                    type_: "submit",
                    ..ButtonProps::default()
                }))
            }
        }))
    })
}

fn render_fixture_card(admin: &DatasetAdminView) -> Markup {
    card(html! {
        (card_header("Fixture downloads", None))
        (card_body(html! {
            @if admin.fixtures.is_empty() {
                p class="text-sm text-gray-500 italic" { "No datasets available." }
            } @else {
                ul class="space-y-2 text-sm text-gray-600" {
                    @for fixture in &admin.fixtures {
                        li class="flex items-center justify-between rounded-md border border-gray-100 bg-gray-50 px-3 py-2" {
                            span class="font-semibold text-navy-900" { (&fixture.name) }
                            span class="space-x-3 text-xs" {
                                a class="text-sky-700 hover:underline" href=(&fixture.manifest_href) { "Manifest" }
                                a class="text-sky-700 hover:underline" href=(&fixture.data_href) { "NDJSON" }
                            }
                        }
                    }
                }
            }
        }))
    })
}

fn render_datamart_card(admin: &DatasetAdminView) -> Markup {
    card(html! {
        (card_header("Datamart connection", None))
        (card_body(html! {
            p class="text-sm text-gray-600" {
                (admin.datamart_url.as_deref().unwrap_or("refractive_swan_WAREHOUSE_URL unset"))
            }
            form method="post" action="/admin/datamart/reset" class="mt-3" {
                button
                    type="submit"
                    class="rounded-md border border-navy-200 px-3 py-2 text-xs font-semibold text-navy-900 shadow-sm hover:bg-navy-50 disabled:opacity-50"
                    disabled[!admin.datamart_reset_supported] {
                    "Reset SQLite file"
                }
            }
        }))
    })
}

fn render_regression_card(jobs: &[RegressionJobView]) -> Markup {
    card(html! {
        (card_header("Regression smoke tests", Some(html! {
            span class="text-xs text-gray-500" { "Runs refractive_swan_test_suite::ping()" }
        })))
        (card_body(html! {
            form method="post" action="/admin/regression/run" {
                (primary_button(ButtonProps {
                    text: "Trigger smoke suite",
                    type_: "submit",
                    ..ButtonProps::default()
                }))
            }
            div class="mt-4 space-y-2" {
                @for job in jobs {
                    div class="rounded-md border border-gray-100 bg-gray-50 px-3 py-2 text-xs text-gray-600" {
                        div class="flex items-center justify-between" {
                            span class="font-semibold text-navy-900" { (&job.label) }
                            (regression_badge(job.status))
                        }
                        span { (format!("Started {}", job.submitted_at)) }
                        @if let Some(done) = &job.completed_at {
                            span class="ml-2 text-gray-500" { (format!("· Completed {}", done)) }
                        }
                        @for line in &job.log {
                            div class="text-[11px] text-gray-500" { (&line) }
                        }
                        @if let Some(error) = &job.error {
                            div class="text-rose-700" { (error) }
                        }
                    }
                }
                @if jobs.is_empty() {
                    p class="text-xs text-gray-500 italic" { "No regression jobs yet." }
                }
            }
        }))
    })
}

fn regression_badge(status: RegressionJobStatusView) -> Markup {
    let (label, class) = match status {
        RegressionJobStatusView::Pending => ("pending", "text-slate-600"),
        RegressionJobStatusView::Running => ("running", "text-blue-600"),
        RegressionJobStatusView::Passed => ("passed", "text-emerald-700"),
        RegressionJobStatusView::Failed => ("failed", "text-rose-700"),
    };
    html! { span class=(format!("text-[11px] font-semibold {}", class)) { (label) } }
}

fn render_maintenance_card() -> Markup {
    card(html! {
        (card_header("Maintenance console", None))
        (card_body(html! {
            p class="text-xs text-gray-500" { "Clear HTMX fragment caches (history/logs) or reset datamart files for local development." }
            form method="post" action="/admin/maintenance/clear" class="mt-3" {
                button
                    type="submit"
                    class="rounded-md border border-navy-200 px-3 py-2 text-xs font-semibold text-navy-900 shadow-sm hover:bg-navy-50" {
                    "Clear caches"
                }
            }
        }))
    })
}
