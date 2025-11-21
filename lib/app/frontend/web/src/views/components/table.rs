use maud::{Markup, html};

pub fn table_container(content: Markup) -> Markup {
    html! {
        div class="overflow-x-auto border border-gray-200 rounded-md" {
            table class="min-w-full divide-y divide-gray-200 text-sm" {
                (content)
            }
        }
    }
}

pub fn table_header(headers: &[&str]) -> Markup {
    html! {
        thead class="bg-gray-50" {
            tr {
                @for header in headers {
                    th class="px-3 py-2 text-left text-xs font-bold text-gray-500 uppercase tracking-wider" { (header) }
                }
            }
        }
    }
}

pub fn table_row(content: Markup) -> Markup {
    html! {
        tr class="hover:bg-gray-50 hover:border-l-4 hover:border-l-navy-900 transition-all duration-150 cursor-pointer" {
            (content)
        }
    }
}

pub fn table_cell(content: Markup) -> Markup {
    html! {
        td class="px-3 py-2 text-xs text-gray-600" { (content) }
    }
}

pub fn table_cell_mono(content: Markup) -> Markup {
    html! {
        td class="px-3 py-2 font-mono text-xs text-navy-900" { (content) }
    }
}
