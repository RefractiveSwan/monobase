# Kanban - feature/docs-hosting-and-search (021)

**Theme:** Docs polish - public hosting, full-text search, theming  
**Branch:** `feature/meta/DOCS-021-docs-hosting-and-search`  
**Goal:** Turn the local mdBook into a searchable, themed, publicly hosted documentation site, integrated with `/docs` in the frontend.

### Columns
* **TODO** – Not started yet  
* **INPROGRESS** – In progress  
* **REVIEW** – Needs code review / polish  
* **DONE** – Completed  

---

## TODO

### DOCS-HOST-01 – Search & theming

- [ ] Enable mdBook search:

  - [ ] Configure `[output.html.search]` in `docs/book/book.toml`.
  - [ ] Ensure the search index builds successfully (`cargo make docs`).

- [ ] Add minimal custom theming:

  - [ ] Custom CSS (e.g., `docs/book/theme/css/dfps.css`).
  - [ ] Optional logo / favicon.
  - [ ] Adjust color palette to align with web frontend.

### DOCS-HOST-02 – Public hosting pipeline

- [ ] Add a CI job to:

  - [ ] Run `cargo make docs` on main branch.
  - [ ] Publish `docs/book/book/` to a static host (e.g., GitHub Pages, S3, or Netlify).
  - [ ] Expose the resulting URL as `DFPS_DOCS_URL` in deployment configs.

- [ ] Ensure CI fails if `cargo make docs` fails (not silently ignored).

### DOCS-HOST-03 – Frontend integration & UX

- [ ] Update `dfps_web_frontend`:

  - [ ] Confirm `/docs` redirect works correctly when `DFPS_DOCS_URL` is set to the public docs site.
  - [ ] Adjust any existing references to local mdBook ports in runbooks.

- [ ] Add a “Docs” link to the main navigation / footer of the mapping workbench.

### DOCS-HOST-04 – Content tidy-up

- [ ] Review `docs/system-design/**` and `docs/runbook/**` for:

  - [ ] Broken links (especially between FHIR/NCIt directories).
  - [ ] Consistent naming (e.g., FHIR vs clinical/fhir paths).
  - [ ] Duplication between `docs/` and `docs/book/src/` (ensure sync scripts are up-to-date).

- [ ] Update `docs/book/src/index.md` to include:

  - [ ] A short “How to navigate DFPS docs” section.
  - [ ] Links to key runbooks (web, env, makefile, analytics).

---

## INPROGRESS
- _Empty_

---

## REVIEW
- _Empty_

---

## DONE
- _Empty_

---

## Acceptance Criteria

- `cargo make docs` builds a searchable mdBook with DFPS-specific theming.
- A CI pipeline publishes docs to a stable URL after merges to main.
- `/docs` in `dfps_web_frontend` reliably redirects to the hosted documentation.
- Internal links across system-design/runbooks/kanban remain intact.

## Out of Scope

- Multi-language docs.
- Heavy custom JS-based documentation frameworks beyond mdBook.
