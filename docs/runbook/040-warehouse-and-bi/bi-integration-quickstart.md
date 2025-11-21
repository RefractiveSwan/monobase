# BI Integration Quickstart

Use this guide to point a BI tool (Metabase/Superset/SQLite client) at the refractive_swan analytics surface.

## Prerequisites
- `cargo` toolchain
- Optional SQLite file for persistence (otherwise analytics stays in-memory)

## Run the API with analytics enabled
1) Configure warehouse env (SQLite URL recommended for quickstart):
   ```
   export refractive_swan_WAREHOUSE_URL="sqlite://./refractive_swan_analytics.sqlite"
   export refractive_swan_WAREHOUSE_MAX_CONNECTIONS=5
   ```
2) Start the API:
   ```
   cargo run -p refractive_swan_api --bin refractive_swan_api
   ```
   - Without `refractive_swan_WAREHOUSE_URL`, analytics are kept in-memory only.

## Endpoints to wire
- `/analytics/ncit-summary` — aggregated counts by NCIt ID, mapping state, and time bucket.
- `/analytics/cohort?ncit_id=C1234&status=active&date_from=2024-01-01&date_to=2024-12-31` — resolved dim/fact rows for cohort exploration.
- `/metrics/summary` — includes analytics counters (`analytics_requests`, `cohort_queries`, `avg_cohort_size`).

## Connecting BI tools
- Point the BI client at `refractive_swan_WAREHOUSE_URL` (SQLite) and use:
  - `dim_patient`, `dim_encounter`, `dim_code`, `dim_ncit`
  - `fact_service_request`
- Recommended starter view: group `fact_service_request` by `ncit_key` (join `dim_ncit`) and `status` to mirror `/analytics/ncit-summary`.

## Troubleshooting
- Persistence disabled: ensure `refractive_swan_WAREHOUSE_URL` is set before starting the API.
- Empty cohorts: confirm `ordered_at` timestamps are RFC 3339 or `YYYY-MM-DD`.
