# Kanban – feature/be-expansion (029)

**Theme:** Backend & mesh-bound server expansion for dataset and policy orchestration  
**Branch:** `feature/app/BE-029-backend-expansion`  
**Goal:** Give the API/mesh ceilings first-class control over dataset-store admin operations (refresh/upload/tier metadata) while bridging compliance, terminology, and datamart signals so each mesh entity exposes predictable management surfaces.

> Status: **TODO**  
> Branch target version: `Unreleased`  
> Introduced in: `v0.1.0`  
> Last updated in: `v0.1.0`

---

## Background

FE-028 put the frontend in command-center mode; now the backend must keep up. The server needs mesh-aware dataset store admin APIs, compliance/terminology diagnostics, and datamart reset hooks so FE-028’s dataset/admin UX can reflect reality. This epic tracks the per-mesh entity work (API, node, hub) required to expose refresh/upload endpoints, policy inspectors, ingestion validation stats, and vector toggles.

---

## Columns

* **TODO** – Planned, not started
* **INPROGRESS** – Implementation/design happening
* **REVIEW** – Needs review or polish
* **DONE** – Completed & merged

---

## TODO

### 029-01 – Dataset store façade per mesh node

- [x] Split `refractive_swan_eval::DatasetStore` access into mesh-aware traits so API/hub/node instances can enforce tenancy and per-entity roots.  
- [x] Mirror manifest tier metadata + checksum state back through `/api/eval/datasets`, including “disabled” flags for nodes that should not surface certain datasets.  
- [x] Add a `DatasetNodeRegistry` cache keyed by mesh entity ID that tracks last refresh timestamp + manifest errors for observability.

### 029-02 – Admin endpoints (refresh/upload/delete)

- [x] Implement POST `/api/datasets/refresh` and `/api/datasets/upload` (accept manifest + NDJSON) with background jobs + status polling; enforce checksum verification before activation.  
- [x] Wire upload actions through to `refractive_swan_eval::DatasetStore` so new manifests land under the correct mesh namespace and tier.  
- [x] Provide DELETE `/api/datasets/:name` with guard rails (confirm active jobs complete, ensure FE sends confirmation token).  
- [ ] Emit structured events to `refractive_swan_observability` so FE-028 history panes can show dataset admin activity.

### 029-03 – Terminology/compliance/ingestion bridges

- [ ] Add `/admin/terminology/insights` endpoint returning registry coverage (licensed/open counts, OBO graph cache health, last sync).  
- [ ] Surface compliance policy summaries (`dfps_compliance::Policy`) via `/admin/compliance/policy` with read/write toggles for mesh operators; include license-blocked stats.  
- [ ] Expose ingestion validation aggregates (per `dfps_ingestion::validation::ValidationReport`) so FE workbench can show counts per job.  
- [ ] Document env toggles for lexical vs vector-backed mapping and add `/admin/toggles` endpoint to flip modes safely.

### 029-04 – Datamart/vector backplane hooks

- [ ] Provide `/admin/datamart/health` and `/admin/datamart/reset` endpoints that call into `refractive_swan_datamart` (with guard rails + audit logging).  
- [ ] Expose vector backend metadata (`dfps_vector_port`) including backend type, namespace, capacity snapshot, and fallback counters.  
- [ ] Ensure each mesh entity reports metrics tagged with `mesh_node_id` so observability dashboards can pivot per node.

### 029-05 – Dataset regression + docs

- [ ] Create regression fixtures + CLI commands that exercise refresh/upload flows end-to-end; wire them into `refractive_swan_test_suite`.  
- [ ] Update FE/BE runbooks with dataset admin instructions (mesh vs standalone) and call out the new endpoints.  
- [ ] Add Playwright/CLI smoke steps covering admin flows once FIT endpoints stabilize.  
- [ ] Coordinate with FE-028 to swap placeholder admin buttons for live HTMX calls once these endpoints hit `main`.

---

## INPROGRESS

*Empty*

---

## REVIEW

*Empty*

---

## DONE

*Will be populated as tasks land.*

---

## Acceptance criteria

1. Mesh-aware dataset admin endpoints (list/refresh/upload/delete/toggles) exist, documented, and enforce tenancy/validation.  
2. Terminology, compliance, ingestion, and vector status endpoints expose structured JSON surfaces consumed by FE-028.  
3. Datamart reset/health plus dataset upload flows emit audit events so observability dashboards remain trustworthy.  
4. Regression/CLI/runbook coverage ensures operators can run dataset admin tasks outside the UI.

---

## Risks & mitigations

- **Cross-entity divergence** – Keep API/hub/node surfaces aligned via shared contracts + DTO crates.  
- **Data loss during resets** – Implement dry-run + snapshot guard rails before destructive actions.  
- **Upload abuse** – Enforce checksum + size limits and require auth scopes for admin routes.  
- **Frontend/backlog drift** – Coordinate milestones with FE-028 so UI toggles only light up when endpoints exist.
