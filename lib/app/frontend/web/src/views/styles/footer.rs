use maud::{Markup, html};

/// Docs/GitHub URLs exposed to the shared footer.
#[derive(Clone, Copy)]
pub struct FooterLinks<'a> {
    pub docs_url: Option<&'a str>,
    pub github_url: Option<&'a str>,
}

/// Footer component shared across pages.
pub fn render_footer(links: FooterLinks<'_>) -> Markup {
    html! {
        footer class="bg-white border-t border-gray-200 mt-12" {
            div class="mx-auto max-w-7xl px-6 py-8 flex flex-col sm:flex-row items-center justify-between gap-4 text-xs text-gray-500" {
                p class="font-serif italic text-center sm:text-left" {
                    "Refractive Swan Clinical Model • Project Hierophancy"
                }
                div class="flex items-center gap-4" {
                    @if let Some(url) = links.docs_url {
                        a href=(url) target="_blank" rel="noreferrer" class="hover:text-gray-800 underline-offset-4 hover:underline" { "Docs" }
                    }
                    @if let Some(url) = links.github_url {
                        a href=(url) target="_blank" rel="noreferrer" class="hover:text-gray-800 underline-offset-4 hover:underline" { "GitHub" }
                    }
                }
            }
        }
    }
}
