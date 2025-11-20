use crate::components::typography::card_heading;
use maud::{Markup, html};

pub fn card(content: Markup) -> Markup {
    html! {
        div class="bg-white shadow-academic rounded-md border border-gray-200" {
            (content)
        }
    }
}

pub fn card_header(title: &str, right_content: Option<Markup>) -> Markup {
    html! {
        div class="bg-navy-50 px-6 py-3 border-b border-gray-200 flex items-center justify-between" {
            (card_heading(title))
            @if let Some(content) = right_content {
                (content)
            }
        }
    }
}

pub fn card_body(content: Markup) -> Markup {
    html! {
        div class="p-6" {
            (content)
        }
    }
}

pub fn metric_card(
    label: &str,
    value: impl std::fmt::Display,
    subtext: &str,
    text_color: &str,
) -> Markup {
    html! {
        div class="rounded-md border border-gray-200 p-4 bg-white shadow-sm" {
            p class="text-xs font-semibold text-gray-500 uppercase tracking-wide" { (label) }
            p class=(format!("text-2xl font-serif font-bold mt-1 {}", text_color)) { (value) }
            p class="text-xs text-gray-400 mt-1" { (subtext) }
        }
    }
}

pub fn state_metric_card(label: &str, count: usize, style_classes: &str, subtext: &str) -> Markup {
    html! {
        div class=(format!("rounded-md border p-4 {}", style_classes)) {
            div class="flex justify-between items-start" {
                p class="text-xs font-bold uppercase tracking-wide opacity-80" { (label) }
                span class="text-2xl font-bold" { (count) }
            }
            p class="text-xs opacity-70 mt-1" { (subtext) }
        }
    }
}

pub fn secondary_metric(title: &str, value: usize, text_class: &str) -> Markup {
    html! {
        div class="text-center" {
            p class="text-xs text-gray-500" { (title) }
            p class=(format!("text-lg font-mono font-bold {}", text_class)) { (value) }
        }
    }
}

pub fn secondary_metric_avg(title: &str, value: Option<f64>, text_class: &str) -> Markup {
    html! {
        div class="text-center" {
            p class="text-xs text-gray-500" { (title) }
            p class=(format!("text-lg font-mono font-bold {}", text_class)) {
                @if let Some(v) = value {
                    (format!("{:.1}", v))
                } @else {
                    "n/a"
                }
            }
        }
    }
}
