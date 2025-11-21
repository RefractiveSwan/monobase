use maud::{Markup, html};

/// Links injected into the navbar from layout chrome (Docs/GitHub, etc.).
#[derive(Clone, Copy)]
pub struct NavbarLinks<'a> {
    pub docs_url: Option<&'a str>,
    pub github_url: Option<&'a str>,
}

/// Top navigation bar shared across workbench pages.
pub fn render_navbar(links: NavbarLinks<'_>) -> Markup {
    html! {
        header class="bg-navy-900 text-white shadow-md" {
            div class="mx-auto max-w-7xl px-6 py-4 flex items-center justify-between" {
                div class="flex items-center gap-3" {
                    (crate::views::components::logo::logo_full())
                }
                nav class="flex items-center gap-6 text-sm font-medium" {
                    (nav_link("/map", "Mapping", r#"M9 6.75V15m6-6v8.25m.503 3.498l4.875-2.437c.381-.19.622-.58.622-1.006V4.82c0-.836-.88-1.38-1.628-1.006l-3.869 1.934c-.317.159-.69.159-1.006 0L9.503 3.252a1.125 1.125 0 00-1.006 0L3.622 5.689C3.24 5.88 3 6.27 3 6.695V19.18c0 .836.88 1.38 1.628 1.006l3.869-1.934c.317-.159.69-.159 1.006 0l4.994 2.497c.317.158.69.158 1.006 0z"#))
                    (nav_link("/analytics", "Analytics", r#"M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v6.75C7.5 20.496 6.996 21 6.375 21h-2.25A1.125 1.125 0 013 19.875v-6.75zM9.75 8.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v11.25c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V8.625zM16.5 4.125c0-.621.504-1.125 1.125-1.125h2.25C20.496 3 21 3.504 21 4.125v15.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V4.125z"#))
                    (nav_link("/eval", "Evaluation", r#"M9.75 3.104v5.714a2.25 2.25 0 01-.659 1.591L5 14.5M9.75 3.104c-.251.023-.501.05-.75.082m.75-.082a24.301 24.301 0 014.5 0m0 0v5.714c0 .597.237 1.17.659 1.591L19.8 15.3M14.25 3.104c.251.023.501.05.75.082M19.8 15.3l-1.57.393A9.065 9.065 0 0112 15a9.065 9.065 0 00-6.23-.693L5 14.5m14.8.8l1.402 1.402c1.232 1.232.65 3.318-1.067 3.611A48.309 48.309 0 0112 21c-2.773 0-5.491-.235-8.135-.687-1.718-.293-2.3-2.379-1.067-3.61L5 14.5"#))
                    (nav_link("/ui/components", "UI Kit", r#"M4 6a2 2 0 012-2h3a2 2 0 012 2v3a2 2 0 01-2 2H6a2 2 0 01-2-2V6zm9 0a2 2 0 012-2h3a2 2 0 012 2v3a2 2 0 01-2 2h-3a2 2 0 01-2-2V6zM4 15a2 2 0 012-2h3a2 2 0 012 2v3a2 2 0 01-2 2H6a2 2 0 01-2-2v-3zm9 0a2 2 0 012-2h3a2 2 0 012 2v3a2 2 0 01-2 2h-3a2 2 0 01-2-2v-3z"#))
                }
                div class="flex items-center gap-3" {
                    @if let Some(url) = links.docs_url {
                        (external_link(url, "Docs", "M12 6v12m6-6H6"))
                    }
                    @if let Some(url) = links.github_url {
                        (external_link(url, "GitHub", "M9 19c-4.418 1.465-4.418-2.589-6-3m12 6v-3.87a3.37 3.37 0 00-.94-2.61c3.14-.35 6.44-1.54 6.44-7a5.44 5.44 0 00-1.5-3.75 5.07 5.07 0 00-.09-3.77S18.73.38 16 2.52a13.38 13.38 0 00-7 0C6.27.38 4.09.74 4.09.74a5.07 5.07 0 00-.09 3.77A5.44 5.44 0 002.5 8.52c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 008 15.13V19"))
                    }
                }
            }
        }
    }
}

fn nav_link(href: &str, label: &str, path_d: &str) -> Markup {
    html! {
        a href=(href) class="flex items-center gap-1.5 text-gray-300 hover:text-white hover:underline underline-offset-4 transition-all duration-200" {
            svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d=(path_d) {}
            }
            span { (label) }
        }
    }
}

fn external_link(url: &str, label: &str, icon_path: &str) -> Markup {
    html! {
        a
            href=(url)
            target="_blank"
            rel="noreferrer"
            class="inline-flex items-center gap-1.5 rounded-md bg-white/10 px-3 py-1 text-sm font-semibold hover:bg-white/20 transition-colors"
        {
            svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d=(icon_path) {}
            }
            span { (label) }
        }
    }
}
