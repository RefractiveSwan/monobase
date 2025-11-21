use maud::{Markup, html};

/// Footer component shared across pages.
pub fn render_footer() -> Markup {
    html! {
        footer class="bg-white border-t border-gray-200 mt-12" {
            div class="mx-auto max-w-7xl px-6 py-8" {
                p class="text-center text-xs text-gray-500 font-serif italic" {
                    "Refractive Swan Clinical Model • Project Hierophancy"
                }
            }
        }
    }
}
