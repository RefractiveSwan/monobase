use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::views::styles::{footer::render_footer, navbar::render_navbar};

/// The base layout for all pages, including header, footer, and common scripts/styles.
pub(crate) fn base_layout(content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html class="h-full bg-[#f8f9fa]" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Refractive Swan Mapping Workbench" }

                // Typography: Merriweather (Serif headings), Inter (Sans body), Roboto Mono (Code)
                link rel="preconnect" href="https://fonts.googleapis.com";
                link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="";
                link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600&family=Merriweather:ital,wght@0,300;0,400;0,700;1,300;1,400&family=Roboto+Mono:wght@400;500&display=swap" rel="stylesheet";

                script src="https://cdn.tailwindcss.com" {}
                script src="https://unpkg.com/htmx.org@1.9.12" {}

                // HTMX Loading Indicator Styles
                style {
                    (PreEscaped(r#"
                        .htmx-indicator {
                            display: none;
                        }
                        .htmx-request .htmx-indicator {
                            display: inline-block;
                        }
                        .htmx-request.htmx-indicator {
                            display: inline-block;
                        }
                        @keyframes spin {
                            to { transform: rotate(360deg); }
                        }
                        .animate-spin {
                            animation: spin 1s linear infinite;
                        }
                    "#))
                }

                // Academic Theme Configuration - Enhanced for Accessibility
                script {
                    (PreEscaped(r#"
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
                                            600: '#334155',  // Better contrast for secondary text
                                            700: '#1e293b',  // Better contrast for body text
                                            800: '#0f172a',
                                            900: '#020617',  // Deeper for maximum contrast
                                        },
                                        gold: {
                                            50: '#fefce8',
                                            100: '#fbf3db',
                                            500: '#a78b4a',  // Adjusted for better contrast
                                            600: '#8b7239',
                                        },
                                        paper: '#fafafa',  // Slightly lighter for better contrast
                                    },
                                    boxShadow: {
                                        'academic': '0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px 0 rgba(0, 0, 0, 0.06)',
                                        'academic-lg': '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
                                    },
                                    spacing: {
                                        '18': '4.5rem',
                                        '22': '5.5rem',
                                    }
                                }
                            }
                        }
                    "#))
                }
            }
            body class="min-h-screen bg-paper text-navy-900 font-sans antialiased" {
                (render_navbar())

                (content)

                (render_footer())
            }
        }
    }
}
