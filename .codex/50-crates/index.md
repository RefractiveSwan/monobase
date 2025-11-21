# 50‑crates — Workspace Codex Index

*Path:* `code/.codex/50-crates/index.md`  
This index summarizes every crate documented under `50-crates/` and links to their Codex shards. It’s organized by layer: *Domain*, *App*, and *Platform*.

---

## Quick Nav

- *Domain*
  - [`refractive_swan_core`](domain/core.md)
  - [`refractive_swan_fake_data`](domain/fake_data.md)
  - [`refractive_swan_ingestion`](domain/ingestion.md)
  - [`refractive_swan_mapping`](domain/mapping.md)
  - [`refractive_swan_pipeline`](domain/pipeline.md)
  - [`refractive_swan_terminology`](domain/terminology.md)
- *App*
  - [`refractive_swan_cli`](app/cli.md)
  - *Web*
    - *Backend*
        - [`refractive_swan_api`](app/servers/api.md)
        - [`refractive_swan_datamart`](app/servers/datamart.md)
  - [`refractive_swan_web_frontend`](app/web/frontend.md)
- *Platform*
  - [`refractive_swan_configuration`](platform/configuration.md)
  - [`refractive_swan_observability`](platform/observability.md)
  - [`refractive_swan_test_suite`](platform/test_suite.md)

---

## Big Picture

```raw
                       ┌────────────────────┐
                       │     refractive_swan_core      │
                       └─────────┬──────────┘
                                 │
        ┌────────────────────────┼────────────────────────┐
        │                        │                        │
 ┌──────▼──────┐          ┌──────▼──────┐          ┌──────▼──────────┐
 │ refractive_swan_ingest │          │ refractive_swan_mapping│          │ refractive_swan_terminology│
 └──────┬──────┘          └──────┬──────┘          └─────────────────┘
        │                        │
        └──────────┬─────────────┘
                   ▼
            ┌──────────────┐
            │ refractive_swan_pipeline│
            └──────┬───────┘
                   │
     ┌─────────────┼───────────────┐
     │             │               │
┌────▼────┐  ┌─────▼─────┐   ┌─────▼────────┐
│ refractive_swan_cli│  │  refractive_swan_api │   │ refractive_swan_datamart│
└─────────┘  └─────┬─────┘   └─────┬────────┘
                    │               │
                    │         (analytics dims/facts)
                    │
               ┌────▼──────────────┐
               │ refractive_swan_web_frontend │
               └───────────────────┘

Platform services used across the stack:
- refractive_swan_configuration  (env loading)
- refractive_swan_observability  (logging + metrics)
- refractive_swan_test_suite     (fixtures/assertions/tests)
```

---

## Domain Layer

### [`refractive_swan_core`](domain/core.md)
Canonical domain/FHIR/staging/mapping/value types with `serde` support. Foundation for all other crates.

### [`refractive_swan_fake_data`](domain/fake_data.md)
Deterministic generators (with seeds) for domain entities and minimal FHIR Bundles; used by tests and demos.

### [`refractive_swan_ingestion`](domain/ingestion.md)
FHIR -> staging -> domain normalization + validation. Clear, typed errors and strict/lenient validation modes.

### [`refractive_swan_mapping`](domain/mapping.md)
Deterministic NCIt mapping engine (lexical + mock vector + rules), UMLS xref shortcuts, and summary tallies.

### [`refractive_swan_pipeline`](domain/pipeline.md)
Thin façade that wires *ingestion + mapping* and returns `{ flats, exploded_codes, mapping_results, dim_concepts }`.

### [`refractive_swan_terminology`](domain/terminology.md)
Code‑system registry/normalization and license/source classification; OBO hints for NCIt.

---

## App Layer

### [`refractive_swan_cli`](app/cli.md)
Shell‑friendly tools:
- `map_bundles`: ingest + map Bundles; emits NDJSON records (including `metrics_summary`).
- `map_codes`: map `StgSrCodeExploded` rows; optional explanation output.

### [`refractive_swan_api`](app/servers/api.md)
Axum HTTP gateway:
- `POST /api/map-bundles` (Bundle object/array/NDJSON)
- `GET /metrics/summary`
- `GET /health`
Maintains global `PipelineMetrics`.

### [`refractive_swan_datamart`](app/servers/datamart.md)
Builds a small star schema (Dims + Facts) from `PipelineOutput`, including a `NO_MATCH` sentinel concept.

### [`refractive_swan_web_frontend`](app/web/frontend.md)
Actix + Maud + HTMX UI:
- Paste/upload Bundle -> show `MappingResult` rows
- Metrics dashboard
- “NoMatch explorer”
Proxies to `refractive_swan_api`.

---

## Platform Layer

### [`refractive_swan_configuration`](platform/configuration.md)
Workspace‑aware, namespaced env loader (`.env.<namespace>.<profile>`), strict mode, and root discovery.

### [`refractive_swan_observability`](platform/observability.md)
Shared logging hooks and `PipelineMetrics` counters; emits per‑bundle summaries and warns on `NoMatch`.

### [`refractive_swan_test_suite`](platform/test_suite.md)
Fixtures, assertions, property tests, plus E2E/Integration suites covering ingestion, mapping, datamart, and web API.

---

## Environment Namespaces (via `refractive_swan_configuration`)

| Crate / Component          | Namespace               |
|---------------------------|-------------------------|
| CLI                       | `app.cli`               |
| Web API (backend)         | `app.web.api`           |
| Web Frontend              | `app.web.frontend`      |
| Observability             | `platform.observability`|
| Test Suite                | `platform.test_suite`   |

> Files are resolved as `.env.<namespace>.<profile>` with `profile = refractive_swan_ENV || APP_ENV || "dev"`.

---

## Change Impact Cheatsheet

- *`refractive_swan_core`* -> ripples to *everything*.
- *`refractive_swan_ingestion`* -> affects pipeline, CLI `map_bundles`, API, and tests.
- *`refractive_swan_mapping`* -> affects pipeline, CLI `map_codes`, API, datamart facts, and tests; update thresholds and summaries accordingly.
- *`refractive_swan_pipeline`* -> affects CLI/API outputs and datamart transformation.
- *`refractive_swan_terminology`* -> impacts mapping result metadata (license/source).
- *`refractive_swan_configuration`* -> env filenames/dirs; update app READMEs and CI.
- *`refractive_swan_observability`* -> metrics schema; update API/Frontend dashboards and tests.
- *`refractive_swan_datamart`* -> schema changes require test and consumer updates.
- *`refractive_swan_web_frontend`* ↔ *`refractive_swan_api`* -> keep `MapBundlesResponse` and UI renderers in sync.

---

## Run & Test (quick reference)

```bash
# CLI
cargo run -p refractive_swan_cli --bin map_bundles -- ./bundles.ndjson
cargo run -p refractive_swan_cli --bin map_codes -- --explain --explain-top 5 < codes.ndjson

# Backend API
cargo run -p refractive_swan_api --bin refractive_swan_api

# Frontend
cargo run -p refractive_swan_web_frontend --bin refractive_swan_web_frontend

# Test suite
cargo test -p refractive_swan_test_suite
```
---

## Doc Files (this folder)

- Domain: `domain/*.md`
- App: `app_cli.md`, `web_api.md`, `web_datamart.md`, `web_frontend.md`
- Platform: `platform_configuration.md`, `platform_observability.md`, `test_suite.md`

> When adding a new shard, keep filenames concise and update this index.
