# `views/` – HTMX/Tailwind renderer

The `refractive_swan_web_frontend` crate keeps all HTML/HTMX markup under `views/` so
handlers stay slim. Each sub-module covers a different slice of the UI stack:

```
views/
├── components/      # Reusable atoms: buttons, cards, inputs, typography helpers
├── layout/          # Base HTML template + page_shell (nav/footer/callouts/breadcrumbs)
├── models/          # View-facing structs (PageContext, MappingResultsView, etc.)
├── pages/           # Top-level pages (landing, workbench, analytics, eval, UI kit)
├── partials/        # HTMX fragments (results panel, eval panel, etc.)
└── styles/          # Navbar/footer renderers + Tailwind theme tokens
```

### Layout flow

1. `layout/base.rs` renders `<html>`/`<head>` boilerplate and loads Tailwind/HTMX.
2. `layout/shell.rs` composes the navbar, breadcrumbs, callouts, announcements, and footer
   by taking a `PageShellProps` (title, breadcrumbs, ViewChrome).
3. Individual pages call `page_shell(...)` with their inner content so every route benefits
   from the shared chrome.

`styles/theme.rs` centralizes the Tailwind config snippet, font preloads, and HTMX indicator
CSS to avoid sprinkling inline `<script>`/`<style>` blocks across pages.

### Component preview

`pages/components_preview.rs` powers the `/ui/components` route, giving designers a
Storybook-like surface that shows the standard cards/buttons/badges with the academic
theme tokens applied. The navbar links to it for quick manual QA.
