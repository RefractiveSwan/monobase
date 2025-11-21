use crate::components::typography::card_heading;
use maud::{Markup, html};

pub fn card(content: Markup) -> Markup {
    html! {
        div class="bg-white shadow-academic rounded-md border border-gray-200 hover:shadow-academic-lg hover:border-fluor-cyan/20 transition-all duration-300" {
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

pub fn metric_card(label: &str, value: &str, trend: Option<(&str, &str)>) -> Markup {
    let (border_color, shadow_effect) = match trend {
        Some((_, "up")) => (
            "border-fluor-cyan",
            "hover:shadow-[0_0_20px_rgba(0,255,204,0.3)]",
        ),
        Some((_, "down")) => (
            "border-fluor-magenta",
            "hover:shadow-[0_0_20px_rgba(255,0,255,0.3)]",
        ),
        _ => ("border-transparent", "hover:shadow-academic-lg"),
    };

    html! {
        div class=(format!("overflow-hidden rounded-lg bg-white px-4 py-5 shadow-academic sm:p-6 border-t-4 {} {} transition-all duration-300 hover:scale-[1.02]", border_color, shadow_effect)) {
            dt class="truncate text-sm font-medium text-gray-500" { (label) }
            dd class="mt-1 text-3xl font-semibold tracking-tight text-navy-900" { (value) }
            @if let Some((trend_val, direction)) = trend {
                div class="mt-2 flex items-center text-sm" {
                    @if direction == "up" {
                        span class="text-fluor-cyan font-medium flex items-center gap-1" {
                            "↑ "
                            (trend_val)
                        }
                        span class="ml-2 text-gray-400" { "vs last week" }
                    } @else {
                        span class="text-fluor-magenta font-medium flex items-center gap-1" {
                            "↓ "
                            (trend_val)
                        }
                        span class="ml-2 text-gray-400" { "vs last week" }
                    }
                }
            }
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
