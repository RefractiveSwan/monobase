use maud::{Markup, html};

use crate::{
    config::AppConfig,
    views::components::layout::page_container,
    views::styles::{
        footer::{FooterLinks, render_footer},
        navbar::{NavbarLinks, render_navbar},
    },
};

use super::base::base_layout;

pub const DEFAULT_GITHUB_URL: &str = "https://github.com/RefractiveSwan/monobase";

#[derive(Debug, Clone)]
pub struct ViewChrome {
    pub docs_url: Option<String>,
    pub github_url: Option<String>,
    pub announcements: Vec<PageAnnouncement>,
}

impl Default for ViewChrome {
    fn default() -> Self {
        Self {
            docs_url: None,
            github_url: Some(DEFAULT_GITHUB_URL.to_string()),
            announcements: Vec::new(),
        }
    }
}

impl From<&AppConfig> for ViewChrome {
    fn from(config: &AppConfig) -> Self {
        let mut chrome = Self::default();
        chrome.docs_url = config.docs_url.clone();
        chrome.github_url = config
            .github_url
            .clone()
            .or_else(|| Some(DEFAULT_GITHUB_URL.to_string()));
        chrome
    }
}

impl ViewChrome {
    pub fn with_announcements(mut self, announcements: Vec<PageAnnouncement>) -> Self {
        self.announcements = announcements;
        self
    }
}

#[derive(Debug, Clone)]
pub struct PageAnnouncement {
    pub text: String,
    pub href: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Breadcrumb {
    pub label: String,
    pub href: Option<String>,
}

impl Breadcrumb {
    pub fn new(label: impl Into<String>, href: Option<impl Into<String>>) -> Self {
        Self {
            label: label.into(),
            href: href.map(|value| value.into()),
        }
    }

    pub fn home() -> Self {
        Self::new("Home", Some("/"))
    }
}

#[derive(Debug, Clone)]
pub enum CalloutKind {
    Info,
    Warning,
}

#[derive(Debug, Clone)]
pub struct PageCallout {
    pub title: String,
    pub body: String,
    pub kind: CalloutKind,
}

impl PageCallout {
    pub fn info(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            kind: CalloutKind::Info,
        }
    }
}

pub struct PageShellProps<'a> {
    pub title: &'a str,
    pub chrome: &'a ViewChrome,
    pub content: Markup,
    pub breadcrumbs: Vec<Breadcrumb>,
    pub callouts: Vec<PageCallout>,
}

/// Shared shell that renders navbar, breadcrumbs, callouts, and footer around page content.
pub fn page_shell(props: PageShellProps<'_>) -> Markup {
    let navbar_links = NavbarLinks {
        docs_url: props.chrome.docs_url.as_deref(),
        github_url: props.chrome.github_url.as_deref(),
    };
    let footer_links = FooterLinks {
        docs_url: props.chrome.docs_url.as_deref(),
        github_url: props.chrome.github_url.as_deref(),
    };
    base_layout(
        props.title,
        html! {
            (render_navbar(navbar_links))
            main class="pb-12" {
                (page_container(html! {
                    (render_announcements(&props.chrome.announcements))
                    (render_breadcrumbs(&props.breadcrumbs))
                    (render_callouts(&props.callouts))
                    (props.content)
                }))
            }
            (render_footer(footer_links))
        },
    )
}

fn render_announcements(items: &[PageAnnouncement]) -> Markup {
    if items.is_empty() {
        return html! {};
    }
    html! {
        div class="space-y-3 mb-4" {
            @for item in items {
                div class="flex items-center justify-between rounded-md bg-amber-50 border border-amber-200 px-3 py-2 text-sm text-amber-900" {
                    span { (&item.text) }
                    @if let Some(href) = &item.href {
                        a href=(href) class="text-amber-800 font-semibold hover:underline" target="_blank" rel="noreferrer" { "View" }
                    }
                }
            }
        }
    }
}

fn render_breadcrumbs(items: &[Breadcrumb]) -> Markup {
    if items.is_empty() {
        return html! {};
    }
    html! {
        nav class="flex items-center text-sm text-gray-500 mb-6" aria-label="Breadcrumb" {
            ol class="inline-flex items-center space-x-2" {
                @for (index, crumb) in items.iter().enumerate() {
                    li class="inline-flex items-center gap-2" {
                        @if index > 0 {
                            span class="text-gray-400" { "›" }
                        }
                        @if let Some(href) = &crumb.href {
                            a href=(href) class="hover:text-gray-900 transition-colors" { (&crumb.label) }
                        } @else {
                            span class="text-gray-700 font-medium" { (&crumb.label) }
                        }
                    }
                }
            }
        }
    }
}

fn render_callouts(items: &[PageCallout]) -> Markup {
    if items.is_empty() {
        return html! {};
    }
    html! {
        div class="space-y-4 mb-8" {
            @for callout in items {
                @let tone = match callout.kind {
                    CalloutKind::Warning => "border-rose-200 bg-rose-50 text-rose-900",
                    CalloutKind::Info => "border-sky-200 bg-sky-50 text-sky-900",
                };
                div class=(format!("rounded-md border px-4 py-3 flex flex-col gap-1 {}", tone)) {
                    span class="text-sm font-semibold" { (&callout.title) }
                    p class="text-sm text-inherit" { (&callout.body) }
                }
            }
        }
    }
}
