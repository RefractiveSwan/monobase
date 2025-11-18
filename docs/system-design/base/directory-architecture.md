# Directory Architecture

**Path:** `code/docs/system-design/directory-architecture.md`  
**Scope:** Rust workspace under `code/lib`

This document describes how the Rust workspace is organized into clear, discoverable buckets. The goal is to make it obvious:

- where **user-facing** surfaces live,
- where **domain logic** lives, and
- where **cross-cutting platform** concerns live.

---

## High-level layout

```text
code/
  docs/
    runbook/
    kanban/
    book/           # mdBook sources + build output (via cargo make docs)
    
  data/
    makefiles/      # cargo-make task shards (core/docs/apps)

  lib/
    app/
        cli/
        desktop/
        web/
          backend/
            api/
          frontend/

    domain/
        core/
        ingestion/
        mapping/
        pipeline/
        fake_data/

    platform/
        observability/
        test_suite/
````

At a glance:

* `app/` – entrypoints and interfaces facing humans or external callers.
* `domain/` – the problem-space logic and data flow.
* `platform/` – cross-cutting support: observability, testing, infra-style helpers.

---

## Buckets and responsibilities

### 1. `app/` – Application surfaces

**Path:** `code/lib/app`

This is where anything “at the edge” of the system lives. These crates should be thin and mostly delegate into `domain/` crates.

Current structure:

```text
code/lib/app/
  cli/
  desktop/
  web/
    backend/
      api/
    frontend/
```

* `app/cli/`

  * Command-line tools and utilities.
  * Typical responsibilities:

    * parse args / config,
    * call domain services (e.g. mapping pipeline),
    * handle basic IO and exit codes.

* `app/desktop/`

  * Future desktop UI shells, if any (e.g., Tauri/Wry/Electron-bridged UIs).

* `app/web/`

  * Web-facing UI or HTTP-gateway shells (e.g., web dashboards, admin panels).
  * Path: `code/lib/app/web/frontend`.
  * Crate: `dfps_web_frontend`.
  * Actix-web UI that renders a Tailwind/HTMX dashboard for bundle uploads and mapping review.
  * Reads `DFPS_API_BASE_URL` to reach the backend API, `DFPS_FRONTEND_LISTEN_ADDR` for its bind address, and `DFPS_API_CLIENT_TIMEOUT_SECS` for the reqwest client timeout.
  * Exposes an HTML form for paste/upload workflows and calls `/api/map-bundles`, `/metrics/summary`, and `/health` via an internal API client.
  * Provides real-time mapping summaries, a metrics dashboard fed by `/metrics/summary`, and a NoMatch explorer so reviewers can triage unmapped codes.
  * When `DFPS_DOCS_URL` is set (see `.env.app.web.frontend.<profile>`), `/docs` issues a redirect to the mdBook server so documentation is visible alongside the UI.
  * `src/views.rs` holds the Maud templates, `src/routes.rs` hosts the paste/upload endpoints, and `src/client.rs` houses the reqwest wrapper that talks to the backend. For a full end-to-end runbook, see `docs/runbook/web-quickstart.md`.

* `web/backend/api/`

  * Crate: `dfps_api`.
  * HTTP API gateway that exposes the DFPS pipeline over `/api/map-bundles`, `/health`, and `/metrics/summary`.
  * Delegates ingestion/mapping work to `dfps_pipeline` and emits metrics via `dfps_observability`.
  * Loads `.env.app.web.api.<profile>` via `dfps_configuration` so dev/test/prod profiles share the same config story as the frontend UI.

**Principle:**
No heavy business logic should live here. If you find complex logic in `app/`, move it into a `domain/` crate and import it.

---

### 2. `domain/` – Core domain logic and flows

**Path:** `code/lib/domain`

These crates represent what the system **does**, independent of UI or specific deployment details.

Current structure:

```text
code/lib/domain/
  core/
  ingestion/
  mapping/
  pipeline/
  fake_data/
