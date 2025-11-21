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
      cli/              (dfps_cli)
      web/              (dfps_web_frontend)
    servers/
      api/              (dfps_api)
      datamart/         (dfps_datamart)
      vector_store/     (dfps_vector_store)
  
  domain/
    core/               (dfps_core)
    contracts/          (dfps_contracts)
    eval/               (dfps_eval)
    pipeline/           (dfps_pipeline)
    vector_port/        (dfps_vector_port)
    ontologies/
      ingestion/        (dfps_ingestion)
      mapping/          (dfps_mapping)
      terminology/      (dfps_terminology)
  
  platform/
    compliance/         (dfps_compliance)
    configuration/      (dfps_configuration)
    observability/      (dfps_observability)
    stores/             (placeholder, minimal)
    test_suite/         (dfps_test_suite)
```

### Key Observations

1. **Domain layer** is already close to target state, with clean separation of:
   - Core domain models (`dfps_core`)
   - Ontology subsystems (ingestion, mapping, terminology)
   - Pipeline orchestration (`dfps_pipeline`)
   - Evaluation harness (`dfps_eval`)
   - Shared contracts (`dfps_contracts`)
   - Vector abstract port (`dfps_vector_port`)

2. **Platform layer** has supporting crates but lacks `data/`, `store/`, `mesh/` organization:
   - Compliance, configuration, observability, and test suite exist as platform crates
   - No mesh-level abstractions yet

3. **App servers** currently host what will become platform/data and platform/store:
   - `app/servers/api` contains node runtime logic
   - `app/servers/datamart` contains warehouse/mart implementation
   - `app/servers/vector_store` contains vector store implementation

---

## Target Tree Structure

The **planned** layout that MESH-025 establishes (directory names and depths are **frozen**):

```text
lib/
  app/
    frontend/
      cli/              (dfps_cli)
      web/              (dfps_web_frontend)
  
  domain/
    core/               (dfps_core)
    contracts/          (dfps_contracts)
    eval/               (dfps_eval)
    pipeline/           (dfps_pipeline)
    vector_port/        (dfps_vector_port)
    ontologies/
      ingestion/        (dfps_ingestion)
      mapping/          (dfps_mapping)
      terminology/      (dfps_terminology)
  
  platform/
    compliance/         (dfps_compliance)
    configuration/      (dfps_configuration)
    observability/      (dfps_observability)
    test_suite/         (dfps_test_suite)
    
    mesh/
      node/             (dfps_mesh_node)       ← node runtime & governance integration
      hub/              (dfps_mesh_hub)        ← research orchestrator / FL coordinator
      governance/       (dfps_mesh_governance) ← mesh-level policies, DP/query model
    
    data/
      mart/             (dfps_datamart)        ← dim/fact logic inside node
      warehouse/        (dfps_datawarehouse)   ← backend-agnostic relational warehouse traits
      lake/             (dfps_datalake)        ← local snapshots / Parquet/Delta lake
    
    store/
      relational_store/ (dfps_relational_store) ← SQLx/Postgres/DuckDB drivers, per node
      vector_store/     (dfps_vector_store)     ← Qdrant/PGVector, per node
      cache_store/      (dfps_cache_store)      ← Redis, per node
      graph_store/      (dfps_graph_store)      ← IndraDB / graph store, per node
