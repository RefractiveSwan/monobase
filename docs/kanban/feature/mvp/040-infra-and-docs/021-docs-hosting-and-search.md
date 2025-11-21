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

  - [ ] Custom CSS (e.g., `docs/book/theme/css/refractive_swan.css`).
  - [ ] Optional logo / favicon.
  - [ ] Adjust color palette to align with web frontend.

### DOCS-HOST-02 – Public hosting pipeline

- [ ] Add a CI job to:

  - [ ] Run `cargo make docs` on main branch.
  - [ ] Publish `docs/book/book/` to a static host (GitHub Pages).
  - [ ] Expose the resulting URL as `refractive_swan_DOCS_URL` in deployment configs.

- [ ] Ensure CI fails if `cargo make docs` fails (not silently ignored).

### DOCS-HOST-03 – Frontend integration & UX

- [ ] Update `refractive_swan_web_frontend`:

  - [ ] Confirm `/docs` redirect works correctly when `refractive_swan_DOCS_URL` is set to the public docs site.
  - [ ] Adjust any existing references to local mdBook ports in runbooks.

- [ ] Add a “Docs” link to the main navigation / footer of the mapping workbench.

### DOCS-HOST-04 – Content tidy-up

- [ ] Review `docs/system-design/**` and `docs/runbook/**` for:

  - [ ] Broken links (especially between FHIR/NCIt directories).
  - [ ] Consistent naming (e.g., FHIR vs clinical/fhir paths).
  - [ ] Duplication between `docs/` and `docs/book/src/` (ensure sync scripts are up-to-date).

- [ ] Update `docs/book/src/index.md` to include:

  - [ ] A short “How to navigate refractive_swan docs” section.
  - [ ] Links to key runbooks (web, env, makefile, analytics).

### DOCS-HOST-05 – Crate inventory & directory-level docs (lib/app, lib/domain, lib/platform)

**Goal:** Every subdirectory under `lib/app`, `lib/domain`, and `lib/platform` is documented, and any docs-hosting/search logic is in the *right* crate with a clear boundary.

#### lib/app crates

- [ ] For each of the following crates, ensure crate-level docs and docs-related logic are clean:

  - [ ] `lib/app/frontend/cli` (`refractive_swan_cli`)
  - [ ] `lib/app/servers/api` (`refractive_swan_api`)
  - [ ] `lib/app/servers/datamart` (`refractive_swan_datamart`)
  - [ ] `lib/app/frontend/web` (`refractive_swan_web_frontend`)
  - [ ] `lib/domain/core` (`refractive_swan_core`)
  - [ ] `lib/domain/meta/evaluation` (`refractive_swan_eval`)
  - [ ] `lib/domain/meta/evaluation::fake_data` (`refractive_swan_eval::fake_data`)
  - [ ] `lib/domain/ingestion::profiles` (embedded FHIR profiles)
  - [ ] `lib/domain/ingestion` (`refractive_swan_ingestion`)
  - [ ] `lib/domain/mapping` (`refractive_swan_mapping`)
  - [ ] Terminology `obo_graph` module (`lib/domain/ontologies/terminology`)
  - [ ] `lib/domain/meta/pipeline` (`refractive_swan_pipeline`)
  - [ ] `lib/domain/ontologies/terminology` (`refractive_swan_terminology`)
  - [ ] `lib/platform/compliance` (`refractive_swan_compliance`)
  - [ ] `lib/platform/configuration` (`refractive_swan_configuration`)
  - [ ] `lib/platform/observability` (`refractive_swan_observability`)
  - [ ] `lib/platform/test_suite` (`refractive_swan_test_suite`)
  - [ ] `lib/app/servers/vector_store` (`refractive_swan_vector_store`)

  For each crate above:

  - [ ] Ensure a `README.md` exists at the crate root that:

    - [ ] States the crate’s responsibility (CLI, API, datamart, web, mapping engine, terminology, eval, etc.)
    - [ ] Describes how (if at all) the crate participates in docs hosting/search (e.g., `/docs` route, CLI doc commands, links).
    - [ ] Points to relevant docs: runbooks and system-design sections that mention this crate.
      - [ ] Which docs reference this crate (system-design diagrams, runbooks, Kanban epics).
    - [ ] Any feature flags that are important for docs examples (`eval-advanced`, `profile_validation`, `obo-graph`, `backend-pgvector`).
    - [ ] Where docs mention domain crates, confirm names and module paths match the latest code (e.g., `refractive_swan_mapping` vs `refractive_swan_terminology` responsibilities).

  - [ ] Confirm docs-related configuration comes only from `refractive_swan_configuration::DocsConfig` (no direct `std::env::var("refractive_swan_DOCS_URL")` scattered in code).
  - [ ] Remove or refactor any ad-hoc docs URLs or ports; wire through `DocsConfig` instead.
  - [ ] Ensure the crate’s binaries exposed in docs (e.g., `map_bundles`, `map_codes`, `eval_mapping`, `validate_fhir`, `load_datamart`, `build_vector_index`) match actual `Cargo.toml` and `src/bin/**` names.

  
  - [ ] In `refractive_swan_configuration`:
    - [ ] Implement a `DocsConfig` (or equivalent) capturing `refractive_swan_DOCS_URL` and docs/search feature flags.
    - [ ] Document these keys in the crate README and ensure they are the **only** source of truth for docs env.
  - [ ] In `refractive_swan_observability`:
    - [ ] Define and document metrics for docs usage (e.g., `docs_view_total`, `docs_search_query_total`, `docs_redirect_error_total`).
    - [ ] Ensure no ad-hoc metrics are defined elsewhere for the same purpose.
  - [ ] In `refractive_swan_test_suite`:
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

- `cargo make docs` builds a searchable mdBook with refractive_swan-specific theming.
- A CI pipeline publishes docs to a stable URL after merges to main.
- `/docs` in `refractive_swan_web_frontend` reliably redirects to the hosted documentation.
- Internal links across system-design/runbooks/kanban remain intact.

## Out of Scope

- Multi-language docs.
- Heavy custom JS-based documentation frameworks beyond mdBook.