```

#### `core/` – Core models and kernel

* Crate: `dfps_core` (intended)
* Responsibilities:

  * Fundamental domain models (graphs, partitions, mappings, etc.).
  * Kernel / algorithm interfaces (ports, traits).
  * Shared value objects and types used across other domain crates.
* Examples:

  * `Graph`, `Partition`, `CodeElement`, `MappingResult`, etc.
  * Port traits for algorithms or IO (e.g. `MappingEnginePort`, `GraphBuilderPort`).
* Internal layout (REFR-04):

  * `primitives/` (IDs), `clinical/` (patient/encounter/order), `interop/` (FHIR + staging landing rows), `semantics/` (mapping concepts/results).
  * Bridges that consume another super-domain live under the **consumer** module’s `bridge/` folder (e.g., `semantics/mapping/bridge/from_staging.rs`).
  * `lib.rs` keeps back-compat re-exports so callers can still use `dfps_core::{patient, order, staging, mapping, value, fhir}` module paths.

#### `ingestion/` – Getting data in

* Crate: `dfps_ingestion` (intended)
* Responsibilities:

  * Adapters that ingest data from external formats into `core` models.
  * FHIR / raw data ingestion, parsers, connectors.
  * Schema/validation for incoming payloads.
* Examples:

  * FHIR ServiceRequest -> internal “procedure request” models.
  * NCIt / UMLS loaders that emit `CodeElement` sets.

#### `mapping/` – Semantic mapping engine

* Crate: `dfps_mapping`
* Responsibilities:

  * Mapping logic between code systems (CPT, HCPCS, SNOMED, NCIt, etc.).
  * Lexical and vector rankers, rule-based re-rankers.
  * Mapping pipelines that operate on already ingested domain objects.
* Examples:

  * `map_staging_codes` pipeline.
  * Rankers combining FAISS/TF-IDF, Jaro-Winkler, and clinical rules.
  * Default mapping engine built from NCIt/UMLS loaders.

#### `pipeline/` – Orchestration and workflows

* Crate: `dfps_pipeline`
* Responsibilities:

  * Compose ingestion + mapping + downstream steps into end-to-end jobs.
  * Define and orchestrate discrete pipeline stages.
  * Provide reusable “flows” that `app/` crates can invoke.
* Examples:

  * “Ingest FHIR ServiceRequests -> normalize -> map to NCIt -> emit structured results.”
  * Job definitions that can be scheduled / invoked from CLI or web.

#### `fake_data/` – Domain-aware generators

* Crate: `dfps_fake_data`
* Responsibilities:

  * Generate realistic fake data that mirrors domain models.
  * Provide fixtures for FHIR-like payloads, NCIt-like vocab sets, graphs, etc.
* Examples:

  * Random FHIR ServiceRequests with plausible combinations of fields.
  * Fake NCIt concept hierarchies for development and tests.
  * Graph generators to test community detection / mapping flows.

**Principle:**
If it encodes business rules, semantics, or domain invariants, it goes under `domain/`.

---

### 3. `platform/` – Cross-cutting support

**Path:** `code/lib/platform`

These are capabilities used across the app and domain, but not specific to the business problem.

Current structure:

```text
code/lib/platform/
  observability/
  test_suite/
```

#### `observability/` – Logging and metrics

* Crate: `dfps_observability`
* Responsibilities:

  * Logging configuration and helpers.
  * Metrics/tracing span helpers.
  * Potential integration with external observability stacks.
* Examples:

  * `init_logging()` / `init_tracing()` functions.
  * Common macros/wrappers for structured logs and metrics.

#### `test_suite/` – Shared test harness

* Crate: `dfps_test_suite`
* Responsibilities:

  * Shared test utilities and assertions.
  * Reusable fixtures and golden data loaders.
  * Property-based test primitives and harness utilities.
* Examples:

  * Golden fixture loaders for regression tests.
  * Helpers to construct fake graphs / mappings in tests.
  * Property-testing combinators reused across crates.
  * `.env.platform.observability.<profile>` drives workspace-wide logging defaults (e.g., `RUST_LOG`) so all binaries/tests emit consistent telemetry.

**Principle:**
If it’s used by many crates and not fundamentally a domain concept, it likely belongs under `platform/`.

---

## Cargo workspace example

Root `code/Cargo.toml` (illustrative):

```toml
[workspace]
members = [
  # app
  "lib/app/frontend/cli",
  "lib/app/frontend/desktop",
  "lib/app/frontend/web",

  # domain
  "lib/domain/core",
  "lib/domain/ingestion",
  "lib/domain/mapping",
  "lib/domain/pipeline",
  "lib/domain/fake_data",

  # platform
  "lib/platform/observability",
  "lib/platform/test_suite",
]
```

Keep crate names stable (e.g. `dfps_core`, `dfps_mapping`); only the paths change.

---

## Old -> new path mapping

For historical reference:

```text
# Before
code/lib/core           -> code/lib/domain/core
code/lib/fake_data      -> code/lib/domain/fake_data
code/lib/ingestion      -> code/lib/domain/ingestion
code/lib/mapping        -> code/lib/domain/mapping
code/lib/pipeline       -> code/lib/domain/pipeline
code/lib/frontend       -> code/lib/app/frontend
code/lib/observability  -> code/lib/platform/observability
code/lib/test_suite     -> code/lib/platform/test_suite
```

---

## Guidelines for adding new crates

When introducing a new crate, choose its home based on intent:

* Put it in **`app/`** if:

  * It exposes a UI, CLI, HTTP API, or other external interface.
  * It mostly delegates to domain services.

* Put it in **`domain/`** if:

  * It expresses domain concepts, rules, or workflows.
  * It would still make sense if you replaced the UI/infra.

* Put it in **`platform/`** if:

  * It provides a reusable capability (metrics, config, IO helpers, etc.).
  * It’s not specific to the clinical/mapping domain and could be reused in another project.

Naming pattern (recommended):

* `code/lib/domain/<bounded_context>/`
* `code/lib/app/<surface>/`
* `code/lib/platform/<capability>/`

This keeps the workspace readable even as the number of crates grows.

---

## Developer tooling and docs

* **Task runner:** `cargo make` (defined in `Makefile.toml` + `data/makefiles/` shards) standardizes `build`, `test`, `docs`, `web`, etc. Install via `cargo install cargo-make`. From the workspace root, run `cargo make help` to list available tasks.
* **Documentation book:** `docs/book/` hosts the mdBook sources. Use `cargo make docs-sync` to sync runbooks/kanban into the book and `cargo make docs` to build HTML under `docs/book/book/`. `cargo make docs-serve` runs `mdbook serve`.
* **Environment files:** All `.env.*` files live under `data/environment/` (ignored by git). Copy the `.example` templates when setting up new profiles. The loader (`dfps_configuration`) reads `.env.<namespace>.<profile>` based on `DFPS_ENV`.

For operational guidance and run instructions, refer to the runbooks under `docs/runbook/` (and their mdBook mirror).
