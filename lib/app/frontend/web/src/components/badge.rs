use crate::view_model::{AlertKind, AlertMessage};
use maud::{Markup, html};

pub fn status_badge(ok: bool, status: &str) -> Markup {
    let (bg, text, border, dot) = if ok {
        (
            "bg-emerald-50",
            "text-emerald-800",
            "border-emerald-200",
            "bg-emerald-600",
        )
    } else {
        (
            "bg-amber-50",
            "text-amber-800",
            "border-amber-200",
            "bg-amber-600",
        )
    };

    html! {
        div class=(format!("inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-medium border {} {} {}", bg, text, border)) {
            span class=(format!("h-1.5 w-1.5 rounded-full {}", dot)) {}
            span { (format!("System Status: {}", status)) }
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
