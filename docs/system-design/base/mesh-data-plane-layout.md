# Mesh Data Plane Layout

**Path:** `code/docs/system-design/base/mesh-data-plane-layout.md`  
**Scope:** Current → Target codebase layout mapping for mesh-first architecture  
**Tracking:** MESH-025 (feature/mesh-data-plane)

This document establishes the explicit mapping between the **current** codebase layout and the **target** `platform/{data,store,mesh}` architecture. It serves as the definitive reference for all MESH-025 implementation work.

---

## Current Tree Structure

As of 2025-11-21, the actual codebase layout under `lib/`:

```text
lib/
  app/
    frontend/
      cli/              (refractive_swan_cli)
      web/              (refractive_swan_web_frontend)
    servers/
      api/              (refractive_swan_api)
      datamart/         (refractive_swan_datamart)
      vector_store/     (refractive_swan_vector_store)
  
  domain/
    core/               (refractive_swan_core)
    contracts/          (refractive_swan_contracts)
    eval/               (refractive_swan_eval)
    pipeline/           (refractive_swan_pipeline)
    vector_port/        (refractive_swan_vector_port)
    ontologies/
      ingestion/        (refractive_swan_ingestion)
      mapping/          (refractive_swan_mapping)
      terminology/      (refractive_swan_terminology)
  
  platform/
    compliance/         (refractive_swan_compliance)
    configuration/      (refractive_swan_configuration)
    observability/      (refractive_swan_observability)
    stores/             (placeholder, minimal)
    test_suite/         (refractive_swan_test_suite)
```

### Key Observations

1. **Domain layer** is already close to target state, with clean separation of:
   - Core domain models (`refractive_swan_core`)
   - Ontology subsystems (ingestion, mapping, terminology)
   - Pipeline orchestration (`refractive_swan_pipeline`)
   - Evaluation harness (`refractive_swan_eval`)
   - Shared contracts (`refractive_swan_contracts`)
   - Vector abstract port (`refractive_swan_vector_port`)

2. **Platform layer** has supporting crates but lacks `data/`, `store/`, `mesh/` organization:
   - Compliance, configuration, observability, and test suite exist as platform crates
   - No mesh-level abstractions yet

3. **App servers** currently host only the HTTP runtime:
   - `app/servers/api` contains node runtime logic (to be extracted into `platform/mesh/node`)
   - `refractive_swan_datamart` and `refractive_swan_vector_store` have already moved under `platform/data/**`

---

## Target Tree Structure

The **planned** layout that MESH-025 establishes (directory names and depths are **frozen**):

```text
lib/
  app/
    frontend/
      cli/              (refractive_swan_cli)
      web/              (refractive_swan_web_frontend)
  
  domain/
    core/               (refractive_swan_core)
    contracts/          (refractive_swan_contracts)
    eval/               (refractive_swan_eval)
    pipeline/           (refractive_swan_pipeline)
    vector_port/        (refractive_swan_vector_port)
    ontologies/
      ingestion/        (refractive_swan_ingestion)
      mapping/          (refractive_swan_mapping)
      terminology/      (refractive_swan_terminology)
  
  platform/
    compliance/         (refractive_swan_compliance)
    configuration/      (refractive_swan_configuration)
    observability/      (refractive_swan_observability)
    test_suite/         (refractive_swan_test_suite)
    
    mesh/
      node/             (refractive_swan_mesh_node)       ← node runtime & governance integration
      hub/              (refractive_swan_mesh_hub)        ← research orchestrator / FL coordinator
      governance/       (refractive_swan_mesh_governance) ← mesh-level policies, DP/query model
    
    data/
      mart/             (refractive_swan_datamart)        ← dim/fact logic inside node
      warehouse/        (refractive_swan_datawarehouse)   ← backend-agnostic relational warehouse traits
      lake/             (refractive_swan_datalake)        ← local snapshots / Parquet/Delta lake
    
    store/
      relational_store/ (refractive_swan_relational_store) ← SQLx/Postgres/DuckDB drivers, per node
      vector_store/     (refractive_swan_vector_store)     ← Qdrant/PGVector, per node
      cache_store/      (refractive_swan_cache_store)      ← Redis, per node
      graph_store/      (refractive_swan_graph_store)      ← IndraDB / graph store, per node
```

### Key Principles

1. **Domain remains stable**: No changes to `lib/domain/*` layout
2. **Platform mesh/data/store names are frozen**: These directory names and depths will NOT change
3. **Each deployment is a sovereign node runtime**: `refractive_swan_mesh_node` wires together data plane components
4. **Clear separation of concerns**:
   - `mesh/` = node runtime, hub orchestration, governance
   - `data/` = analytical data layers (mart, warehouse, lake)
   - `store/` = persistence backends (relational, vector, cache, graph)

