# dfps_web_frontend (Actix UI)

Serves the HTMX/Tailwind pages and proxies to the backend.

## Routes → backend endpoints

| Frontend route | Purpose | Backend call |
| --- | --- | --- |
| `/` | Mapping workbench landing page that drives paste/upload flows plus the health/metrics sidebar. | `GET /health`, `GET /metrics/summary`, `POST /api/map-bundles` |
| `/analytics` | NCIt summary tiles + Cohort explorer. | `GET /analytics/ncit-summary`, `GET /analytics/cohort` |
| `/eval` / `/eval/run` / `/eval/report` | Eval dataset picker and run panel. | `GET /api/eval/datasets`, `GET /api/eval/summary`, `POST /api/eval/run` |
| `/docs` | Redirect to the mdBook / docs host if configured. | n/a (redirects to `DFPS_DOCS_URL`) |

Each HTMX form posts back to these endpoints so the UI stays aligned with the
`dfps_dataplane` (`dfps_api`) contracts.

Env (loaded with `app.web.frontend`):
- `DFPS_FRONTEND_LISTEN_ADDR` (default `127.0.0.1:8090`) – matches `app.web.frontend.listen_addr` in `dfps_configuration` docs.
- `DFPS_API_BASE_URL` (default `http://127.0.0.1:8080`) – should point at the `dfps_api` base URL.
- `DFPS_API_CLIENT_TIMEOUT_SECS` (default `15`) – request timeout in seconds for backend calls.
- `DFPS_DOCS_URL` (optional) – when set (e.g., `http://127.0.0.1:3000`), `/docs` redirects there so you can surface an mdBook server.

Notes:
- Paste/upload flows accept JSON payloads up to 512KiB, mirroring `dfps_cli map_bundles`.
- If the backend cannot list datasets, the eval panel defaults to `gold_pet_ct_small` (documented in `docs/runbook/mapping-eval-quickstart.md`).

## Architecture

- `handlers/` – feature routers (`home`, `mapping`, `analytics`, `eval`, `docs`) that register their own Actix resources so mapping/analytics/eval logic stays localized.
- `client.rs` – canonical backend client that consumes `dfps_contracts` DTOs and exposes user-friendly errors for the UI.
- `view_model.rs` & `views.rs` – shared contexts + HTMX fragments, aligned with `dfps_dataplane` contracts.
- `routes.rs` – tiny aggregator that stitches handler modules together for the server startup path.

Run:
```bash
cd code
cargo run -p dfps_web_frontend --bin dfps_web_frontend
```
