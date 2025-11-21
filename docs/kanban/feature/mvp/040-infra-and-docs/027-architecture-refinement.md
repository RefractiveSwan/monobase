# Kanban – feature/architecture-refinement (027)

**Theme:** Simplify workspace architecture & clarify port/adapter ownership  
**Branch:** `feature/meta/REFR-027-architecture-refinement`  
**Goal:** Produce a neater, easier-to-navigate architecture plan so every crate knows its home and migration steps from the current layout to the mesh-ready structure are unambiguous.

> Status: **TODO**  
> Branch target version: `Unreleased`  
> Introduced in: `v0.1.0`  
> Last updated in: `Unreleased`

---

## Background

REFR-024 delivered baseline documentation for `lib/app`, `lib/domain`, and `lib/platform`. MESH-025 sketches the end-state (`platform/{mesh,data,store}`) but the current repository still mixes runtime adapters (`dfps_api`, `dfps_datamart`, `dfps_vector_store`) with domain crates, and only the web DTOs have their own bucket.

REFR-027 crystallizes the “next hop” architecture so future migrations are incremental instead of ad hoc. It also introduces a dedicated **ports shelf** so domain-level contracts (vector, terminology, datamart sinks, external validators) have one discoverable home before adapters wire them.

---

## Current pain points

1. **Ports scattered inside feature crates.** `vector_port` lives next to `ontologies` while other outbound ports (`DatamartPort`, `ExternalValidator`, `TerminologyClient`) sit inside their feature crates. New contributors cannot find all “hex ports” in one place.
2. **App vs platform boundary is fuzzy.** `lib/app/servers/{api,datamart,vector_store}` include adapters that conceptually belong to `platform/{mesh,data,store}` but moving them lacks a concrete roadmap.
3. **DTO veneers incomplete.** Only the web layer enjoys a dedicated `lib/dto/web`. CLI and mesh-control planes still import `dfps_contracts` directly, so schema drift is likely.
4. **Kanban split between REFR-024 and MESH-025 is unclear.** REFR-024 focuses on docs, MESH-025 on mesh layout, but the mid-term architecture (before mesh) is not written down.

---

## Target architecture (REFR-027)

```text
lib/
  app/
    frontends/
      cli/            (dfps_cli binaries)
      web/            (dfps_web_frontend)
      desktop/        (reserved)
    servers/
      api/            (HTTP ingress; thin adapters only)
      datamart_api/   (future node APIs)
  domain/
    core/
    semantics/        (mapping, analytics)
    pipeline/
    meta/
      ingestion/
      evaluation/
    ports/
      data/
        dto/
          web/              (existing)
          cli/              (NDJSON/CLI veneers)
          mesh/             (node control-plane DTOs)
        data-store/
          vector/
        data-plane/
          datamart/
      terminology/
      validation/
  platform/
    meta/
      configuration/
      compliance/
      observability/
    test_suite/
    data/
      data-plane/
        mart/
        warehouse/
        lake/
      data-store/
        vector_store/
        relational_store/
        cache_store/
    mesh/
      node/
      hub/
      governance/
```

**Key decisions**

- **App/frontends vs servers.** Frontends host user-facing binaries (CLI, web, desktop shells). Servers host ingress surfaces (Axum API, future datamart-facing APIs) and stop embedding datastore/vector logic.
- **Domain semantics + meta.** `domain/semantics` groups shared mapping/analytics logic. `domain/meta/{ingestion,evaluation}` becomes the home for flows currently scattered under `ontologies` and `evaluation`. Every domain crate delegates cross-cutting contracts to `domain/ports`.
- **Domain ports shelf.** `lib/domain/ports` becomes the single source of truth for outbound ports. Subtrees (`data/dto`, `data/data-store`, `data/data-plane`, `terminology`, `validation`) match the eventual platform adapter layout so the wiring story is symmetrical.
- **DTO veneers everywhere.** Under `domain/ports/data/dto/*` we house the veneers (`web`, `cli`, `mesh`) that re-export canonical contracts for each surface. This keeps CLI + mesh adapters aligned with the web layer.
- **Platform meta/data/store/mesh.** Platform-level adapters consolidate env/config/compliance/observability under `platform/meta`, with runtime adapters moving into `platform/data`, `platform/store`, and `platform/mesh` following the MESH-025 blueprint.
- **App layer stays thin.** With DTOs + ports centralized, applications simply compose ports, adapters, and view models; no more business logic lives in `lib/app`.

