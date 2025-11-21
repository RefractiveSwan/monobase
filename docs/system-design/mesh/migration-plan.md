# Migration Plan: Mesh Data Plane Architecture

**Path:** `code/docs/system-design/mesh/migration-plan.md`  
**Scope:** Phased migration from current layout to `platform/{data,store,mesh}`  
**Tracking:** MESH-025 (feature/mesh-data-plane)

This document outlines the **phased migration strategy** for transitioning from the current codebase layout to the target mesh-first architecture, minimizing disruption to existing workflows.

---

## Overview

The migration follows a **4-phase approach**:

1. **Phase 1: Conceptual Only** – Design docs + skeleton directories (no code moves)
2. **Phase 2: Internal Aliasing** – Re-exports and module aliases (backward compat)
3. **Phase 3: Physical Move** – Move crate code to target locations
4. **Phase 4: Mesh Node** – Extract `dfps_mesh_node` from `dfps_api`

---

## Phase 1: Conceptual Only (MESH-025, Current)

**Goal**: Document the target architecture without moving code.

### Deliverables

- ✅  `docs/system-design/base/mesh-data-plane-layout.md` – Current → target mapping
- ✅ `docs/system-design/base/directory-architecture.md` – Stability guardrails
- ✅ `platform/{data,store,mesh}` skeleton directories
- ✅ Design READMEs for all future crates

### Code Changes

- **None** – Only documentation and empty directories
- Existing crates stay at current paths

### Verification

- `cargo check` passes (mesh contracts compile)
- All design docs reference correct target paths
- No accidental directory renames

---

## Phase 2: Internal Aliasing

**Goal**: Make target paths accessible while keeping code at current locations.

### Approach

Create **alias crates** or **module re-exports** at target locations that delegate to current implementations.

### Example: dfps_datamart

#### Current
```
lib/app/servers/datamart/
  src/
    lib.rs
    port.rs
    sql.rs
  Cargo.toml
```

#### Phase 2
```
lib/platform/data/mart/
  Cargo.toml  (final location  dfps_datamart)
  
lib/app/servers/datamart/
  (removed)
```

**`lib/platform/data/mart/Cargo.toml`:**
```toml
[package]
name = "dfps_datamart_alias"
version.workspace = true

[dependencies]
dfps_datamart = { path = "../../../app/servers/datamart" }

[lib]
path = "src/lib.rs"
```

**`lib/platform/data/mart/src/lib.rs`:**
```rust
pub use dfps_datamart::*;
```

### Benefits

- New code can reference `platform/data/mart` path
- Old code continues using `app/servers/datamart`
- Tests pass with either path
- Gradual migration without breaking changes

### Timeline

- **Duration**: 1-2 weeks
- **Effort**: Low (create alias crates for 3-5 crates)

---

## Phase 3: Physical Move

**Goal**: Move actual crate code to target locations.

### Prerequisites

- All tests passing with alias crates
- No active feature branches touching moved crates

### Steps

#### 1. Move Directories

```powershell
# Example: move datamart
git mv lib/app/servers/datamart lib/platform/data/mart
```

#### 2. Update Cargo.toml Paths

```toml
# In workspace Cargo.toml
[workspace]
members = [
  # OLD
  # "lib/app/servers/datamart",
  
  # NEW
  "lib/platform/data/mart",
]
```

#### 3. Update Internal Path References

```toml
# In other crates' Cargo.toml
[dependencies]
dfps_datamart = { path = "../../platform/data/mart" }  # was: ../app/servers/datamart
```

#### 4. Verify Tests

```powershell
cargo test --workspace
cargo check --workspace
```

### Crates to Move

| Crate | Current | Target |
|-------|---------|--------|
| `dfps_datamart` | `app/servers/datamart` | `platform/data/mart` |
| `dfps_vector_store` | `app/servers/vector_store` | `platform/store/vector_store` |

**NOT moved (yet)**:
- `dfps_api` (stays until Phase 4)
- `dfps_cli`, `dfps_web_frontend` (stay in `app/`)

### Timeline

- **Duration**: 1 week
- **Effort**: Medium (coordinate with active development)

---

## Phase 4: Mesh Node

**Goal**: Extract `dfps_mesh_node` from `dfps_api`.

### Prerequisites

