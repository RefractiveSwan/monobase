use maud::{Markup, html};

pub fn page_title(text: &str) -> Markup {
    html! {
        h1 class="text-xl font-serif font-bold tracking-wide" { (text) }
    }
}

pub fn section_heading(text: &str) -> Markup {
    html! {
        h2 class="text-3xl font-serif font-bold text-navy-900 leading-tight" { (text) }
    }
}

pub fn subsection_heading(text: &str) -> Markup {
    html! {
        h3 class="text-sm font-bold text-navy-900 uppercase tracking-wider" { (text) }
    }
}

pub fn card_heading(text: &str) -> Markup {
    html! {
        h2 class="text-lg font-serif font-bold text-navy-900" { (text) }
    }
}

pub fn body_text(text: &str) -> Markup {
    html! {
        p class="text-slate-600 leading-relaxed" { (text) }
    }
}

pub fn code_badge(text: &str) -> Markup {
    html! {
        code class="font-mono text-xs bg-navy-50 px-1 py-0.5 rounded text-navy-800" { (text) }
    }
}

pub fn label_text(text: &str) -> Markup {
    html! {
        span class="text-xs font-semibold text-gray-700 uppercase tracking-wide" { (text) }
    }
}

pub fn helper_text(text: &str) -> Markup {
    html! {
        span class="text-xs text-gray-500 ml-2" { (text) }
    }
}
