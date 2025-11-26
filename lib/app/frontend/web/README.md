# refractive_swan_web_frontend (Actix UI)

Serves the HTMX/Tailwind pages and proxies to the backend.

## Routes → backend endpoints

| Frontend route | Purpose | Backend call |
| --- | --- | --- |
| `/` | Mapping workbench landing page that drives paste/upload flows plus the health/metrics sidebar. | `GET /health`, `GET /metrics/summary`, `POST /api/map-bundles` |
| `/analytics` | NCIt summary tiles + Cohort explorer. | `GET /analytics/ncit-summary`, `GET /analytics/cohort` |
| `/eval` / `/eval/run` / `/eval/report` | Eval dataset picker and run panel. | `GET /api/eval/datasets`, `GET /api/eval/summary`, `POST /api/eval/run` |
| `/docs` | Redirect to the mdBook / docs host if configured. | n/a (redirects to `refractive_swan_DOCS_URL`) |
| `/ui/components` | Storybook-like preview of the shared cards/buttons/badges for visual QA. | n/a (render-only) |
| `/observability` | Vector/compliance/dataset health dashboard + live log panel. | `GET /metrics/summary`, `GET /api/eval/datasets` |
| `/map/template` | HTMX fragment that refills the paste textarea with curated Bundle templates. | n/a (render-only) |
| `/map/history/{id}` | Replay a cached mapping run (used by the upload-history panel). | n/a (serves cached HTML) |
| `/map/download/latest` | Download the latest mapping results as NDJSON. | n/a (serves cached results) |
| `/analytics/summary/fragment` | HTMX endpoint that re-renders the analytics summary when filters change. | `GET /analytics/ncit-summary` (filtered client-side) |
| `/logs/latest` | Live log fragment (polled by the Observability page). | n/a (serves cached log entries) |
| `/analytics/cohort/export` | Export the current cohort query as CSV (filters preserved via query params). | `GET /analytics/cohort` (same filters) |
| `/map/history/{id}` | HTMX endpoint that replays a cached mapping run from the upload history panel. | n/a (serves cached mapping output) |

Each HTMX form posts back to these endpoints so the UI stays aligned with the
`refractive_swan_dataplane` (`refractive_swan_api`) contracts.

Env (loaded with `app.web.frontend`):
- `refractive_swan_FRONTEND_LISTEN_ADDR` (default `127.0.0.1:8090`) – matches `app.web.frontend.listen_addr` in `refractive_swan_configuration` docs.
- `refractive_swan_API_BASE_URL` (default `http://127.0.0.1:8080`) – should point at the `refractive_swan_api` base URL.
- `refractive_swan_API_CLIENT_TIMEOUT_SECS` (default `15`) – request timeout in seconds for backend calls.
- `refractive_swan_DOCS_URL` (optional) – when set (e.g., `http://127.0.0.1:3000`), `/docs` redirects there so you can surface an mdBook server.
- `refractive_swan_GITHUB_URL` (optional) – controls the GitHub link shown in the navbar/footer (`https://github.com/RefractiveSwan/monobase` by default).

Notes:
- Paste/upload flows accept JSON payloads up to 512KiB, mirroring `refractive_swan_cli map_bundles`.
- If the backend cannot list datasets, the eval panel defaults to `gold_pet_ct_small` (documented in `docs/runbook/mapping-eval-quickstart.md`).
- Mapping workbench now ships with sample Bundle templates (PET/CT, oncology, blank), a remediation-oriented NoMatch explorer, an upload history panel, and an NDJSON download button for the latest run.
- Observability page highlights vector usage, compliance status, dataset store health, and a live NoMatch/API error log. Analytics summary also supports HTMX-powered filters (state + time range) with inline charts, and the cohort explorer exposes CSV exports.
- Workbench pages now surface inline validation summaries (powered by the API’s validation reports) and an upload history panel that can replay the last few mapping jobs without re-submitting Bundles.

## Architecture

- `handlers/` – feature routers (`home`, `mapping`, `analytics`, `eval`, `docs`, `ui`) that register their own Actix resources so mapping/analytics/eval logic stays localized.
- `client.rs` – canonical backend client that consumes `refractive_swan_contracts` DTOs and exposes user-friendly errors for the UI.
- `view_model.rs` & `views/` – shared contexts + HTMX fragments, aligned with `refractive_swan_dataplane` contracts (see `src/views/README.md` for the layout/partials/components map).
- `routes.rs` – tiny aggregator that stitches handler modules together for the server startup path.

Run:
```bash
cd code
cargo run -p refractive_swan_web_frontend --bin refractive_swan_web_frontend
```