- Phase 3 complete (datamart, vector_store moved)
- Mesh contracts (`dfps_contracts::mesh`) stable
- Governance/hub design reviewed

### Steps

#### 1. Create dfps_mesh_node Crate

```
lib/platform/mesh/node/
  src/
    data_plane.rs  (extract from dfps_api::server::NodeDataPlane)
    config.rs
    http/
      routes.rs
      handlers.rs
    lib.rs
  Cargo.toml
```

#### 2. Move Business Logic

Extract from `dfps_api::server`:
- `NodeDataPlane` struct → `dfps_mesh_node::data_plane`
- Job execution methods → `data_plane.rs`
- Config loading → `config.rs`

Keep in `dfps_api`:
- Axum router setup
- HTTP handler glue (thin wrappers)

#### 3. Create dfps_api Shim

```rust
// In dfps_api/src/lib.rs
pub use dfps_mesh_node::NodeDataPlane;

// Thin HTTP adapter
pub mod http {
    use dfps_mesh_node::NodeDataPlane;
    use axum::Router;
    
    pub fn router(plane: NodeDataPlane) -> Router {
        // Delegate to handlers
    }
}
```

#### 4. Update Deployments

Existing deployments can:
- **Option A**: Continue using `dfps_api` (shim wraps `dfps_mesh_node`)
- **Option B**: Migrate to `dfps_mesh_node` directly + choose HTTP framework

### Timeline

- **Duration**: 2-3 weeks
- **Effort**: High (requires careful extraction + testing)

---

## Backward Compatibility

### During Phase 2-3

- Both old and new paths work via aliases/re-exports
- No breaking changes to existing code
- Tests run against both paths

### After Phase 4

- `dfps_api` exists as compatibility shim
- Deprecated in favor of `dfps_mesh_node`
- Removed in next major version (e.g., v2.0)

---

## Rollback Strategy

### Phase 1
- **Rollback**: Delete empty directories + design docs
- **Impact**: Minimal (no code changed)

### Phase 2
- **Rollback**: Remove alias crates, delete target directories
- **Impact**: Low (no code moved, only aliases removed)

### Phase 3
- **Rollback**: `git mv` crates back to original locations, revert Cargo.toml
- **Impact**: Medium (requires coordination, but no code logic changed)

### Phase 4
- **Rollback**: Keep `dfps_mesh_node`, restore `dfps_api` as standalone crate
- **Impact**: High (significant refactoring reversed)

---

## Communication Plan

### Before Each Phase

- **Email**: Notify team of upcoming changes
- **Slack/Discord**: Post migration timeline
- **Docs**: Update README with current phase status

### During Each Phase

- **Daily standups**: Report progress
- **CI/CD**: Monitor test failures
- **Blockers**: Escalate quickly

### After Each Phase

- **Retrospective**: What worked, what didn't
- **Documentation**: Update runbooks, design docs
- **Announcement**: Phase complete, next phase ETA

---

## Success Criteria

### Phase 1
- ✅ All design docs published
- ✅ Skeleton directories created
- ✅ `cargo check` passes

### Phase 2
- ✅ Alias crates created
- ✅ New code can reference target paths
- ✅ All tests pass

### Phase 3
- ✅ Crates moved to target locations
- ✅ Cargo.toml paths updated
- ✅ CI/CD green

### Phase 4
- ✅ `dfps_mesh_node` extracted
- ✅ `dfps_api` shim works
- ✅ Existing deployments unaffected

---

## Timeline Summary

| Phase | Duration | Start | End |
|-------|----------|-------|-----|
| 1: Conceptual | 1 week | Week 1 | Week 1 |
| 2: Aliasing | 1-2 weeks | Week 2 | Week 3 |
| 3: Physical Move | 1 week | Week 4 | Week 4 |
| 4: Mesh Node | 2-3 weeks | Week 5 | Week 7 |

**Total**: ~7 weeks (excluding testing/stabilization)

---

## References

- **Layout Mapping**: `docs/system-design/base/mesh-data-plane-layout.md`
- **Node Runtime**: `docs/system-design/mesh/node-runtime.md`
- **Runbooks**: `docs/runbook/mesh-node-quickstart.md`, `docs/runbook/mesh-hub-quickstart.md`