```

### Key Principles

1. **Domain remains stable**: No changes to `lib/domain/*` layout
2. **Platform mesh/data/store names are frozen**: These directory names and depths will NOT change
3. **Each deployment is a sovereign node runtime**: `dfps_mesh_node` wires together data plane components
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
| `lib/app/frontend/cli` | `dfps_cli` | `lib/app/frontend/cli` | `dfps_cli` | **Stays** – user-facing CLI |
| `lib/app/frontend/web` | `dfps_web_frontend` | `lib/app/frontend/web` | `dfps_web_frontend` | **Stays** – user-facing web UI |
| `lib/app/servers/api` | `dfps_api` | `lib/platform/mesh/node` | `dfps_mesh_node` | **Moves** – becomes node runtime |
| `lib/app/servers/datamart` | `dfps_datamart` | `lib/platform/data/mart` | `dfps_datamart` | **Moves** – mart is a data plane component |
| `lib/app/servers/vector_store` | `dfps_vector_store` | `lib/platform/store/vector_store` | `dfps_vector_store` | **Moves** – store is a platform component |

### Domain Layer (No Changes)

| Current Location | Current Crate | Future Location | Future Crate | Notes |
|------------------|---------------|-----------------|--------------|-------|
| `lib/domain/core` | `dfps_core` | `lib/domain/core` | `dfps_core` | **Stays** |
| `lib/domain/contracts` | `dfps_contracts` | `lib/domain/contracts` | `dfps_contracts` | **Stays** – extended with `mesh` module |
| `lib/domain/eval` | `dfps_eval` | `lib/domain/eval` | `dfps_eval` | **Stays** |
| `lib/domain/pipeline` | `dfps_pipeline` | `lib/domain/pipeline` | `dfps_pipeline` | **Stays** |
| `lib/domain/vector_port` | `dfps_vector_port` | `lib/domain/vector_port` | `dfps_vector_port` | **Stays** – domain abstraction for vector stores |
| `lib/domain/ontologies/ingestion` | `dfps_ingestion` | `lib/domain/ontologies/ingestion` | `dfps_ingestion` | **Stays** |
| `lib/domain/ontologies/mapping` | `dfps_mapping` | `lib/domain/ontologies/mapping` | `dfps_mapping` | **Stays** |
| `lib/domain/ontologies/terminology` | `dfps_terminology` | `lib/domain/ontologies/terminology` | `dfps_terminology` | **Stays** |

### Platform Layer Migrations

| Current Location | Current Crate | Future Location | Future Crate | Notes |
|------------------|---------------|-----------------|--------------|-------|
| `lib/platform/compliance` | `dfps_compliance` | `lib/platform/compliance` | `dfps_compliance` | **Stays** |
| `lib/platform/configuration` | `dfps_configuration` | `lib/platform/configuration` | `dfps_configuration` | **Stays** |
| `lib/platform/observability` | `dfps_observability` | `lib/platform/observability` | `dfps_observability` | **Stays** |
| `lib/platform/test_suite` | `dfps_test_suite` | `lib/platform/test_suite` | `dfps_test_suite` | **Stays** |

### New Platform Crates (Not Yet Implemented)

| Future Location | Future Crate | Purpose | Seed/Dependencies |
|-----------------|--------------|---------|-------------------|
| `lib/platform/mesh/node` | `dfps_mesh_node` | Node runtime & data plane orchestration | Seeds from current `dfps_api` |
| `lib/platform/mesh/hub` | `dfps_mesh_hub` | Research orchestrator, FL coordinator | Net new, design-first |
| `lib/platform/mesh/governance` | `dfps_mesh_governance` | Mesh-level policies, query governance | Extends `dfps_compliance` |
| `lib/platform/data/warehouse` | `dfps_datawarehouse` | Backend-agnostic warehouse traits | Extracts from `dfps_datamart` SQL wiring |
| `lib/platform/data/lake` | `dfps_datalake` | Parquet/Delta lake snapshots | Net new, design-first |
| `lib/platform/store/relational_store` | `dfps_relational_store` | SQLx/Postgres/DuckDB drivers | Extracts from `dfps_datamart` connection logic |
| `lib/platform/store/cache_store` | `dfps_cache_store` | Redis cache abstraction | Net new, design-first |
| `lib/platform/store/graph_store` | `dfps_graph_store` | IndraDB/graph store abstraction | Related to `dfps_terminology` OBO graphs |

---

## Conceptual vs Physical Locations During Transition

During the MESH-025 design and implementation phases, crates may be **conceptually** assigned to their target locations while **physically** remaining in their current locations. This allows:

1. **Design-first approach**: READMEs and architecture docs can reference target locations
2. **Incremental migration**: Code moves happen in controlled phases after design is stable
3. **Zero-disruption development**: Existing workflows continue while new structure is planned

### Example: dfps_datamart

- **Conceptual location**: `lib/platform/data/mart` (as documented in design)
- **Physical location**: `lib/app/servers/datamart` (until Phase 3 of migration)
- **Cargo.toml path**: `lib/app/servers/datamart` (unchanged until migration)
- **Documentation references**: Should use conceptual location in new docs

### Example: dfps_vector_store

- **Conceptual location**: `lib/platform/store/vector_store`
- **Physical location**: `lib/app/servers/vector_store`
- **Abstraction**: Domain uses `dfps_vector_port`, never depends on `dfps_vector_store` directly

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
- `dfps_datamart` and `dfps_vector_store` become available via new paths
- Old paths still work via re-exports

### Phase 3: Physical Move

- Update `Cargo.toml` paths to new directories
- Move actual crate code to target locations
- Verify all tests pass after migration

### Phase 4: Mesh Node

- Move `dfps_api` → `dfps_mesh_node`
- Create thin `dfps_api` shim for backward compatibility
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

Crate names follow the pattern `dfps_<component>`:

- `dfps_mesh_node`, `dfps_mesh_hub`, `dfps_mesh_governance`
- `dfps_datamart`, `dfps_datawarehouse`, `dfps_datalake`
- `dfps_relational_store`, `dfps_vector_store`, `dfps_cache_store`, `dfps_graph_store`

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