### Directory notes

- `domain/ports/data/dto/*` – crates such as `dfps_web_dto`, upcoming `dfps_cli_dto`, and future `dfps_mesh_dto`.
- `domain/ports/data/data-store/vector` – replacement home for `dfps_vector_port` traits and helpers.
- `domain/ports/data/data-plane/datamart` – defines sinks/ports for analytics persistence consumed by platform data-plane adapters.
- `domain/ports/terminology` – shared `TerminologyClient` traits/configs used by mapping + ingestion flows.
- `domain/ports/validation` – `ExternalValidator` and similar invariants for ingestion.
- `platform/meta/{configuration,compliance,observability}` – existing crates; the doc simply groups them explicitly under a `meta` bucket.
- `platform/data/data-plane/*` & `platform/data/data-store/*` – the physical home for datamart, warehouse, lake, and store adapters once migrations complete.

---

## Migration plan

### Phase 0 – Documentation & guardrails
- [x] Update `docs/system-design/base/directory-architecture.md` with the new tree and add a **ports** subsection under `domain`.
- [x] Extend `docs/system-design/base/dependency-seams.md` with entries for each port (vector, terminology, datamart, validator) referencing the new `lib/domain/ports` path.
- [x] Record the target tree inside `docs/kanban/feature/mvp/040-infra-and-docs/025-mesh-data-plane.md` so both kanbans align.

### Phase 1 – Domain ports consolidation
- [x] Create `lib/domain/ports/` and move `dfps_vector_port` there (pure rename; no code change besides path updates).
- [x] Extract `TerminologyClient`, `ExternalValidator`, and `DatamartSinkPort` traits into sibling crates/modules under `domain/ports`.
- [x] Update feature crates to depend on these ports via the new path.

### Phase 2 – Domain layout alignment
- [x] Move `dfps_ingestion` and `dfps_eval` under `domain/meta/{ingestion,evaluation}` to reflect the plan.
- [x] Introduce `domain/semantics` module that groups mapping + analytics helpers (without breaking crate boundaries yet) and document the intent.
- [x] Update `directory-architecture.md` + crate READMEs to reflect the new naming.

### Phase 3 – DTO veneers
- [ ] Add `lib/dto/cli` for NDJSON/CLI payload wrappers (map_bundles/map_codes) and update CLI bins to consume it.
- [ ] Draft `lib/dto/mesh` for node governance/control-plane payloads needed by the upcoming mesh server.

### Phase 4 – Platform moves (ties into MESH-025)
- [ ] Relocate `lib/app/servers/vector_store` → `lib/platform/store/vector_store`.
- [ ] Relocate `lib/app/servers/datamart` → `lib/platform/data/mart`.
- [ ] Extract `NodeDataPlane` orchestration from `dfps_api` into `lib/platform/mesh/node`.

Each phase must keep CI green (run `cargo make fmt`, `clippy`, `test`) and update docs/kanban entries plus `CHANGELOG.md`.

---

## Acceptance criteria

1. Architecture docs reflect the new buckets (app/frontends vs servers, domain/ports, dto expansion, platform/{data,store,mesh}).
2. `lib/domain/ports` exists and houses vector + terminology + validator + datamart traits, with domain crates referencing it.
3. DTO veneers cover web + CLI (mesh optional until Phase 2) with clear ownership documented.
4. Platform adapters live under `platform/{data,store,mesh}` after migrations, and app crates reference them via the new paths.
5. MESH-025 milestones link back to this card for all moves touching platform directories.

---

## Risks & mitigations

- **Risk:** Large path renames break `cargo` until every crate updates dependencies.  
  **Mitigation:** Use phased PRs (ports → DTOs → adapters), rely on `cargo check -p <crate>` during each move, and keep `git mv` history.

- **Risk:** Docs fall out of sync.  
  **Mitigation:** Require updates to `directory-architecture.md`, `dependency-seams.md`, and the relevant kanban card in every architecture PR.

- **Risk:** Tests coupling to old paths (e.g., fixtures referencing `dfps_vector_port`).  
  **Mitigation:** Export compatibility re-exports (`pub use dfps_domain_ports::vector::*;`) during the migration window.

---

## Next steps

- Approve this plan.
- Spin up branch `feature/meta/REFR-027-architecture-refinement`.
- Execute Phase 0 documentation updates + add tracking checklist to this card.
