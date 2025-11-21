use maud::{Markup, PreEscaped, html};

/// Shared font preloads for the workbench's academic aesthetic.
pub fn font_preloads() -> Markup {
    html! {
        link rel="preconnect" href="https://fonts.googleapis.com";
        link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="";
        link
            href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600&family=Merriweather:ital,wght@0,300;0,400;0,700;1,300;1,400&family=Roboto+Mono:wght@400;500&display=swap"
            rel="stylesheet";
    }
}

/// Configures Tailwind's runtime theme so components stay consistent across pages.
pub fn tailwind_theme_script() -> Markup {
    html! {
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
                            },
                            spacing: {
                                '18': '4.5rem',
                                '22': '5.5rem',
                            }
                        }
                    }
                };
            "#))
        }
    }
}

/// Minimal HTMX indicator styles shared across fragments.
pub fn htmx_indicator_styles() -> Markup {
    html! {
        style {
            (PreEscaped(
                r#"
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
                "#,
            ))
        }
    }
}
