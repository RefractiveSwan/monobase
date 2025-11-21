use maud::{Markup, html};

use crate::views::components::{badge::*, button::*, card::*, input::*, layout::*, typography::*};
use crate::views::layout::{Breadcrumb, PageCallout, PageShellProps, ViewChrome, page_shell};
use crate::views::models::{AlertKind, AlertMessage};

pub fn render_components_preview_page(chrome: &ViewChrome) -> String {
    page_shell(PageShellProps {
        title: "UI Components Gallery",
        chrome,
        content: html! {
            (section_heading("Buttons"))
            (render_button_gallery())
            (section_heading("Cards & Metrics"))
            (render_card_gallery())
            (section_heading("Badges & Alerts"))
            (render_badge_gallery())
            (section_heading("Form Inputs"))
            (render_form_gallery())
        },
        breadcrumbs: vec![
            Breadcrumb::home(),
            Breadcrumb::new("UI Kit", Some("/ui/components")),
        ],
        callouts: vec![PageCallout::info(
            "Visual QA surface",
            "Route used by developers to manually verify Tailwind tokens/components after refactors."
        )],
    })
    .into_string()
}

fn render_button_gallery() -> Markup {
    html! {
        div class="grid gap-4 md:grid-cols-3" {
            (card(html! {
                (card_header("Primary button", None))
                (card_body(html! {
                    div class="py-4" {
                        (primary_button(ButtonProps {
                            text: "Submit bundle",
                            type_: "button",
                            icon: Some(sample_icon()),
                            ..Default::default()
                        }))
                    }
                }))
            }))
            (card(html! {
                (card_header("Secondary button", None))
                (card_body(html! {
                    div class="py-4" {
                        button class="inline-flex items-center rounded-md border border-gray-300 px-4 py-2 text-sm font-medium text-navy-900 hover:bg-gray-50" { "Download NDJSON" }
                    }
                }))
            }))
            (card(html! {
                (card_header("Ghost button", None))
                (card_body(html! {
                    div class="py-4" {
                        button class="inline-flex items-center rounded-md px-4 py-2 text-sm font-medium text-gray-500 hover:text-navy-900" { "Reset filters" }
                    }
                }))
            }))
        }
    }
}

fn render_card_gallery() -> Markup {
    html! {
        (grid_section(html! {
            (card(html! {
                (card_header("Mapping metrics", None))
                (card_body(html! {
                    div class="grid gap-4 md:grid-cols-3" {
                        (metric_card("Bundles", "128", Some(("↑ 8%", "up"))))
                        (metric_card("AutoMapped", "2,431", Some(("↓ 2%", "down"))))
                        (metric_card("Needs review", "87", None))
                    }
                }))
            }))
            (card(html! {
                (card_header("Typography tokens", None))
                (card_body(html! {
                    div class="space-y-2" {
                        (section_heading("Section Heading"))
                        (subsection_heading("Subsection heading"))
                        (body_text("Body copy keeps a relaxed line height for longform docs."))
                        (label_text("Label text"))
                        (helper_text("Helper text anchors forms to requirements."))
                        div class="space-x-2" {
                            (code_badge("MappingResult"))
                            (code_badge("PipelineMetrics"))
                        }
                    }
                }))
            }))
        }))
    }
}

fn render_badge_gallery() -> Markup {
    html! {
        (grid_section(html! {
            (card(html! {
                (card_header("State chips", None))
                (card_body(html! {
                    div class="flex flex-wrap gap-2" {
                        (state_chip("auto_mapped"))
                        (state_chip("needs_review"))
                        (state_chip("no_match"))
                    }
                }))
            }))
            (card(html! {
                (card_header("Alerts", None))
                (card_body(html! {
                    div class="space-y-3" {
                        (alert(&AlertMessage { kind: AlertKind::Info, text: "Bundle ingested successfully.".into() }))
                        (alert(&AlertMessage { kind: AlertKind::Error, text: "Validation error: missing ServiceRequest.".into() }))
                        div class="flex flex-wrap gap-2" {
                            (status_badge("active"))
                            (status_badge("needs_review"))
                            (status_badge("no_match"))
                        }
                    }
                }))
            }))
        }))
    }
}

fn render_form_gallery() -> Markup {
    html! {
        (grid_section(html! {
            (card(html! {
                (card_header("Text inputs", None))
                (card_body(html! {
                    form class="space-y-4" {
                        label class="block" {
                            (label_text("NCIt ID"))
                            (text_input("ncit_id_demo", Some("C1234"), "C1234"))
                        }
                        label class="block" {
                            (label_text("JSON"))
                            (textarea("bundle_demo", 4, "{ \"resourceType\": \"Bundle\" }"))
                        }
                    }
                }))
            }))
            (card(html! {
                (card_header("File upload", None))
                (card_body(html! {
                    form class="space-y-4" {
                        (file_upload("bundle_file_demo", "application/json"))
                        (primary_button(ButtonProps {
                            text: "Upload sample",
                            type_: "button",
                            icon: Some(sample_icon()),
                            ..ButtonProps::default()
                        }))
                    }
                }))
            }))
        }))
    }
}

fn sample_icon() -> Markup {
    html! {
        svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M12 5l7 7-7 7" {}
        }
    }
}
