use maud::{Markup, html};

use crate::views::layout::{PageCallout, PageShellProps, ViewChrome, page_shell};

pub fn render_landing_page(chrome: &ViewChrome) -> String {
    page_shell(PageShellProps {
        title: "Refractive Swan Mapping Workbench",
        chrome,
        content: html! {
            (render_hero(chrome))
            (render_feature_grid())
        },
        breadcrumbs: Vec::new(),
        callouts: vec![PageCallout::info(
            "HTMX workbench",
            "Every landing page action flows into the same backend routes exposed by the CLI / API."
        )],
    })
    .into_string()
}

fn render_hero(chrome: &ViewChrome) -> Markup {
    let github_url = chrome
        .github_url
        .as_deref()
        .unwrap_or(crate::views::layout::DEFAULT_GITHUB_URL);
    html! {
        div class="relative isolate overflow-hidden bg-navy-900 py-24 sm:py-32 rounded-3xl shadow-academic" {
            div class="mx-auto max-w-7xl px-6 lg:px-8" {
                div class="mx-auto max-w-2xl lg:mx-0" {
                    h1 class="text-4xl font-serif font-bold tracking-tight text-white sm:text-6xl" { "Precision Clinical Mapping" }
                    p class="mt-6 text-lg leading-8 text-gray-300" {
                        "HTMX-powered workbench that mirrors CLI/API workflows for ServiceRequest normalization, mapping, and evaluation."
                    }
                    div class="mt-10 flex items-center gap-x-6" {
                        a href="/map" class="rounded-md bg-gold-500 px-3.5 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-gold-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-gold-500 transition-colors" { "Launch Workbench" }
                        a href=(github_url) target="_blank" rel="noreferrer" class="text-sm font-semibold leading-6 text-white hover:underline underline-offset-4" { "View on GitHub " span aria-hidden="true" { "→" } }
                    }
                }
            }
        }
    }
}

fn render_feature_grid() -> Markup {
    html! {
        div class="mx-auto max-w-7xl px-6 lg:px-8 py-24 sm:py-32" {
            div class="mx-auto max-w-2xl lg:text-center" {
                h2 class="text-base font-semibold leading-7 text-gold-600" { "Why Refractive Swan?" }
                p class="mt-2 text-3xl font-bold tracking-tight text-navy-900 sm:text-4xl font-serif" { "Built for Modern Healthcare Data" }
            }
            div class="mx-auto mt-16 max-w-2xl sm:mt-20 lg:mt-24 lg:max-w-none" {
                dl class="grid max-w-xl grid-cols-1 gap-x-8 gap-y-16 lg:max-w-none lg:grid-cols-3" {
                    (feature("Blazing Fast", "Engineered in Rust for millisecond-latency vector search and terminology resolution.", r#"M3.75 13.5l10.5-11.25L12 10.5h8.25L9.75 21.75 12 13.5H3.75z"#))
                    (feature("Clinically Accurate", "Validated against NCIt ontologies with transparent matching logic and confidence scoring.", r#"M12 3v17.25m0 0c-1.472 0-2.882.265-4.185.75M12 20.25c1.472 0 2.882.265 4.185.75M18.75 4.97A48.416 48.416 0 0012 4.5c-2.291 0-4.545.16-6.75.47m13.5 0c1.01.143 2.01.317 3 .52m-3-.52l2.62 10.726c.122.499-.106 1.028-.589 1.202a5.988 5.988 0 01-2.031.352 5.988 5.988 0 01-2.031-.352c-.483-.174-.711-.703-.59-1.202L18.75 4.971zm-16.5.52c.99-.203 1.99-.377 3-.52m0 0l2.62 10.726c.122.499-.106 1.028-.589 1.202a5.988 5.988 0 01-2.031.352 5.988 5.988 0 01-2.031-.352c-.483-.174-.711-.703-.59-1.202L5.25 4.971z"#))
                    (feature("Open Source", "Fully open source. Inspect the code, contribute to the community, and deploy anywhere.", r#"M17.25 6.75L22.5 12l-5.25 5.25m-10.5 0L1.5 12l5.25-5.25m7.5-3l-4.5 18"#))
                }
            }
        }
    }
}

fn feature(title: &str, description: &str, path: &str) -> Markup {
    html! {
        div class="flex flex-col" {
            dt class="flex items-center gap-x-3 text-base font-semibold leading-7 text-navy-900" {
                svg class="h-5 w-5 flex-none text-gold-600" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" {
                    path stroke-linecap="round" stroke-linejoin="round" d=(path) {}
                }
                (title)
            }
            dd class="mt-4 flex flex-auto flex-col text-base leading-7 text-gray-600" {
                p class="flex-auto" { (description) }
            }
        }
    }
}
