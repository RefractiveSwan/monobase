# Kanban - feature/app/web-mvp

**Epic:** Web surface for the FHIR -> NCIt pipeline, implemented as:
- Backend API: `feature/app/web/backend-mvp`
- Frontend UI: `feature/app/web/frontend-mvp`

### Columns
* **TODO** – Not started yet  
* **INPROGRESS** – In progress  
* **REVIEW** – Needs code review / refactor / docs polish  
* **DONE** – Completed  

---

## TODO

### Frontend – Web UI (`dfps_web_frontend` or external app)
_Working branch: `feature/app/web/frontend-mvp`_

#### WEB-FE-04 – UX polish & copy
- [x] Align terminology with `docs/system-design/clinical/fhir/*` and `docs/system-design/clinical/ncit/*` (Bundle, `stg_servicerequest_flat`, `stg_sr_code_exploded`, etc.).
- [x] Add basic help text / tooltips explaining:
  - [x] What AutoMapped/NeedsReview/NoMatch mean.
  - [x] How the mapping engine uses NCIt and mock UMLS xrefs.
- [x] Add sensible empty/error states for:
  - [x] No mappings returned.
  - [x] Backend unreachable or `health` failing.

#### WEB-FE-05 – Frontend tests & wiring
- [x] Add unit/interaction tests (component tests or minimal e2e) for:
  - [x] Submitting a Bundle and rendering mappings.
  - [x] Rendering metrics and NoMatch lists.
- [ ] Optional: add a small CI step that:
  - [ ] Builds the frontend.
  - [ ] Runs the critical tests.

#### WEB-FE-06 – Docs & Quickstart (frontend)
- [x] Extend `docs/system-design/base/directory-architecture.md` or a new `docs/system-design/clinical/web-ui.md` to describe:
  - [x] Where the frontend lives in `code/`.
  - [x] How it interacts with the backend API.
- [x] Add Quickstart snippets:
  - [x] “Run backend server + frontend dev server.”
  - [x] Example curl/UI flows for mapping bundles.

---

## INPROGRESS

### Backend – HTTP API gateway (`dfps_api`)
_Working branch: `feature/app/web/backend-mvp`_

---

### Frontend – Web UI (`dfps_web_frontend`)
_Working branch: `feature/app/web/frontend-mvp`_

#### WEB-FE-01 – Frontend project scaffold
- [x] Create a frontend project (e.g., `code/app/web/frontend`) using actix-web.
- [x] Document how it discovers the backend base URL (env var, config file, etc.).
- [x] Add a small “API client” layer that calls:
  - [x] `POST /api/map-bundles`
  - [x] `GET /metrics/summary`
  - [x] `GET /health`

#### WEB-FE-02 – Bundle upload & mapping viewer
- [x] Implement a “Bundle upload” or “Paste JSON” screen:
  - [x] Let the user upload a Bundle JSON file or paste raw JSON.
  - [x] Call the backend `map-bundles` endpoint.
- [x] Render:
  - [x] A summary of SR flats (count, statuses/intents).
  - [x] A table of `MappingResult` rows (code, system, NCIt ID, state).
  - [x] A small badge or chips for `AutoMapped / NeedsReview / NoMatch`.

#### WEB-FE-03 – Metrics & NoMatch explorer
- [x] Add a simple dashboard view that:
  - [x] Calls `GET /metrics/summary`.
  - [x] Shows counts by mapping state as Tailwind cards.
- [x] Add a “NoMatch explorer” view that:
  - [x] Lists codes with `MappingState::NoMatch` plus their code metadata.
  - [x] Displays the `reason` badge for triage.
- [x] Add unit coverage for the frontend view model to validate NoMatch derivation.

---

## REVIEW

### Backend – HTTP API gateway (`dfps_api`)
_Working branch: `feature/app/web/backend-mvp`_

#### WEB-BE-01 – Scaffold web backend crate
- [x] Create `code/lib/app/web/backend/api` (or similar) with `Cargo.toml` + `src/main.rs`.
- [x] Add the crate to the root `[workspace].members` under the `app` section.
- [x] Expose a `run()` function that `main()` delegates to so tests can drive the server in-process.

#### WEB-BE-02 – Core FHIR -> NCIt HTTP API
- [x] Add dependencies on `dfps_pipeline` and `dfps_observability`.
- [x] Implement `POST /api/map-bundles`:
  - [x] Accept a single FHIR `Bundle` or an array/NDJSON of Bundles.
  - [x] For each bundle, call `bundle_to_mapped_sr`.
  - [x] Return JSON containing `flats`, `exploded_codes`, `mapping_results`, and `dim_concepts`.
- [x] Define clear error responses for:
  - [x] Invalid JSON.
  - [x] Invalid FHIR (surfacing `IngestionError` information).
  - [x] Internal errors (500 with correlation ID).

#### WEB-BE-03 – Health & metrics endpoints
- [x] Add `GET /health` for basic liveness.
- [x] Add `GET /metrics/summary` that:
  - [x] Computes or aggregates `PipelineMetrics` for recent runs.
  - [x] Returns counts per `MappingState` to support dashboards.
- [x] Ensure structured logs include a request ID / correlation ID for each call.

#### WEB-BE-04 – Tests & CI for backend
- [x] Add integration tests (in `dfps_api` or `dfps_test_suite`) that:
  - [x] Spin up the server in-process (no external port binding).
  - [x] `POST /api/map-bundles` with the baseline FHIR bundle fixture and assert NCIt IDs and mapping states.
  - [x] `POST /api/map-bundles` with an “unknown code” bundle and assert `NoMatch` handling + proper HTTP status.
- [x] Add a CI smoke test that:
  - [x] Starts the server.
  - [x] Runs `GET /health` and a minimal `POST /api/map-bundles`.

#### WEB-BE-05 – Directory-architecture alignment (backend)
- [x] Update `docs/system-design/base/directory-architecture.md` to:
  - [x] Add a “web backend” entry under `lib/app` (e.g., `app/web/backend/api`).
  - [x] Describe its responsibilities as an HTTP gateway over the FHIR -> NCIt pipeline.

---

## DONE
- _Empty_


---

# Next steps

- Customize bundles to reproduce `NeedsReview` or `NoMatch` scenarios and verify how the UI reflects them.
- Tail `dfps_api` logs while submitting bundles to correlate request IDs between backend and frontend (HTMX surfaces alert banners with request context).
- Hook the frontend into future CI by running `cargo test -p dfps_web_frontend` plus optional `cargo fmt --check`.

Once comfortable with this workflow, you can iterate on new UX panels or backend endpoints knowing the full loop from FHIR bundle to NCIt mapping.
