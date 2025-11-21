use maud::{Markup, html};

/// Top navigation bar shared across workbench pages.
pub fn render_navbar() -> Markup {
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
