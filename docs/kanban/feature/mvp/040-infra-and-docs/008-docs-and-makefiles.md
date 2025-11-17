# Kanban – 008-docs-and-makefiles

**Branch:** `feature/meta/docs-and-makefiles`  
**Goal:** Introduce a visible documentation system (mdBook) and a workspace task runner (Makefile). Expose built docs at `/docs` in the web frontend when available.

### Columns
* **TODO** – Not started yet
* **INPROGRESS** – In progress
* **REVIEW** – Needs code review / polish
* **DONE** – Completed

---

## TODO


### DOCS-06 – CI hook
- [ ] Optional: add a CI step to run `cargo make docs` or `cargo make ci` to ensure tasks/docs build.

---

## INPROGRESS
- _Empty_

## REVIEW
- [ ] Confirm `/docs` redirects to the configured `DFPS_DOCS_URL`.
- [ ] Confirm `make` targets succeed across the workspace on a clean checkout.

### DOCS-01 – mdBook scaffold
- [x] Add `docs/book/book.toml` and `docs/book/src/{SUMMARY.md,index.md}`.
- [x] Structure “Runbooks” + “Kanban (Feature)” navigation.

### DOCS-02 – Sync existing docs into the book
- [x] `cargo make docs-sync` copies `docs/runbook/**` and `docs/kanban/**` into `docs/book/src/`.
- [x] Keep source of truth in `docs/*` (book holds synced copies only).

### DOCS-03 – Build/serve docs
- [x] `cargo make docs` builds the book to `docs/book/book/`.
- [x] `cargo make docs-serve` runs `mdbook serve` for local browsing.

### DOCS-04 – Web UI integration
- [x] Add `DFPS_DOCS_URL` support to `dfps_web_frontend`; when set, `/docs` redirects there.
- [x] Update the frontend README with the new env var.

### MAKE-01 – Workspace Makefile
- [x] Add a root `Makefile` with standard workspace targets:
  - [x] `build`, `check`, `fmt`, `clippy`, `test`, `ci`, `clean`
  - [x] `docs`, `docs-sync`, `docs-serve`
  - [x] `api` (backend), `web` (frontend)
  - [x] `map-bundles` / `map-codes` (wrapping `dfps_cli`)
- [x] Ensure commands run as `make <target>` from `code/`.

### DOCS-05 – Makefile quickstart
- [x] Add `docs/runbook/makefile-quickstart.md` covering:
  - [x] Installing `mdbook` for docs targets.
  - [x] Core targets (`cargo make build`, `cargo make ci`, `cargo make docs`, `cargo make web`, etc.).
  - [x] How `DFPS_ENV` / env profiles interact with targets.

---

- `docs/book/` scaffolded with runbook + kanban navigation.
- `cargo make docs-sync`, `cargo make docs`, and `cargo make docs-serve` manage the mdBook pipeline.
- Web frontend redirects `/docs` to `DFPS_DOCS_URL`; README documents the env var.
- Workspace `Makefile` and `docs/runbook/makefile-quickstart.md` describe all targets.

---

## Acceptance Criteria
- `cargo make docs` builds mdBook without errors.
- `cargo make web` serves the UI and, when `DFPS_DOCS_URL` is set, `/docs` redirects there.
- `cargo make ci` runs fmt, clippy (`-D warnings`), and tests successfully.

## Out of Scope
- Full-text search, custom mdBook theming, or public hosting for the book.
