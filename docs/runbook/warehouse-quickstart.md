# Warehouse Quickstart

This guide shows how to run the datamart migrations and load pipeline output into SQLite.

## Prerequisites
- Rust toolchain installed
- `cargo` available in your PATH

## 1) Configure env
Set the warehouse env vars (see `data/environment/.env.domain.fhir_validation.example` for pattern):

```bash
export DFPS_WAREHOUSE_URL=sqlite::memory:
export DFPS_WAREHOUSE_SCHEMA=
export DFPS_WAREHOUSE_MAX_CONNECTIONS=5
```

## 2) Run migrations

```bash
cargo test -p dfps_datamart -- --nocapture
# or programmatically:
cargo test -p dfps_test_suite --test integration_tests warehouse
```

## 3) Load pipeline output

```bash
# Using existing PipelineOutput NDJSON
target/debug/load_datamart --input ./pipeline_output.ndjson --input-kind pipeline

# Or map Bundles on the fly (NDJSON of FHIR Bundles)
target/debug/load_datamart --input ./bundles.ndjson --input-kind bundle
```

The CLI applies migrations automatically, then prints a `load_summary` JSON line showing dim/fact counts.

## 4) Inspect the DB
For SQLite, use `sqlite3`:

```bash
sqlite3 :memory:
.tables
select count(*) from fact_service_request;
```

## 5) Analytics API persistence
- Set `DFPS_WAREHOUSE_URL` before running `dfps_api` to persist analytics cohorts and NCIt summaries.
- The API writes to `dim_*` + `fact_service_request` via `dfps_datamart` and exposes:
  - `/analytics/ncit-summary` (aggregated counts)
  - `/analytics/cohort` (filtered fact rows)
- See `docs/runbook/bi-integration-quickstart.md` for BI tool wiring tips.
