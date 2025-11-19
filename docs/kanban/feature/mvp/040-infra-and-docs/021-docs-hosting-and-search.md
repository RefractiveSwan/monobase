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
  - [ ] Publish `docs/book/book/` to a static host (GitHub Pages).
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

### DOCS-HOST-05 – Crate inventory & directory-level docs (lib/app, lib/domain, lib/platform)

**Goal:** Every subdirectory under `lib/app`, `lib/domain`, and `lib/platform` is documented, and any docs-hosting/search logic is in the *right* crate with a clear boundary.

#### lib/app crates

- [ ] For each of the following crates, ensure crate-level docs and docs-related logic are clean:

  - [ ] `lib/app/cli` (`dfps_cli`)
  - [ ] `lib/app/web/backend/api` (`dfps_api`)
  - [ ] `lib/app/web/backend/datamart` (`dfps_datamart`)
  - [ ] `lib/app/web/frontend` (`dfps_web_frontend`)
  - [ ] `lib/domain/core` (`dfps_core`)
  - [ ] `lib/domain/eval` (`dfps_eval`)
  - [ ] `lib/domain/eval::fake_data` (`dfps_eval::fake_data`)
  - [ ] `lib/domain/ingestion::profiles` (embedded FHIR profiles)
  - [ ] `lib/domain/ingestion` (`dfps_ingestion`)
  - [ ] `lib/domain/mapping` (`dfps_mapping`)
  - [ ] Terminology `obo_graph` module (`lib/domain/ontologies/terminology`)
  - [ ] `lib/domain/pipeline` (`dfps_pipeline`)
  - [ ] `lib/domain/ontologies/terminology` (`dfps_terminology`)
  - [ ] `lib/platform/compliance` (`dfps_compliance`)
  - [ ] `lib/platform/configuration` (`dfps_configuration`)
  - [ ] `lib/platform/observability` (`dfps_observability`)
  - [ ] `lib/platform/test_suite` (`dfps_test_suite`)
  - [ ] `lib/platform/vector_store` (`dfps_vector_store`)

  For each crate above:

  - [ ] Ensure a `README.md` exists at the crate root that:

    - [ ] States the crate’s responsibility (CLI, API, datamart, web, mapping engine, terminology, eval, etc.)
    - [ ] Describes how (if at all) the crate participates in docs hosting/search (e.g., `/docs` route, CLI doc commands, links).
    - [ ] Points to relevant docs: runbooks and system-design sections that mention this crate.
      - [ ] Which docs reference this crate (system-design diagrams, runbooks, Kanban epics).
    - [ ] Any feature flags that are important for docs examples (`eval-advanced`, `profile_validation`, `obo-graph`, `backend-pgvector`).
    - [ ] Where docs mention domain crates, confirm names and module paths match the latest code (e.g., `dfps_mapping` vs `dfps_terminology` responsibilities).

  - [ ] Confirm docs-related configuration comes only from `dfps_configuration::DocsConfig` (no direct `std::env::var("DFPS_DOCS_URL")` scattered in code).
  - [ ] Remove or refactor any ad-hoc docs URLs or ports; wire through `DocsConfig` instead.
  - [ ] Ensure the crate’s binaries exposed in docs (e.g., `map_bundles`, `map_codes`, `eval_mapping`, `validate_fhir`, `load_datamart`, `build_vector_index`) match actual `Cargo.toml` and `src/bin/**` names.

  
  - [ ] In `dfps_configuration`:
    - [ ] Implement a `DocsConfig` (or equivalent) capturing `DFPS_DOCS_URL` and docs/search feature flags.
    - [ ] Document these keys in the crate README and ensure they are the **only** source of truth for docs env.
  - [ ] In `dfps_observability`:
    - [ ] Define and document metrics for docs usage (e.g., `docs_view_total`, `docs_search_query_total`, `docs_redirect_error_total`).
    - [ ] Ensure no ad-hoc metrics are defined elsewhere for the same purpose.
  - [ ] In `dfps_test_suite`:
    - [ ] Group any `/docs`-related tests in a clearly named module (e.g., `tests/docs_integration.rs` or a dedicated test submodule).
    - [ ] Ensure tests cover: valid `/docs` redirects, missing/disabled docs behavior, and basic sanity for the hosted docs URL.


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
