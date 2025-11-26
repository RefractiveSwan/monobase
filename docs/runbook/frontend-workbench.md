# Frontend Workbench Runbook

The HTMX + Maud workbench under `lib/app/frontend/web` exposes the same workflows the CLI/API ship today: bundle mapping, analytics pivots, the observability dashboard, and the new Evaluation Control Center. This runbook gathers everything operators need to wire it up locally or in staging.

## Setup

1. **Config** – copy `.env.frontend.example` into `.env.frontend` or export the following variables:
   - `refractive_swan_FRONTEND_LISTEN_ADDR` – default `127.0.0.1:8090`.
   - `refractive_swan_API_BASE_URL` – defaults to `http://127.0.0.1:8080`; point it at the running API.
   - `refractive_swan_API_CLIENT_TIMEOUT_SECS` – override for slow tunnels.
   - `refractive_swan_DOCS_URL`, `refractive_swan_GITHUB_URL` – render links in the navigation/footer.
   - Feature flags: `refractive_swan_VECTOR_ENABLED`, `refractive_swan_VECTOR_BACKEND`, `refractive_swan_VECTOR_NAMESPACE` toggle vector-powered mapping. `refractive_swan_EVAL_DATA_ROOT` supplies dataset manifests for the eval catalog.
2. **Run** – from `code/`, execute `cargo run -p refractive_swan_web_frontend --bin refractive_swan_web_frontend`. The app sources bundle/eval data from the API, so keep `refractive_swan_api` running alongside.
3. **Smoke test** – visit `http://127.0.0.1:8090` and submit the sample bundle template. Mapping results, validation summary, and upload history should refresh in under 3 seconds.

## Feature surfaces

| Route | Purpose | Backing endpoints |
| --- | --- | --- |
| `/map` | Mapping forms, upload history pane, validation summary, NoMatch explorer. | `POST /api/map-bundles`, `GET /analytics/cohort`, `GET /analytics/ncit-summary` |
| `/observability` | Vector + compliance cards, dataset health, live NoMatch log. | `GET /metrics/summary`, `/api/eval/datasets`, `/logs/latest` |
| `/eval` | Dataset catalog, run scheduler, calibration and comparison panels. | `/api/eval/datasets`, `/api/eval/run`, `/api/eval/summary` |
| `/admin/datasets` | Dataset store admin (list/enable/disable/upload), fixture download links, regression jobs, datamart reset + maintenance console. | Local `refractive_swan_eval::DatasetStore`, `refractive_swan_test_suite::ping()`, `refractive_swan_WAREHOUSE_URL` |
| `/mesh` | Mesh readiness preview: node capabilities, stubbed EvalDataset/Analytics job rows, governance fallback messaging. | `dfps_mesh_dto::{MeshNodeId, NodeCapabilities}` (static mock until mesh_node endpoints ship) |

## HTMX fragments & tips

- Keep expensive fragments behind polling divs (`hx-get="…" hx-trigger="load, every Ns"`). The eval job queue (`/eval/jobs`) now broadcasts queue + history without repainting the rest of the page.
- Use `hx-swap-oob="innerHTML"` sparingly—prefer dedicated fragment endpoints like `/eval/calibration` and `/eval/compare` to keep markup small and predictable.
- When wiring new forms, set their `hx-target` explicitly so flash messages or panels don’t accidentally replace the wrong region. The eval scheduler form targets `#eval-jobs-panel` so history updates atomically.

## Running the UI smoke tests

1. `cd tools/ui-smoke-tests && npm install` (installs Playwright locally).
2. Export `FRONTEND_BASE_URL` if the UI server is not running on `http://127.0.0.1:8090`.
3. Run `npm test` to execute the smoke scenarios: mapping workbench, analytics filters, and the eval queue page.

The Playwright spec asserts page structure and key HTMX fragments so we are warned before regressions reach release notes.

## Screenshot capture workflow

Screenshots that feed the mdBook chapter (`docs/book/src/frontend/command-center.md`) should be generated from a clean `cargo run` session:

1. Run the frontend + API.
2. Use the Playwright inspector (`npm run codegen -- --load-storage=state.json`) to capture deterministic viewport snapshots of `/map`, `/observability`, and `/eval`.
3. Export PNG/SVG assets into `docs/book/src/static/frontend/` and commit them with the doc updates so `cargo make docs` can embed them.

## Troubleshooting

| Symptom | Likely fix |
| --- | --- |
| Eval catalog empty | Ensure `refractive_swan_EVAL_DATA_ROOT` points to the manifests under `lib/domain/meta/evaluation/data/eval`. New `tier` metadata is required. |
| Queue stays in `queued` | Backend `/api/eval/run` logs will spell out dataset or checksum errors; the frontend also emits log entries via the observability page. |
| Upload history blank | The history cache lives in `AppState`. If you restart the frontend, seed history by replaying one of the sample bundles via the template picker. |
| Dataset upload fails | Confirm the manifest `name` contains only alphanumeric/`_`/`-` characters and that both the manifest JSON + NDJSON file fields are populated. |

## Domain & storage surfaces

- **Observability:** Terminology insights, compliance tiers, vector metrics, and ingestion validation stats now render directly from `refractive_swan_terminology`, `refractive_swan_compliance::Policy`, and cached validation reports. Use the vector toggle on `/map` when you need deterministic lexical-only runs; the toggle drives a query param on `/api/map-bundles`.
- **Dataset admin:** `/admin/datasets` operates on the same `refractive_swan_eval::DatasetStore` that powers eval runs. Use the catalog table to enable/disable manifests, upload new `.manifest.json`/`.ndjson` pairs, download fixtures, kick off regression smoke tests (calls `refractive_swan_test_suite::ping()`), reset SQLite datamart files (only supported for `sqlite://` URLs), and clear app caches when HTMX fragments get stale.
