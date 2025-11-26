use maud::{Markup, html};

use crate::views::components::{card::*, table::*};
use crate::views::layout::{Breadcrumb, PageCallout, PageShellProps, page_shell};
use crate::views::models::{EnvironmentView, PageContext};

pub fn render_environment_page(ctx: &PageContext) -> String {
    page_shell(PageShellProps {
        title: "Environment & Diagnostics",
        chrome: &ctx.chrome,
        content: html! {
            (render_env_card(ctx.environment.as_ref()))
            (render_diagnostics(ctx.environment.as_ref()))
        },
        breadcrumbs: vec![
            Breadcrumb::home(),
            Breadcrumb::new("Environment", Some("/environment")),
        ],
        callouts: vec![PageCallout::info(
            "Config surface",
            "Frontend + backend URLs, feature flags, and raw health/metrics JSON for quick debugging.",
        )],
    })
    .into_string()
}

fn render_env_card(env: Option<&EnvironmentView>) -> Markup {
    card(html! {
        (card_header("Runtime config", Some(html! {
            span class="text-xs text-gray-500" { "Loaded via refractive_swan_configuration" }
        })))
        (card_body(html! {
            @if let Some(env) = env {
                div class="grid gap-3 text-sm text-gray-700" {
                    (kv("Frontend listen", &env.frontend_listen_addr))
                    (kv("API base URL", &env.backend_base_url))
                    (kv_opt("Docs URL", env.docs_url.as_ref()))
                    (kv_opt("GitHub URL", env.github_url.as_ref()))
                }
                @if env.feature_flags.is_empty() {
                    p class="mt-3 text-xs text-gray-500" { "No feature flags detected." }
                } @else {
                    (table_container(html! {
                        (table_header(&["Feature flag", "Value"]))
                        tbody class="bg-white divide-y divide-gray-200 text-xs" {
                            @for flag in &env.feature_flags {
                                (table_row(html! {
                                    (table_cell(html! { (&flag.name) }))
                                    (table_cell(html! { (flag.value.as_deref().unwrap_or("unset")) }))
                                }))
                            }
                        }
                    }))
                }
            } @else {
                p class="text-sm text-gray-500 italic" { "Config not available." }
            }
        }))
    })
}

fn render_diagnostics(env: Option<&EnvironmentView>) -> Markup {
    card(html! {
        (card_header("Diagnostics", Some(html! {
            span class="text-xs text-gray-500" { "Raw /health and /metrics/summary" }
        })))
        (card_body(html! {
            @if let Some(env) = env {
                div class="grid gap-3 md:grid-cols-2 text-xs font-mono text-gray-700" {
                    (pre_block("Health", env.diagnostics.health.as_ref().map(|h| serde_json::to_string_pretty(h).unwrap_or_default()).unwrap_or_else(|| "unavailable".into())))
                    (pre_block("Metrics", env.diagnostics.metrics.as_ref().map(|m| serde_json::to_string_pretty(m).unwrap_or_default()).unwrap_or_else(|| "unavailable".into())))
                }
                div class="mt-3 text-xs text-gray-500" {
                    a class="text-sky-700 hover:underline" href="/environment/diagnostics" { "Download diagnostics JSON" }
                }
            } @else {
                p class="text-sm text-gray-500 italic" { "Diagnostics unavailable." }
            }
        }))
    })
}

fn kv(label: &str, value: &str) -> Markup {
    html! {
        div class="flex items-center justify-between rounded-md border border-gray-100 bg-gray-50 px-3 py-2" {
            span class="text-xs text-gray-500" { (label) }
            span class="text-sm font-semibold text-navy-900" { (value) }
        }
    }
}

fn kv_opt(label: &str, value: Option<&String>) -> Markup {
    kv(label, value.map(|v| v.as_str()).unwrap_or("unset"))
}

fn pre_block(label: &str, body: String) -> Markup {
    html! {
        div class="rounded-md border border-gray-100 bg-slate-50 p-3" {
            p class="text-[11px] font-semibold text-gray-600 mb-1" { (label) }
            pre class="text-[11px] whitespace-pre-wrap break-all" { (body) }
        }
    }
}
