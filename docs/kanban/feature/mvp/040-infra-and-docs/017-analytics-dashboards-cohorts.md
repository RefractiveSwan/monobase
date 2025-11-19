# Kanban - feature/analytics-dashboards-cohorts (017)

**Epic:** ANA-017 – Analytics dashboards & cohorts  
**Branch:** `feature/app/web/ANA-017-analytics-dashboard-cohorts` | **Target version:** `v0.1.0`  
**Status:** INPROGRESS | **Introduced:** `v0.1.0` | **Last updated:** `v0.1.0`  
**Theme:** Warehouse & analytics platform - BI-style dashboards & cohort UI  
**Goal:** Provide a minimal analytics surface (HTTP + web UI) to explore NCIt-coded cohorts and mapping state distributions, and define integration points for external BI tools.

### Columns
* **TODO** – Not started yet  
* **INPROGRESS** – In progress  
* **REVIEW** – Needs code review / refactor / docs polish  
* **DONE** – Completed  

---

## TODO

### ANL-01 – Backend analytics endpoints

- [ ] Extend `dfps_api` router with an `analytics` module:

  - [ ] `GET /analytics/ncit-summary`:

    - Returns counts of `FactServiceRequest` grouped by `ncit_id`, mapping state, and time bucket (if available).
  
  - [ ] `GET /analytics/cohort`:

    - Accepts query params (e.g., `ncit_id`, `status`, `date_from`, `date_to`).
    - Returns a list of matching `FactServiceRequest` rows plus dim context.

- [ ] Back these endpoints with either:

  - [ ] Direct queries into the warehouse DB (when epic 016 is implemented), or
  - [ ] In-memory aggregation over a streamed `PipelineOutput` (for single-bundle / demo mode).

#### Cross-Cohesion

- **Engineering Targets:** B, C, D
- **Crates & Paths:**
  - `lib/app/servers/api` (`dfps_api`)
  - `lib/app/servers/datamart` (`dfps_datamart`)
- **Shared Metrics & Signals:**
  - auto_mapped
  - needs_review
  - no_match
- **Docs & Kanbans Touched:**
  - `docs/system-design/clinical/ncit/architecture.md`
  - `docs/system-design/clinical/ncit/models/data-model-er.md`
  - `docs/kanban/feature/mvp/017-analytics-dashboards-cohorts.md`
- **Experiments / CI Hooks:**
  - `dfps_test_suite/tests/integration/web_api.rs`
  - `dfps_test_suite/tests/e2e/observability_metrics.rs`
- **Interfaces & Contracts:**
  - `GET /analytics/ncit-summary`
  - `GET /analytics/cohort`
  - `DFPS_API_HOST`, `DFPS_API_PORT`

### ANL-02 – Frontend analytics views

- [ ] Extend `dfps_web_frontend` with new routes/views:

  - [ ] `/analytics`:

    - Renders:

      - A chart of mapping state distributions over time.
      - Top NCIt concepts (by count) tiles.

  - [ ] `/analytics/cohort`:

    - Simple form to filter by NCIt ID, status, and date range.
    - Table of matching orders (sr_id, patient, ncit, status, intent, ordered_at).

- [ ] Add view models for analytics responses (e.g., `AnalyticsSummaryView`, `CohortRowView`) and tests to validate mapping.

#### Cross-Cohesion

- **Engineering Targets:** B, C
- **Crates & Paths:**
  - `lib/app/frontend/web` (`dfps_web_frontend`)
  - `lib/app/servers/api` (`dfps_api`)
- **Shared Metrics & Signals:**
  - auto_mapped
  - needs_review
  - no_match
- **Docs & Kanbans Touched:**
  - `docs/system-design/clinical/ncit/architecture.md`
  - `docs/kanban/feature/mvp/017-analytics-dashboards-cohorts.md`
- **Experiments / CI Hooks:**
  - `dfps_test_suite/tests/integration/web_api.rs`
  - UI snapshot/assertion tests under `dfps_web_frontend`
- **Interfaces & Contracts:**
  - `/analytics`, `/analytics/cohort` frontend routes
  - Backend clients for `GET /analytics/ncit-summary`, `GET /analytics/cohort`

### ANL-03 – BI integration surface

- [ ] Document a set of database views or API endpoints intended for BI tools:

  - [ ] E.g., `vw_fact_pet_ct`, `vw_dim_ncit`, or the `/analytics/cohort` endpoint.