---

## Mapping Table: Current → Target

This table shows where each **current** crate will live in the **target** layout.

### App Layer Migrations

| Current Location | Current Crate | Future Location | Future Crate | Notes |
|------------------|---------------|-----------------|--------------|-------|
| `lib/app/frontend/cli` | `refractive_swan_cli` | `lib/app/frontend/cli` | `refractive_swan_cli` | **Stays** – user-facing CLI |
| `lib/app/frontend/web` | `refractive_swan_web_frontend` | `lib/app/frontend/web` | `refractive_swan_web_frontend` | **Stays** – user-facing web UI |
| `lib/app/servers/api` | `refractive_swan_api` | `lib/platform/mesh/node` | `refractive_swan_mesh_node` | **Moves** – becomes node runtime |
| `lib/app/servers/datamart` | `refractive_swan_datamart` | `lib/platform/data/mart` | `refractive_swan_datamart` | **Completed** – crate now lives under `platform/data/data-plane/mart` |
| `lib/app/servers/vector_store` | `refractive_swan_vector_store` | `lib/platform/store/vector_store` | `refractive_swan_vector_store` | **Completed** – crate now lives under `platform/data/data-stores/vector_store` |

### Domain Layer (No Changes)

| Current Location | Current Crate | Future Location | Future Crate | Notes |
|------------------|---------------|-----------------|--------------|-------|
| `lib/domain/core` | `refractive_swan_core` | `lib/domain/core` | `refractive_swan_core` | **Stays** |
| `lib/domain/contracts` | `refractive_swan_contracts` | `lib/domain/meta/contracts` | `refractive_swan_contracts` | **Stays** conceptually; crate now lives under `domain/meta/contracts`. |
| `lib/domain/meta/evaluation` | `refractive_swan_eval` | `lib/domain/meta/evaluation` | `refractive_swan_eval` | **Stays** |
| `lib/domain/meta/pipeline` | `refractive_swan_pipeline` | `lib/domain/meta/pipeline` | `refractive_swan_pipeline` | **Stays** |
| `lib/domain/ports/data/data-store/vector` | `refractive_swan_vector_port` | `lib/domain/ports/data/data-store/vector` | `refractive_swan_vector_port` | **Stays** – domain abstraction for vector stores |
| `lib/domain/meta/ingestion` | `refractive_swan_ingestion` | `lib/domain/meta/ingestion` | `refractive_swan_ingestion` | **Stays** |
| `lib/domain/ontologies/mapping` | `refractive_swan_mapping` | `lib/domain/ontologies/mapping` | `refractive_swan_mapping` | **Stays** |
| `lib/domain/ontologies/terminology` | `refractive_swan_terminology` | `lib/domain/ontologies/terminology` | `refractive_swan_terminology` | **Stays** |

### Platform Layer Migrations

| Current Location | Current Crate | Future Location | Future Crate | Notes |
|------------------|---------------|-----------------|--------------|-------|
| `lib/platform/compliance` | `refractive_swan_compliance` | `lib/platform/compliance` | `refractive_swan_compliance` | **Stays** |
| `lib/platform/configuration` | `refractive_swan_configuration` | `lib/platform/configuration` | `refractive_swan_configuration` | **Stays** |
| `lib/platform/observability` | `refractive_swan_observability` | `lib/platform/observability` | `refractive_swan_observability` | **Stays** |
| `lib/platform/test_suite` | `refractive_swan_test_suite` | `lib/platform/test_suite` | `refractive_swan_test_suite` | **Stays** |

### New Platform Crates (Not Yet Implemented)

| Future Location | Future Crate | Purpose | Seed/Dependencies |
|-----------------|--------------|---------|-------------------|
| `lib/platform/mesh/node` | `refractive_swan_mesh_node` | Node runtime & data plane orchestration | Seeds from current `refractive_swan_api` |
| `lib/platform/mesh/hub` | `refractive_swan_mesh_hub` | Research orchestrator, FL coordinator | Net new, design-first |
| `lib/platform/mesh/governance` | `refractive_swan_mesh_governance` | Mesh-level policies, query governance | Extends `refractive_swan_compliance` |
| `lib/platform/data/warehouse` | `refractive_swan_datawarehouse` | Backend-agnostic warehouse traits | Extracts from `refractive_swan_datamart` SQL wiring |
| `lib/platform/data/lake` | `refractive_swan_datalake` | Parquet/Delta lake snapshots | Net new, design-first |
| `lib/platform/store/relational_store` | `refractive_swan_relational_store` | SQLx/Postgres/DuckDB drivers | Extracts from `refractive_swan_datamart` connection logic |
| `lib/platform/store/cache_store` | `refractive_swan_cache_store` | Redis cache abstraction | Net new, design-first |
| `lib/platform/store/graph_store` | `refractive_swan_graph_store` | IndraDB/graph store abstraction | Related to `refractive_swan_terminology` OBO graphs |

