# Crate: lib/app/servers/api — `refractive_swan_api`

**Purpose**  
Axum HTTP API for mapping requests and metrics.

**Env & config**
- Loads `app.web.api` via `refractive_swan_configuration`.
- `ApiServerConfig` (defaults): `refractive_swan_API_HOST=127.0.0.1`, `refractive_swan_API_PORT=8080`.
- `init_logging()` bootstraps `env_logger` once.

**Routes**
- `GET /health` → `{"status":"ok"}` (logs `request_id`)
- `GET /metrics/summary` → `PipelineMetrics`
- `POST /api/map-bundles` → `MapBundlesResponse`
  - Accepts: **Bundle object**, **array**, or **NDJSON**.
  - For each bundle: `bundle_to_mapped_sr` → aggregate `flats`, `exploded_codes`, `mapping_results`, `dim_concepts`.
  - Dedupes concepts by `ncit_id`; updates global `PipelineMetrics`.

**Errors**
- `400 invalid_json`, `422 invalid_fhir`, `500 internal_error` — all include `request_id`.

**Run**
```bash
cd code
cargo run -p refractive_swan_api --bin refractive_swan_api
```

**Notes**
- `parse_bundles` rejects empty/whitespace bodies; auto‑detects NDJSON.
- Warns per `NoMatch` via `refractive_swan_observability::log_no_match`.
