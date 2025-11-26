# Frontend Accessibility Checklist

The FE-028 expansion surfaces far more controls, so every release must re-run this checklist across `/`, `/map`, `/observability`, and `/eval`.

## Form semantics

- [x] Every input and textarea carries a `<label>` or `aria-label`. The eval scheduler and mapping upload forms share the `label_text` helper so screen readers announce control intent.
- [x] HTMX-only buttons include `type="button"` to avoid accidental submissions (`Compare runs`, `Reset selection`, and the template picker).
- [x] Status badges expose textual labels (`queued`, `running`, etc.) alongside color; never rely on color alone.
- [x] Pagination/actions reachable via keyboard (tab order mirrors visual order).

## Keyboard navigation

- [x] All dynamic panes (`#results`, `#eval-jobs-panel`, `#eval-compare-panel`) set `hx-target` so swaps stay inside focusable containers.
- [x] After HTMX form submissions, focus remains on the triggering element (HTMX default). Screen readers can re-announce the updated region with `aria-live="polite"`—applied to log + job panels.
- [x] Pressing `Esc` anywhere inside paste/upload forms returns focus to the main textarea (handled via `autofocus` + `tabindex=0`).

## Color & contrast

- [x] Tier badges now use 4.5:1 compliant color pairs (amber, slate, orange, gray) and icon-less tags.
- [x] Job status badges rely on `status_badge` which enforces contrast-safe palettes.

## Miscellaneous

- [x] Live log and eval job panes include textual timestamps so screen-reader users can triage without watching animations.
- [x] System metrics tables render as semantic `<table>` when data is tabular; grids are reserved for card layouts only.

Re-run this list whenever a new HTMX fragment or UI surface lands; reference `docs/book/src/frontend/command-center.md` for expected screenshots/states.
