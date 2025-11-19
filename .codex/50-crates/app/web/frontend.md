# Crate: lib/app/frontend/web — `dfps_web_frontend`

**Purpose**  
Actix‑Web UI (HTMX + Tailwind) that talks to the backend.

**Env**
- Loads `app.web.frontend` via `dfps_configuration`.
- `AppConfig`:
  - `DFPS_FRONTEND_LISTEN_ADDR` (default `127.0.0.1:8090`)
  - `DFPS_API_BASE_URL` (default `http://127.0.0.1:8080`)
  - `DFPS_API_CLIENT_TIMEOUT_SECS` (default `15`)
  - `DFPS_DOCS_URL` (optional `/docs` redirect)

**Backend client**
- `GET /health` → `HealthResponse`
- `GET /metrics/summary` → `PipelineMetrics`
- `POST /api/map-bundles` → `MapBundlesResponse`
- Friendly `ClientError` → alert text for the UI.

**Routes**
- `GET /` — base page with health + metrics
- `POST /map/paste` — parse JSON from textarea; HTMX fragment swap
- `POST /map/upload` — multipart file read (UTF‑8 JSON only; **max 512 KiB**)
- `GET /docs` — redirect to `DFPS_DOCS_URL` if present, else 404

**UI**
- Results panel with `MappingResult` rows and state chips:
  - AutoMapped / Needs review / No match
- Metrics dashboard from `PipelineMetrics`
- “NoMatch explorer” (SR, code, reason)

**Run**
```bash
cd code
cargo run -p dfps_web_frontend --bin dfps_web_frontend
```

**Tests**
- Route tests w/ Wiremock backend
- Template rendering assertions (metrics + NoMatch)
