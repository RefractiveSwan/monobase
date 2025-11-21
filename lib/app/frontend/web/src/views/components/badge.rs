use crate::views::models::{AlertKind, AlertMessage};
use maud::{Markup, html};

pub fn status_badge(status: &str) -> Markup {
    let (bg, text) = match status {
        "active" | "ok" | "auto_mapped" => ("bg-fluor-cyan/10", "text-fluor-cyan"),
        "error" | "no_match" | "license_blocked" => ("bg-fluor-magenta/10", "text-fluor-magenta"),
        "warning" | "needs_review" | "processing" => ("bg-fluor-orange/10", "text-fluor-orange"),
        "new" | "info" => ("bg-fluor-lime/10", "text-fluor-lime"),
        _ => ("bg-gray-50", "text-gray-600"),
    };
    html! {
        span class=(format!("inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ring-1 ring-inset ring-current {} {}", bg, text)) {
            (status)
        }
    }
}

pub fn state_chip(state: &str) -> Markup {
    let (bg, text, border) = match state {
        "auto_mapped" => ("bg-emerald-50", "text-emerald-800", "border-emerald-100"),
        "needs_review" => ("bg-amber-50", "text-amber-800", "border-amber-100"),
        "no_match" => ("bg-rose-50", "text-rose-800", "border-rose-100"),
        _ => ("bg-gray-50", "text-gray-800", "border-gray-200"),
    };

    html! {
        span class=(format!("inline-flex items-center rounded px-2 py-0.5 text-xs font-medium border {} {} {}", bg, text, border)) {
            (state)
        }
    }
}

pub fn alert(alert: &AlertMessage) -> Markup {
    let (bg, border, text_color, icon) = match alert.kind {
        AlertKind::Info => ("bg-blue-50", "border-blue-200", "text-blue-800", "ℹ️"),
        AlertKind::Error => ("bg-rose-50", "border-rose-200", "text-rose-800", "⚠️"),
    };
    html! {
        div class=(format!("rounded-md border px-4 py-3 text-sm font-medium flex items-start gap-2 {} {} {}", bg, border, text_color)) {
            span { (icon) }
            span { (&alert.text) }
        }
    }
}
