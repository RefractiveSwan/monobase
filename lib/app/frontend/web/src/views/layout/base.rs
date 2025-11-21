use maud::{DOCTYPE, Markup, html};

use crate::views::styles::theme::{font_preloads, htmx_indicator_styles, tailwind_theme_script};

/// Base HTML scaffold shared by every page. Higher level shells inject layout chrome.
pub(crate) fn base_layout(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html class="h-full bg-[#f8f9fa]" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) }

                (font_preloads())
                script src="https://cdn.tailwindcss.com" {}
                script src="https://unpkg.com/htmx.org@1.9.12" {}
                (htmx_indicator_styles())
                (tailwind_theme_script())
            }
            body class="min-h-screen bg-paper text-navy-900 font-sans antialiased" {
                (content)
            }
        }
    }
}
