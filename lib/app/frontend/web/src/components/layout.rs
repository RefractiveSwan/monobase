use maud::{Markup, html};

pub fn page_container(content: Markup) -> Markup {
    html! {
        main class="mx-auto max-w-7xl px-6 lg:px-8 py-10 space-y-10" {
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
pub fn navbar() -> Markup {
    html! {
        nav class="sticky top-0 z-50 w-full bg-white/80 backdrop-blur-md border-b border-gray-100" {
            div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8" {
                div class="flex h-16 items-center justify-between" {
                    // Brand
                    div class="flex-shrink-0 flex items-center gap-3" {
                        a href="/" class="flex items-center gap-2 group" {
                            // Swan Icon (Abstract)
                            svg class="h-8 w-8 text-navy-900 group-hover:text-gold-600 transition-colors duration-300" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M14 10l-2 1m0 0l-2-1m2 1v2.5M20 7l-2 1m2-1l-2-1m2 1v2.5M14 4l-2-1-2 1M4 7l2-1M4 7l2 1M4 7v2.5M12 21l-2-1m2 1l2-1m-2 1v-2.5M6 18l-2-1v-2.5M18 18l2-1v-2.5" {}
                            }
                            span class="font-serif text-xl font-bold text-navy-900 tracking-tight group-hover:text-navy-700 transition-colors" { "Refractive Swan" }
                        }
                    }

                    // Navigation
                    div class="hidden md:block" {
                        div class="ml-10 flex items-baseline space-x-8" {
                            (crate::components::button::nav_link("/map", "Workbench", html!{}))
                            (crate::components::button::nav_link("/analytics", "Analytics", html!{}))
                            (crate::components::button::nav_link("/eval", "Evaluation", html!{}))
                            a href="https://github.com/refractive-swan" target="_blank" class="text-sm font-medium text-gray-500 hover:text-gold-600 transition-colors" { "GitHub" }
                        }
                    }
                }
            }
        }
    }
}

pub fn footer() -> Markup {
    html! {
        footer class="bg-white border-t border-gray-100 mt-auto" {
            div class="mx-auto max-w-7xl px-6 py-12 md:flex md:items-center md:justify-between lg:px-8" {
                div class="flex justify-center space-x-6 md:order-2" {
                    a href="#" class="text-gray-400 hover:text-gray-500" {
                        span class="sr-only" { "GitHub" }
                        svg class="h-6 w-6" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true" {
                            path fill-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clip-rule="evenodd" {}
                        }
                    }
                }
                div class="mt-8 md:order-1 md:mt-0" {
                    p class="text-center text-xs leading-5 text-gray-500" {
                        "&copy; 2024 Refractive Swan. Open Source Clinical Terminology."
                    }
                }
            }
        }
    }
}

pub fn base_layout(content: Markup) -> Markup {
    html! {
        (maud::DOCTYPE)
        html class="h-full bg-paper antialiased" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Refractive Swan" }

                link rel="preconnect" href="https://fonts.googleapis.com";
                link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="";
                link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600&family=Merriweather:ital,wght@0,300;0,400;0,700;1,300;1,400&family=Roboto+Mono:wght@400;500&display=swap" rel="stylesheet";

                script src="https://cdn.tailwindcss.com" {}
                script src="https://unpkg.com/htmx.org@1.9.12" {}

                // HTMX Loading Indicator Styles
                style {
                    (maud::PreEscaped(r#"
                        .htmx-indicator { display: none; }
                        .htmx-request .htmx-indicator { display: inline-block; }
                        .htmx-request.htmx-indicator { display: inline-block; }
                        @keyframes spin { to { transform: rotate(360deg); } }
                        .animate-spin { animation: spin 1s linear infinite; }
                    "#))
                }

                // Theme Configuration
                script {
                    (maud::PreEscaped(r#"
                        tailwind.config = {
                            theme: {
                                extend: {
                                    fontFamily: {
                                        sans: ['Inter', 'sans-serif'],
                                        serif: ['Merriweather', 'serif'],
                                        mono: ['Roboto Mono', 'monospace'],
                                    },
                                    colors: {
                                        navy: {
                                            50: '#f0f4f8',
                                            100: '#d9e2ec',
                                            600: '#334155',
                                            700: '#1e293b',
                                            800: '#0f172a',
                                            900: '#020617',
                                        },
                                        gold: {
                                            50: '#fefce8',
                                            100: '#fbf3db',
                                            500: '#a78b4a',
                                            600: '#8b7239',
                                        },
                                        paper: '#fafafa',
                                    },
                                    boxShadow: {
                                        'academic': '0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px 0 rgba(0, 0, 0, 0.06)',
                                        'academic-lg': '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
                                    }
                                }
                            }
                        }
                    "#))
                }
            }
            body class="flex min-h-full flex-col" {
                (navbar())
                (content)
                (footer())
            }
        }
    }
}