- [ ] Add a minimal `docs/runbook/bi-integration-quickstart.md` describing:

  - [ ] How to point a BI tool (Superset/Metabase/etc.) at the warehouse schema.
  - [ ] Recommended views and fields.

#### Cross-Cohesion

- **Engineering Targets:** C, D
- **Crates & Paths:**
  - `lib/app/servers/datamart` (`dfps_datamart`)
  - `lib/app/servers/api` (`dfps_api`)
- **Shared Metrics & Signals:**
  - auto_mapped
  - needs_review
  - no_match
- **Docs & Kanbans Touched:**
  - `docs/kanban/feature/mvp/017-analytics-dashboards-cohorts.md`
  - `docs/system-design/clinical/ncit/models/data-model-er.md`
  - `docs/runbook/warehouse-quickstart.md`
- **Experiments / CI Hooks:**
  - `dfps_test_suite/tests/integration/warehouse.rs`
  - BI connectivity smoke tests (manual/CI)
- **Interfaces & Contracts:**
  - Warehouse views (e.g., `fact_sr`, `dim_ncit`)
  - `/analytics/cohort` API surface for BI tools

### ANL-04 – Observability & metrics

- [ ] Extend `PipelineMetrics` or introduce `AnalyticsMetrics` to track:

  - [ ] `cohort_queries`, `analytics_requests`, `avg_cohort_size`.

- [ ] Log analytics requests with correlation IDs and filters for debugging.

#### Cross-Cohesion

- **Engineering Targets:** C, D
- **Crates & Paths:**
  - `lib/platform/observability` (`dfps_observability`)
  - `lib/app/servers/api` (`dfps_api`)
- **Shared Metrics & Signals:**
  - auto_mapped
  - needs_review
  - no_match
  - vector_queries
  - vector_hits
- **Docs & Kanbans Touched:**
  - `docs/kanban/feature/mvp/017-analytics-dashboards-cohorts.md`
  - `docs/system-design/clinical/ncit/architecture.md`
- **Experiments / CI Hooks:**
  - `dfps_test_suite/tests/e2e/observability_metrics.rs`
  - `dfps_test_suite/tests/integration/web_api.rs`
- **Interfaces & Contracts:**
  - `GET /metrics/summary`
  - `PipelineMetrics` JSON schema
  - `DFPS_API_HOST`, `DFPS_API_PORT`

### ANL-05 – Tests & UX polish

- [ ] Integration tests (in `dfps_test_suite`) for:

  - [ ] `GET /analytics/ncit-summary` on baseline & unknown code fixtures.
  - [ ] `GET /analytics/cohort` returns coherent rows tied back to dims.

- [ ] Frontend tests to ensure:

  - [ ] Charts/tables render correctly given mock analytics endpoints.
  - [ ] Empty-state / error handling UX is sensible (no data, backend down, etc.).

#### Cross-Cohesion

- **Engineering Targets:** B, C
- **Crates & Paths:**
  - `lib/platform/test_suite` (`dfps_test_suite`)
  - `lib/app/frontend/web` (`dfps_web_frontend`)
- **Shared Metrics & Signals:**
  - auto_mapped
  - needs_review
  - no_match
- **Docs & Kanbans Touched:**
  - `docs/kanban/feature/mvp/017-analytics-dashboards-cohorts.md`
  - `docs/system-design/clinical/ncit/models/data-model-er.md`
- **Experiments / CI Hooks:**
  - `dfps_test_suite/tests/integration/web_api.rs`
  - `dfps_test_suite/tests/integration/vector_mapping.rs`
- **Interfaces & Contracts:**
  - `/analytics` and `/analytics/cohort` UI flows
  - Backend analytics endpoints and view models

---

## INPROGRESS
- _Empty_

---

## REVIEW
- _Empty_

---

## DONE
- _Empty_

---

## Acceptance Criteria

- A user can browse `http://localhost:<frontend>/analytics` to see NCIt-coded usage and mapping state distributions.
- Cohorts can be defined via the web UI or an HTTP endpoint, returning resolved dim/fact data.
- External BI tools have a documented and stable integration surface.

## Out of Scope

- Full-featured cohort editor (drag-and-drop criteria, saved queries).
- Fancy charting libraries beyond a simple, maintainable MVP.
