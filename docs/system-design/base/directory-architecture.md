# Directory Architecture

**Path:** `code/docs/system-design/directory-architecture.md`  
**Scope:** Rust workspace under `code/lib`

This document describes how the Rust workspace is organized into clear, discoverable buckets. The goal is to make it obvious:

- where **user-facing** surfaces live,
- where **domain logic** lives,
- where **ports/DTOs** are defined, and
- where **cross-cutting platform** concerns live.

---

## High-level layout

```text
code/
  docs/
    runbook/
    kanban/
    book/           # mdBook sources + build output (via cargo make docs)

  data/
    makefiles/

  lib/
    app/
      frontends/
        cli/
        web/
        desktop/
      servers/
        api/
        datamart_api/

    dto/
      web/
      cli/
      mesh/

    domain/
      core/
      semantics/
      pipeline/
      meta/
        ingestion/
        evaluation/
      ports/
        data/
          dto/
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

At a glance:

* `app/` – entrypoints and interfaces facing humans or external callers (CLI/web/desktop frontends + HTTP servers).
* `dto/` – surface-specific DTO veneers re-exporting canonical contracts (web today, CLI/mesh upcoming).
* `domain/` – problem-space logic, semantics, and outbound ports.
* `platform/` – runtime adapters: configuration, compliance, observability, data planes/stores, mesh orchestration.
* `docs/system-design/base/dependency-seams.md` – DTO & port ownership map.

### Layer boundaries & dependency hygiene

- **App → Domain → Platform (one way).** App crates may depend on domain/platform, domain crates may consume platform helpers, and platform crates never depend on app/domain.
- **No env/config in domain.** Configuration/env reads live in app or platform adapters. Domain crates accept typed configs/traits. Enforced via `cargo make layers-check`.
- **Package graph check.** `cargo make layers-check` uses `guppy` metadata to ensure `lib/domain/**` never references `std::env` and obeys dependency ordering.
- **Ports & adapters.** Domain crates expose traits/DTOs (under `domain/ports`). App crates implement inbound adapters; platform crates implement outbound adapters (vector store, datamart sink, external validators, mesh runtimes).

---

## Buckets and responsibilities

### 1. `app/` – Application surfaces

**Path:** `code/lib/app`

This is where anything “at the edge” of the system lives. Crates stay thin and delegate into `domain/` via ports + DTOs.

Current structure:

```text
code/lib/app/
  frontends/
    cli/
    web/
    desktop/
  servers/
    api/
    datamart_api/
```

* `frontends/cli` – `dfps_cli` binaries. Parse args/config, stream NDJSON, call pipeline/mapping ports, surface compliance/logging, exit with codes.
* `frontends/web` – `dfps_web_frontend` Actix UI. Renders HTMX/Tailwind, calls backend via DTO veneers, handles paste/upload workflows, metrics dashboard, docs redirect.
* `frontends/desktop` – reserved for future desktop shells (Tauri/Wry/etc.).
* `servers/api` – `dfps_api` Axum gateway. Thin HTTP ingress wiring domain pipeline + platform adapters (vector store, datamart sink, compliance). Loads `.env.app.web.api.<profile>`.
* `servers/datamart_api` – placeholder for future node/datamart APIs in the mesh rollout.

**Principle:** No heavy business logic lives in `app/`. Frontends & servers orchestrate domain ports + platform adapters only.

---

### 2. `dto/` – Surface-specific contract adapters

**Path:** `code/lib/dto`

Crates under `dto/` provide interface-specific DTO veneers over the canonical domain contracts (`lib/domain/contracts`). Each subdirectory maps to a surface and prevents drift.

Current structure:

```text
code/lib/dto/
  web/
  cli/
  mesh/   # planned
```

* `dto/web` (`dfps_web_dto`) – re-exports analytics/eval/pipeline payloads for the HTTP surfaces (Axum API + Actix frontend). Contains **no** env/config logic.
* `dto/cli` (`dfps_cli_dto`) – re-exports the DTOs consumed by the CLI binaries (pipeline metrics, eval summaries, load summaries, etc.) so the CLI depends on a curated surface instead of the entire contracts crate.
* `dto/mesh` – planned veneer for mesh/node control-plane DTOs used by future `dfps_mesh_node` APIs.

---

### 3. `domain/` – Core domain logic & ports

**Path:** `code/lib/domain`

These crates represent what the system **does**, independent of delivery details. Data flows `core → semantics → pipeline`, with supporting meta crates (ingestion/evaluation) and outbound ports.

Current (transitioning) structure:

```text
code/lib/domain/
  core/
  semantics/
  pipeline/
  meta/
    ingestion/
    evaluation/
  ports/
    data/
      dto/
      data-store/vector/
      data-plane/datamart/
    terminology/
    validation/
```

#### `core/` – Core models & kernel

Crate: `dfps_core`.

* Responsibilities: ID/primitives, clinical aggregates, staging/mapping value objects, serde + bridge helpers.
* Notes: module README ties back to system-design docs; `lib.rs` exposes stable paths (`dfps_core::{patient, order, staging, mapping, value, fhir}`).

#### `semantics/`

Crates: `dfps_mapping` (mapping engine, rankers), analytics helpers (summary calculators, mapping state logic).

* Responsibilities: lexical/vector rankers, rule rerankers, semantic helpers reused by pipeline/app layers. See `lib/domain/semantics/README.md` for the evolving directory structure; the actual crate still lives under `lib/domain/ontologies/mapping` until the migration is complete.

#### `pipeline/`

Crate: `dfps_pipeline`.

* Responsibilities: orchestrate ingestion + mapping via injected ports, emit `PipelineOutput`, expose `PipelinePort` for CLI/API/web.

#### `meta/ingestion`

Crate: `dfps_ingestion`.

* Responsibilities: transforms from FHIR bundles → staging rows, validation semantics, profile metadata. Houses profile snapshots (feature-gated) and consumes the validation port.

#### `meta/evaluation`

Crate: `dfps_eval` (+ `fake_data`).

* Responsibilities: evaluation harness, dataset store traits, deterministic fake data generators.

#### `ports/`

Namespace for outbound ports. Contains traits + DTO veneers consumed by domain crates and implemented by platform adapters.

* `data/dto` – e.g., `dfps_web_dto` (existing), `dfps_cli_dto` + `dfps_mesh_dto` (planned).
* `data/data-store/vector` – home for `dfps_vector_port` traits + helpers.
* `data/data-plane/datamart` – datamart sink port consumed by pipeline + analytics.
* `terminology` – terminology client/config traits re-exported from `dfps_terminology`.
* `validation` – `ExternalValidator` & related contexts used by ingestion.

During migration we keep compatibility re-exports (e.g., `dfps_mapping::vector` modules) so downstream crates compile.

---

### 4. `platform/` – Cross-cutting runtime adapters

**Path:** `code/lib/platform`

Platform crates sit between domain ports and operating-system concerns (env/config, storage, observability). They implement outbound adapters and shared helpers.

Structure:

```text
code/lib/platform/
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

* `meta/*` – env/config (`dfps_configuration`), compliance policies (`dfps_compliance`), observability (`dfps_observability`). Shared helpers for all apps.
* `test_suite` – integration/regression helpers (`dfps_test_suite`).
* `data/data-plane` – adapters that persist/query analytics data (`dfps_datamart`, warehouse/lake crates). Today some live under `app/servers/*`; MESH-025 + REFR-027 migrate them here.
* `data/data-store` – physical stores (vector, relational, cache). `dfps_vector_store` moves here.
* `mesh/*` – node runtime orchestration (`dfps_mesh_node`), research/orchestrator (`dfps_mesh_hub`), governance/policy services.

**Guardrails:** The `platform/{data,store,mesh}` directory names and depth are stable—do not rename them without updating the architecture docs + MESH-025 kanban. Runtime adapters should never drift back into `app/` once migrated.

---

## Using this document

1. When adding a new crate, decide its bucket here before writing code.
2. If creating a new port or DTO, update this file **and** `docs/system-design/base/dependency-seams.md`.
3. When migrating crates (e.g., moving `dfps_vector_store` into `platform/data-store/vector_store`), reference both REFR-027 and MESH-025 kanbans to keep status aligned.
4. Run `cargo make layers-check` after structural changes to ensure dependencies obey the architectural boundaries.
