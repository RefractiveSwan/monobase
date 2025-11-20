use maud::{Markup, PreEscaped, html};

pub struct ButtonProps<'a> {
    pub text: &'a str,
    pub icon: Option<Markup>,
    pub type_: &'a str,           // "submit", "button"
    pub hx_attrs: Option<Markup>, // e.g. hx-post="..."
}

impl<'a> Default for ButtonProps<'a> {
    fn default() -> Self {
        Self {
            text: "",
            icon: None,
            type_: "button",
            hx_attrs: None,
        }
    }
}

pub fn primary_button(props: ButtonProps) -> Markup {
    let hx_attrs = props.hx_attrs.unwrap_or(html! {}).0;
    let class = "inline-flex items-center gap-2 rounded-md bg-navy-900 px-6 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-navy-800 hover:scale-[1.02] active:scale-[0.98] focus:ring-2 focus:ring-navy-900 focus:ring-offset-2 transition-all duration-200";
    html! {
        (PreEscaped(format!("<button type=\"{}\" {} class=\"{}\">", props.type_, hx_attrs, class)))
            @if let Some(icon) = props.icon {
                (icon)
            }
            span { (props.text) }
            // Loading spinner (hidden by default, shown by htmx)
            svg class="htmx-indicator w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24" {
                circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" {}
                path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" {}
            }
        (PreEscaped("</button>"))
    }
}

pub fn nav_link(href: &str, text: &str, icon: Markup) -> Markup {
    html! {
        a href=(href) class="flex items-center gap-1.5 text-gray-300 hover:text-white hover:underline underline-offset-4 transition-all duration-200" {
            (icon)
            span { (text) }
        }
    }
}
