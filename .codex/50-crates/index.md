# 50‑crates — Workspace Codex Index

*Path:* `code/.codex/50-crates/index.md`  
This index summarizes every crate documented under `50-crates/` and links to their Codex shards. It’s organized by layer: *Domain*, *App*, and *Platform*.

---

## Quick Nav

- *Domain*
  - [`dfps_core`](domain/core.md)
  - [`dfps_fake_data`](domain/fake_data.md)
  - [`dfps_ingestion`](domain/ingestion.md)
  - [`dfps_mapping`](domain/mapping.md)
  - [`dfps_pipeline`](domain/pipeline.md)
  - [`dfps_terminology`](domain/terminology.md)
- *App*
  - [`dfps_cli`](app/cli.md)
  - *Web*
    - *Backend*
        - [`dfps_api`](app/servers/api.md)
        - [`dfps_datamart`](app/servers/datamart.md)
  - [`dfps_web_frontend`](app/web/frontend.md)
- *Platform*
  - [`dfps_configuration`](platform/configuration.md)
  - [`dfps_observability`](platform/observability.md)
  - [`dfps_test_suite`](platform/test_suite.md)

---

## Big Picture

```raw
                       ┌────────────────────┐
                       │     dfps_core      │
                       └─────────┬──────────┘
                                 │
        ┌────────────────────────┼────────────────────────┐
        │                        │                        │
 ┌──────▼──────┐          ┌──────▼──────┐          ┌──────▼──────────┐
 │ dfps_ingest │          │ dfps_mapping│          │ dfps_terminology│
 └──────┬──────┘          └──────┬──────┘          └─────────────────┘
        │                        │
        └──────────┬─────────────┘
                   ▼
            ┌──────────────┐
            │ dfps_pipeline│
            └──────┬───────┘
                   │
     ┌─────────────┼───────────────┐
     │             │               │
┌────▼────┐  ┌─────▼─────┐   ┌─────▼────────┐
│ dfps_cli│  │  dfps_api │   │ dfps_datamart│
└─────────┘  └─────┬─────┘   └─────┬────────┘
                    │               │
                    │         (analytics dims/facts)
                    │
               ┌────▼──────────────┐
               │ dfps_web_frontend │
               └───────────────────┘

Platform services used across the stack:
- dfps_configuration  (env loading)
- dfps_observability  (logging + metrics)
- dfps_test_suite     (fixtures/assertions/tests)
```

---

## Domain Layer

### [`dfps_core`](domain/core.md)
Canonical domain/FHIR/staging/mapping/value types with `serde` support. Foundation for all other crates.

### [`dfps_fake_data`](domain/fake_data.md)
Deterministic generators (with seeds) for domain entities and minimal FHIR Bundles; used by tests and demos.

### [`dfps_ingestion`](domain/ingestion.md)
FHIR -> staging -> domain normalization + validation. Clear, typed errors and strict/lenient validation modes.

### [`dfps_mapping`](domain/mapping.md)
Deterministic NCIt mapping engine (lexical + mock vector + rules), UMLS xref shortcuts, and summary tallies.

### [`dfps_pipeline`](domain/pipeline.md)
Thin façade that wires *ingestion + mapping* and returns `{ flats, exploded_codes, mapping_results, dim_concepts }`.

### [`dfps_terminology`](domain/terminology.md)
Code‑system registry/normalization and license/source classification; OBO hints for NCIt.

---

## App Layer

### [`dfps_cli`](app/cli.md)
Shell‑friendly tools:
- `map_bundles`: ingest + map Bundles; emits NDJSON records (including `metrics_summary`).
- `map_codes`: map `StgSrCodeExploded` rows; optional explanation output.

### [`dfps_api`](app/servers/api.md)
Axum HTTP gateway:
- `POST /api/map-bundles` (Bundle object/array/NDJSON)
- `GET /metrics/summary`
- `GET /health`
Maintains global `PipelineMetrics`.

### [`dfps_datamart`](app/servers/datamart.md)
Builds a small star schema (Dims + Facts) from `PipelineOutput`, including a `NO_MATCH` sentinel concept.

### [`dfps_web_frontend`](app/web/frontend.md)
Actix + Maud + HTMX UI:
- Paste/upload Bundle -> show `MappingResult` rows
- Metrics dashboard
- “NoMatch explorer”
Proxies to `dfps_api`.

---

## Platform Layer

### [`dfps_configuration`](platform/configuration.md)
Workspace‑aware, namespaced env loader (`.env.<namespace>.<profile>`), strict mode, and root discovery.

### [`dfps_observability`](platform/observability.md)
Shared logging hooks and `PipelineMetrics` counters; emits per‑bundle summaries and warns on `NoMatch`.

### [`dfps_test_suite`](platform/test_suite.md)
Fixtures, assertions, property tests, plus E2E/Integration suites covering ingestion, mapping, datamart, and web API.

---

## Environment Namespaces (via `dfps_configuration`)

| Crate / Component          | Namespace               |
|---------------------------|-------------------------|
| CLI                       | `app.cli`               |
| Web API (backend)         | `app.web.api`           |
| Web Frontend              | `app.web.frontend`      |
| Observability             | `platform.observability`|
| Test Suite                | `platform.test_suite`   |

> Files are resolved as `.env.<namespace>.<profile>` with `profile = DFPS_ENV || APP_ENV || "dev"`.

---

## Change Impact Cheatsheet

- *`dfps_core`* -> ripples to *everything*.
- *`dfps_ingestion`* -> affects pipeline, CLI `map_bundles`, API, and tests.
- *`dfps_mapping`* -> affects pipeline, CLI `map_codes`, API, datamart facts, and tests; update thresholds and summaries accordingly.
- *`dfps_pipeline`* -> affects CLI/API outputs and datamart transformation.
- *`dfps_terminology`* -> impacts mapping result metadata (license/source).
- *`dfps_configuration`* -> env filenames/dirs; update app READMEs and CI.
- *`dfps_observability`* -> metrics schema; update API/Frontend dashboards and tests.
- *`dfps_datamart`* -> schema changes require test and consumer updates.
- *`dfps_web_frontend`* ↔ *`dfps_api`* -> keep `MapBundlesResponse` and UI renderers in sync.

---

## Run & Test (quick reference)

```bash
# CLI
cargo run -p dfps_cli --bin map_bundles -- ./bundles.ndjson
cargo run -p dfps_cli --bin map_codes -- --explain --explain-top 5 < codes.ndjson

# Backend API
cargo run -p dfps_api --bin dfps_api

# Frontend
cargo run -p dfps_web_frontend --bin dfps_web_frontend

# Test suite
cargo test -p dfps_test_suite
```
---

## Doc Files (this folder)

- Domain: `domain/*.md`
- App: `app_cli.md`, `web_api.md`, `web_datamart.md`, `web_frontend.md`
- Platform: `platform_configuration.md`, `platform_observability.md`, `test_suite.md`

> When adding a new shard, keep filenames concise and update this index.
