use maud::{Markup, html};

pub fn text_input(name: &str, value: Option<&str>, placeholder: &str) -> Markup {
    html! {
        input type="text" name=(name) value=[value] placeholder=(placeholder) class="rounded-md border-gray-300 px-3 py-1.5 text-sm focus:border-navy-900 focus:ring-1 focus:ring-navy-900 w-full" {}
    }
}

pub fn textarea(name: &str, rows: u8, placeholder: &str) -> Markup {
    html! {
        textarea id=(name) name=(name) rows=(rows) placeholder=(placeholder) class="w-full rounded-md border-2 border-gray-300 p-3 font-mono text-xs focus:border-navy-900 focus:ring-2 focus:ring-navy-900 focus:ring-offset-2 transition-all" {}
    }
}

pub fn file_upload(name: &str, accept: &str) -> Markup {
    html! {
        input type="file" id=(name) name=(name) accept=(accept) class="w-full rounded-md border-2 border-gray-300 p-3 text-sm focus:border-navy-900 focus:ring-2 focus:ring-navy-900 focus:ring-offset-2 transition-all" {}
    }
}
