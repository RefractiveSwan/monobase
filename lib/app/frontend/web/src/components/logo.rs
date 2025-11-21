use maud::{Markup, PreEscaped, html};

/// Renders the full logo with icon and text.
pub fn logo_full() -> Markup {
    html! {
        a href="/" class="flex items-center gap-3 group" {
            (logo_icon("h-10 w-10"))
            span class="font-serif text-xl font-bold text-navy-900 tracking-tight group-hover:text-navy-700 transition-all duration-300" {
                "Refractive Swan"
            }
        }
    }
}

/// Renders just the icon (enhanced swan with fluorescent gradient).
pub fn logo_icon(classes: &str) -> Markup {
    html! {
        svg class=(format!("{} transition-all duration-300 group-hover:drop-shadow-[0_0_8px_rgba(0,255,204,0.6)]", classes))
             viewBox="0 0 100 100"
             fill="none"
             xmlns="http://www.w3.org/2000/svg" {
            (PreEscaped(r##"
                <defs>
                    <linearGradient id="swanGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                        <stop offset="0%" style="stop-color:#00ffcc;stop-opacity:1" />
                        <stop offset="50%" style="stop-color:#00d4ff;stop-opacity:1" />
                        <stop offset="100%" style="stop-color:#00ffcc;stop-opacity:0.8" />
                    </linearGradient>
                    <filter id="iconGlow">
                        <feGaussianBlur stdDeviation="2" result="coloredBlur"/>
                        <feMerge>
                            <feMergeNode in="coloredBlur"/>
                            <feMergeNode in="SourceGraphic"/>
                        </feMerge>
                    </filter>
                </defs>
                
                <!-- Swan Icon with gradient and glow -->
                <g transform="translate(20, 20)" filter="url(#iconGlow)">
                    <!-- Swan body - geometric elegant shape -->
                    <path d="M 30 12 Q 42 8, 48 20 Q 46 32, 42 42 L 24 42 Q 18 32, 14 24 Q 18 14, 30 12 Z" fill="url(#swanGradient)" opacity="0.95"/>
                    
                    <!-- Swan neck - flowing S-curve suggesting refraction -->
                    <path d="M 30 12 Q 22 18, 18 30 Q 16 38, 14 48" stroke="url(#swanGradient)" stroke-width="5" fill="none" stroke-linecap="round"/>
                    
                    <!-- Swan head -->
                    <circle cx="12" cy="52" r="6" fill="url(#swanGradient)"/>
                    
                    <!-- Light refraction accent lines -->
                    <path d="M 34 22 L 42 28 M 28 24 L 36 32 M 32 28 L 38 36" stroke="#00ffcc" stroke-width="2" opacity="0.6" stroke-linecap="round"/>
                    
                    <!-- Additional glow accent -->
                    <circle cx="36" cy="26" r="18" fill="#00ffcc" opacity="0.08"/>
                </g>
            "##))
        }
    }
}

/// Renders just the text "Refractive Swan" with optional styling.
pub fn logo_text_only(classes: &str) -> Markup {
    html! {
        span class=(format!("font-serif text-xl font-bold text-navy-900 tracking-tight {}", classes)) {
            "Refractive Swan"
        }
    }
}
