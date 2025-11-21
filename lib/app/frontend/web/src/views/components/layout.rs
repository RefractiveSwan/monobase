use maud::{Markup, html};

pub fn page_container(content: Markup) -> Markup {
    html! {
        div class="mx-auto max-w-7xl px-6 lg:px-8 space-y-8" {
            (content)
        }
    }
}

pub fn grid_section(content: Markup) -> Markup {
    html! {
        section class="grid gap-6 lg:grid-cols-2" {
            (content)
        }
    }
}