---

## Conceptual vs Physical Locations During Transition

During the MESH-025 design and implementation phases, crates may be **conceptually** assigned to their target locations while **physically** remaining in their current locations. This allows:

1. **Design-first approach**: READMEs and architecture docs can reference target locations
2. **Incremental migration**: Code moves happen in controlled phases after design is stable
3. **Zero-disruption development**: Existing workflows continue while new structure is planned

### Example: refractive_swan_datamart

- **Conceptual location**: `lib/platform/data/mart`
- **Physical location**: `lib/platform/data/data-plane/mart`
- **Cargo.toml path**: `lib/platform/data/data-plane/mart`
- **Documentation references**: Use the platform path (legacy references should be updated)

### Example: refractive_swan_vector_store

- **Conceptual location**: `lib/platform/store/vector_store`
- **Physical location**: `lib/platform/data/data-stores/vector_store`
- **Abstraction**: Domain uses `refractive_swan_vector_port`, never depends on `refractive_swan_vector_store` directly

---

## Migration Phases

The transition from current to target layout follows these phases (detailed in `docs/system-design/mesh/migration-plan.md`):

### Phase 1: Conceptual Only (MESH-025 current work)

- Create `platform/{data,store,mesh}` directory structures with READMEs
- Document design and mapping (this document)
- Establish stability guardrails
- **No code moves**

### Phase 2: Internal Aliasing

- Create alias crates or module re-exports at target locations
- `refractive_swan_datamart` and `refractive_swan_vector_store` become available via new paths
- Old paths still work via re-exports

### Phase 3: Physical Move

- Update `Cargo.toml` paths to new directories
- Move actual crate code to target locations
- Verify all tests pass after migration

### Phase 4: Mesh Node

- Move `refractive_swan_api` → `refractive_swan_mesh_node`
- Create thin `refractive_swan_api` shim for backward compatibility
- Fully establish mesh runtime patterns

---

## Stability Contract

To maintain consistency throughout MESH-025 and beyond:

### Directory Names (FROZEN)

The following directory names and depths are **frozen** and will NOT change:

- `lib/platform/mesh/` (depth: 2)
  - `mesh/node/` (depth: 3)
  - `mesh/hub/` (depth: 3)
  - `mesh/governance/` (depth: 3)

- `lib/platform/data/` (depth: 2)
  - `data/mart/` (depth: 3)
  - `data/warehouse/` (depth: 3)
  - `data/lake/` (depth: 3)

- `lib/platform/store/` (depth: 2)
  - `store/relational_store/` (depth: 3)
  - `store/vector_store/` (depth: 3)
  - `store/cache_store/` (depth: 3)
  - `store/graph_store/` (depth: 3)

### Crate Names (STABLE)

Crate names follow the pattern `refractive_swan_<component>`:

- `refractive_swan_mesh_node`, `refractive_swan_mesh_hub`, `refractive_swan_mesh_governance`
- `refractive_swan_datamart`, `refractive_swan_datawarehouse`, `refractive_swan_datalake`
- `refractive_swan_relational_store`, `refractive_swan_vector_store`, `refractive_swan_cache_store`, `refractive_swan_graph_store`

### Cross-References

Any documentation or code referencing these directories should:

1. Use the **target** (conceptual) path in new design docs
2. Use the **current** (physical) path in `Cargo.toml` until migration
3. Include a note about conceptual vs physical location where relevant

---

## Usage Notes

### For Implementation Work

When implementing MESH-0X tasks:

- **Create new directories/READMEs**: Use target paths from this document
- **Reference existing crates**: Use current physical paths in code
- **Documentation**: Use conceptual (target) paths for architectural descriptions

### For Code Reviews

When reviewing MESH-025 PRs:

- Verify new `platform/{data,store,mesh}` directories match this document exactly
- Check that no accidental directory renames occur
- Ensure READMEs reference both conceptual and physical locations during transition

### For Future Work

When planning post-MESH-025 implementation epics:

- Use this document's mapping table to plan code moves
- Follow the phased migration strategy
- Maintain backward compatibility during transitions

---

## References

- **Kanban**: `docs/kanban/feature/mvp/040-infra-and-docs/025-mesh-data-plane.md`
- **Directory Architecture**: `docs/system-design/base/directory-architecture.md`
- **Migration Plan**: `docs/system-design/mesh/migration-plan.md` (to be created)
- **Dependency Seams**: `docs/system-design/base/dependency-seams.md`
