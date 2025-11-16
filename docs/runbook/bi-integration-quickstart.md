# BI Integration Quickstart

Use this guide to point a BI tool (Metabase/Superset/SQLite client) at the DFPS analytics surface.

## Prerequisites
- `cargo` toolchain
- Optional SQLite file for persistence (otherwise analytics stays in-memory)

## Run the API with analytics enabled
1) Configure warehouse env (SQLite URL recommended for quickstart):
   ```
   export DFPS_WAREHOUSE_URL="sqlite://./dfps_analytics.sqlite"
   export DFPS_WAREHOUSE_MAX_CONNECTIONS=5
   ```
2) Start the API:
   ```
   cargo run -p dfps_api --bin dfps_api
   ```
   - Without `DFPS_WAREHOUSE_URL`, analytics are kept in-memory only.

## Endpoints to wire
- `/analytics/ncit-summary` — aggregated counts by NCIt ID, mapping state, and time bucket.
- `/analytics/cohort?ncit_id=C1234&status=active&date_from=2024-01-01&date_to=2024-12-31` — resolved dim/fact rows for cohort exploration.
- `/metrics/summary` — includes analytics counters (`analytics_requests`, `cohort_queries`, `avg_cohort_size`).

## Connecting BI tools
- Point the BI client at `DFPS_WAREHOUSE_URL` (SQLite) and use:
  - `dim_patient`, `dim_encounter`, `dim_code`, `dim_ncit`
  - `fact_service_request`
- Recommended starter view: group `fact_service_request` by `ncit_key` (join `dim_ncit`) and `status` to mirror `/analytics/ncit-summary`.

## Troubleshooting
- Persistence disabled: ensure `DFPS_WAREHOUSE_URL` is set before starting the API.
- Empty cohorts: confirm `ordered_at` timestamps are RFC 3339 or `YYYY-MM-DD`.
