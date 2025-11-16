# Kanban — feature/mapping-vector-backend (013)
**Epic:** VEC‑013 – Mapping vector backend  
**Branch:** `feature/VEC-013-mapping-vector-backend` | **Target version:** `v0.1.0`  
**Status:** DOING | **Introduced:** `v0.1.0` | **Last updated:** `v0.1.0`  
**Theme:** External infra & heavy services — real vector DB / vector search  
**Goal:** Pluggable vector‑store backed ranker for `dfps_mapping` (pgvector/Qdrant/Milvus), wired into `MappingEngine` and CLIs, with clean fallbacks when the backend is unavailable.

## Executive Summary
- Add a `VectorStore` trait + feature‑flagged implementations (pgvector, Qdrant; optional Milvus). FOSS‑only.
- Introduce `VectorRankerBackend` used by `MappingEngine`; default test mode remains offline/pure‑Rust.
- Provide a CLI index builder; indexes are namespace‑scoped; rebuild is idempotent.
- When vector backend is disabled/unhealthy, pipeline falls back deterministically to lexical + mock vector ranker.
- Observability: counters (`vector_queries`, `vector_hits`, `vector_fallbacks`) and latency; structured logs.
```mermaid
flowchart LR
  Stg[stg_sr_code_exploded] --> ME[MappingEngine]
  ME --> Lex[Lexical ranker]
  ME -->|if DFPS_VECTOR_ENABLED| Vec[VectorStore backend]
  Vec --> Cand[Top‑k candidates]
  Lex --> Cand
  Cand --> Thresh[Thresholds & rules]
  Thresh --> Out[MappingResult]
```

## Scope & Non‑Goals
- In scope: VectorStore trait + config; one concrete backend (pgvector or Qdrant) behind feature flag; optional Milvus notes; CLI index builder; MappingEngine wiring; tests; metrics; docs/runbooks.
- Out of Scope (verbatim):
  - Online training / incremental embedding updates.
  - Multi-tenant / sharded vector clusters beyond a single-namespace MVP.

### Columns

* **TODO** – Not started yet
* **INPROGRESS** – In progress
* **REVIEW** – Needs code review / refactor / docs polish
* **DONE** – Completed

# Cards

## TODO

### VEC-05 – Tests & observability
Add integration coverage, capacity drift checks, and metrics so vector mode is observable and gated (Targets A3/B/D).
- Tests
  - [ ] `dfps_test_suite/tests/integration/vector_mapping.rs` with Docker backend and test double; assert uplift vs mock and deterministic offline path.
  - [ ] Capacity proxy test (norm/participation ratio) stable for same seed; hit@k histogram expectations captured in fixture.
- Metrics/CI
  - [ ] Metrics: `vector_queries`, `vector_hits`, `vector_fallbacks`, latency (mean/p95) exposed via `dfps_observability`; structured logs for connectivity/index events with namespace/backend/duration.
  - [ ] CI gate fails if vector-enabled recall drops >X% vs baseline or latency exceeds budget; include error/timeout codes in logs and Prometheus-friendly exports.

#### Cross-Cohesion

- **Engineering Targets:** A3, B, D
- **Crates & Paths:**
  - `lib/platform/test_suite/tests/integration/vector_mapping.rs`
  - `lib/domain/mapping` (`dfps_mapping`)
- **Shared Metrics & Signals:**
  - `geom_rm`, `geom_dm`, `cap_alpha_sim`
  - `vector_queries`, `vector_hits`, `vector_fallbacks`, `vector_latency_ms_p95`
- **Docs & Kanbans Touched:**
  - `docs/system-design/clinical/ncit/concepts/vector-layer.md`
  - `docs/runbook/vector-store-quickstart.md`
- **Experiments / CI Hooks:**
  - CI job collecting capacity proxies and hit@k per backend
  - Drift checks that fail when recall or capacity proxy regresses
- **Interfaces & Contracts:**
  - Traits: `VectorStore`
  - Env: `DFPS_VECTOR_ENABLED`, `DFPS_VECTOR_BACKEND`

### VEC-06 – Docs & runbooks
Document the vector layer concept and operational runbook, including capacity checklist and fallback steps (Targets A1/A3/B/D/C).
- System design
  - [ ] Author `docs/system-design/clinical/ncit/concepts/vector-layer.md` covering placement between staging and mapping, geometry effects (radius/dimension/centroid overlap), and Leiden/Louvain graph conditioning.
  - [ ] Include capacity checklist (embedding_version, dim, norm stats, community health notes) and explicit fallback guidance.
- Runbook
  - [ ] Add `docs/runbook/vector-store-quickstart.md` with pgvector/Qdrant setup snippets, FOSS-only dependencies, and CLI examples (`build-vector-index`, `map-codes`).
  - [ ] Troubleshooting steps for downtime/capacity regressions (switch to mock, rebuild index) and instructions for running eval harness comparing vector-enabled vs mock quality with expected metrics.

#### Cross-Cohesion

- **Engineering Targets:** A1, A3, B, D
- **Crates & Paths:**
  - `docs/system-design/clinical/ncit/concepts/vector-layer.md`
  - `docs/runbook/vector-store-quickstart.md`
- **Shared Metrics & Signals:**
  - `geom_rm`, `geom_dm`, `geom_centroid_cos`
  - `vector_queries`, `vector_fallbacks`
- **Docs & Kanbans Touched:**
  - `docs/kanban/feature/mvp/013-mapping-vector-backend.md`
  - `docs/system-design/clinical/ncit/architecture/system-architecture.md`
- **Experiments / CI Hooks:**
  - Reference paths for `dfps_eval` comparisons of vector-enabled vs mock runs
  - Guidance for `dfps_test_suite/tests/integration/vector_mapping.rs` expectations
- **Interfaces & Contracts:**
  - CLIs: `dfps_cli build-vector-index`, `dfps_cli map-codes`
  - Env: `DFPS_VECTOR_NAMESPACE`, `DFPS_VECTOR_ENABLED`

## DONE

### VEC-01 – VectorStore abstraction & wiring
Define the shared VectorStore crate/config so vector search can be toggled on without breaking offline determinism; expose capacity hooks to watch geometry health (Targets A1/A3/B).
- Implementation
  - [x] Create `dfps_vector_store` under `lib/platform/vector_store` with FOSS-only deps; add `VectorStore` trait (`health`, `index_items`, `search`) and optional `EmbeddingProvider`, all `Send + Sync` and namespace-required.
  - [x] Ship `VectorStoreConfig` (`DFPS_VECTOR_URL`, `DFPS_VECTOR_NAMESPACE`, `DFPS_VECTOR_BACKEND`, `DFPS_VECTOR_POOL_MAX`, `DFPS_VECTOR_HEALTH_TIMEOUT_MS`, `DFPS_VECTOR_ENABLED`) plus validation for backend/namespace combinations and pool/timeout bounds.
  - [x] Add `VectorRankerBackend` in `dfps_mapping` implementing `CandidateRanker` via `VectorStore::search`; deterministic ordering on ties and explicit namespace resolution per code.
- Metrics/tests/docs
  - [x] Provide embedding hook metadata (`embedding_version`, dim) and deterministic embedding fn to bridge staging codes to vectors; thread through `MappingResult` source version as needed.
  - [x] Expose capacity/correlation proxies per namespace (`geom_rm`, `geom_dm`, `geom_rm_sqrt_dm`, `cap_alpha_sim`) through the trait; document deterministic embedding complexity and GPLv3/FOSS-only posture
  - [x] Provide a mock `VectorStore` that records `vector_queries`/`vector_fallbacks`; unit tests for env parsing, namespace guards, and ordered ties, including disabled-mode behavior.
- Tests/observability
  - [x] Surface capacity proxies (`geom_rm_sqrt_dm`, `cap_alpha_sim`) and document deterministic embedding complexity (`O(kd)` query, bounded build) plus error/fallback logging contract.

#### Cross-Cohesion

- **Engineering Targets:** A1, A3, B
- **Crates & Paths:**
  - `lib/platform/vector_store` (`dfps_vector_store`)
  - `lib/domain/mapping` (`dfps_mapping`)
- **Shared Metrics & Signals:**
  - `geom_rm`, `geom_dm`, `geom_rm_sqrt_dm`
  - `vector_queries`, `vector_hits`, `vector_fallbacks`
- **Docs & Kanbans Touched:**
  - `docs/system-design/clinical/ncit/concepts/vector-layer.md`
  - `docs/kanban/feature/mvp/013-mapping-vector-backend.md`
- **Experiments / CI Hooks:**
  - `dfps_test_suite/tests/integration/vector_mapping.rs`
  - `dfps_cli map-codes` offline vs vector-enabled smoke tests
- **Interfaces & Contracts:**
  - Traits: `VectorStore`, `CandidateRanker`
  - Env: `DFPS_VECTOR_ENABLED`, `DFPS_VECTOR_BACKEND`, `DFPS_VECTOR_NAMESPACE`

### VEC-02 – First concrete backend (Qdrant)
Implement and harden the first FOSS backend (Qdrant) behind a feature flag with namespace isolation, health probes, and drift checks (Targets A1/A2/B/C).
- Implementation
  - [x] Implement the backend client (pool + `health()`) and schema/collection keyed by `(namespace, ref_id)` with recorded metadata (`backend`, `dim`, `metric`, `embedding_version`).
  - [x] Provide backend bootstrap (DDL/collection init, ANN params) with cost notes (`O(n log n)` build, `O(kd)` query) and rebuild semantics; add CI skip when backend feature disabled.
  - [x] Add `.env.domain.mapping.dev` and `.env.platform.vector_store.dev` templates plus dev docker-compose instructions; include FOSS-only dependency statement.
- Guardrails/tests
  - [x] Feature-gated health/create/drop integration tests per namespace; skip path when backend unavailable.
  - [x] Capacity/recall guardrails: CI thresholds `geom_rm_sqrt_dm` drift ≤ 10%, `cap_alpha_sim` ≥ baseline − 5%, mapping recall uplift ≥ +3% vs mock on PET/CT fixture with baseline recorded in fixture metadata.

#### Cross-Cohesion

- **Engineering Targets:** A1, A2, B
- **Crates & Paths:**
  - `lib/platform/vector_store` (`dfps_vector_store`)
  - `lib/domain/mapping` (`dfps_mapping`)
- **Shared Metrics & Signals:**
  - `geom_rm`, `geom_dm`, `geom_rm_sqrt_dm`
  - `vector_latency_ms_p50`, `vector_latency_ms_p95`, `vector_fallbacks`
- **Docs & Kanbans Touched:**
  - `docs/runbook/vector-store-quickstart.md`
  - `docs/system-design/clinical/ncit/architecture/system-architecture.md`
- **Experiments / CI Hooks:**
  - Backend smoke in `dfps_test_suite/tests/integration/vector_mapping.rs` with feature flags
  - Capacity drift snapshot per namespace during CI
- **Interfaces & Contracts:**
  - Traits: `VectorStore`
  - CLIs: `dfps_cli build-vector-index`
  - Env: `DFPS_VECTOR_URL`, `DFPS_VECTOR_BACKEND`, `DFPS_VECTOR_NAMESPACE`

### VEC-03 – Reference index builder
Provide a deterministic index builder CLI that loads NCIt/UMLS references, generates embeddings, and (re)builds the backend namespace safely (Targets A1/A3/D).
- Implementation
  - [x] Add `dfps_cli build-vector-index` (or `map-codes --build-index`) to bulk-index embeddings with namespace guardrails and reject dimension mismatches.
  - [x] Pin `embedding_version`, allow seed/dim caps (`--embedding-version`, `--max-dim`), and enforce idempotent rebuild (truncate/upsert per backend).
  - [x] Emit summary stats (count, mean/median norm, participation ratio) to stdout and structured logs; store metadata alongside index.
- Tests/docs
  - [x] Integration test against mock store to confirm deterministic embeddings for a seed, stable ordering, and identical payload on re-run; refuse empty namespace.
  - [x] Document CLI examples in the quickstart, including where metrics are recorded.

#### Cross-Cohesion

- **Engineering Targets:** A1, A3, D
- **Crates & Paths:**
  - `lib/app/cli` (`dfps_cli`)
  - `lib/platform/vector_store` (`dfps_vector_store`)
- **Shared Metrics & Signals:**
  - `geom_rm`, `geom_dm`, `geom_centroid_cos`
  - `vector_queries`, `vector_fallbacks`
- **Docs & Kanbans Touched:**
  - `docs/runbook/vector-store-quickstart.md`
  - `docs/system-design/clinical/ncit/concepts/vector-layer.md`
- **Experiments / CI Hooks:**
  - `dfps_cli build-vector-index` dry-run + idempotency check in CI
  - Snapshot of embedding norms and participation ratio for capacity drift
- **Interfaces & Contracts:**
  - CLIs: `dfps_cli build-vector-index`
  - Env: `DFPS_VECTOR_ENABLED`, `DFPS_VECTOR_NAMESPACE`

### VEC-04 – MappingEngine integration & feature flags
Wire the optional backend ranker into `MappingEngine` with feature flags, deterministic fallback, and score-fusion hooks (Targets B/D with A3 observability).
- Implementation
  - [x] Accept `VectorRankerBackend` alongside `VectorRankerMock`; keep `default_engine()` offline-only and provide `vector_engine(store)` when `DFPS_VECTOR_ENABLED=true`.
  - [x] In `map_staging_codes_with_summary`, route to backend when healthy else fall back to lexical+mock deterministically; log vector vs lexical score gaps and centroid similarity to avoid false merges.
  - [x] Add weighted fusion/reranker hook with configurable weights and guardrails on slowdown vs lexical-only.
- Tests/docs
  - [x] Unit/integration tests: offline path parity with baseline; vector-enabled path shows recall/precision uplift on PET/CT fixture with deterministic seeds.
  - [x] Env toggle tests proving `DFPS_VECTOR_ENABLED=false` bypasses network calls and increments `vector_fallbacks`; document latency budget and acceptable slowdown in `dfps_mapping` docs.

#### Cross-Cohesion

- **Engineering Targets:** A3, B, D
- **Crates & Paths:**
  - `lib/domain/mapping` (`dfps_mapping`)
  - `lib/platform/vector_store` (`dfps_vector_store`)
- **Shared Metrics & Signals:**
  - `auto_mapped`, `needs_review`, `no_match`
  - `vector_hits`, `vector_fallbacks`, `vector_latency_ms_p95`
- **Docs & Kanbans Touched:**
  - `docs/system-design/clinical/ncit/behavior/sequence-servicerequest.md`
  - `docs/kanban/feature/mvp/013-mapping-vector-backend.md`
- **Experiments / CI Hooks:**
  - Regression suites comparing lexical vs vector-enabled mapping in `dfps_test_suite`
  - CI alert when mapping recall drops or latency exceeds budget
- **Interfaces & Contracts:**
  - Traits: `CandidateRanker`
  - CLIs: `dfps_cli map-codes`, `dfps_cli map-bundles`
  - Env: `DFPS_VECTOR_ENABLED`

----

## Configuration
| Variable | Example | Purpose |
| ------------------------------- | ---------------------------------------------------------------------- | -------------------------- |
| `DFPS_VECTOR_ENABLED` | `true` | Toggle real vector backend |
| `DFPS_VECTOR_BACKEND` | `pgvector` / `qdrant` / `milvus` / `mock` | Select backend |
| `DFPS_VECTOR_URL` | `postgres://...` or `http://localhost:6333` or `tcp://localhost:19530` | Endpoint |
| `DFPS_VECTOR_NAMESPACE` | `ncit_dev` | Index namespace |
| `DFPS_VECTOR_POOL_MAX` | `10` | Pool size |
| `DFPS_VECTOR_HEALTH_TIMEOUT_MS` | `500` | Health probe budget |

## Rust API & Types (pseudocode)
```rust
pub enum VectorBackend { PgVector, Qdrant, Milvus, Mock }

pub struct VectorStoreConfig {
    pub backend: VectorBackend,
    pub url: String,
    pub namespace: String,
    pub pool_max: u32,
    pub health_timeout_ms: u64,
    pub enabled: bool,
}

#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn health(&self) -> Result<(), VectorError>;
    async fn index_items(&self, namespace: &str, items: &[(String, Vec<f32>)]) -> Result<(), VectorError>;
    async fn search(&self, namespace: &str, query_vec: &[f32], top_k: usize) -> Result<Vec<(String, f32)>, VectorError>;
}

pub struct VectorRankerBackend<S: VectorStore> { store: Arc<S>, top_k: usize }

impl<S: VectorStore> CandidateRanker for VectorRankerBackend<S> {
    async fn ranked_candidates(&self, code: &StgSrCodeExploded) -> Vec<MappingCandidate> {
        let query = embed(code); // deterministic embedding function
        match self.store.search(&namespace(code), &query, self.top_k).await {
            Ok(rows) => rows.into_iter().map(to_candidate).collect(),
            Err(err) => { emit_fallback_metric(err); VectorRankerMock::default().ranked_candidates(code) }
        }
    }
}
```

## Backends (pgvector/Qdrant/Milvus)
| Backend | Pros | Cons |
| --- | --- | --- |
| pgvector | FOSS, simple schema, runs with Postgres tooling | ANN options limited; tuning required for large dims |
| Qdrant | FOSS, HTTP/gRPC, rich ANN | Extra service to operate; needs collection bootstrap |
| Milvus | FOSS, scalable ANN | Heavier ops footprint; Java/Go dependencies |

Example pgvector DDL (example):
```sql
CREATE EXTENSION IF NOT EXISTS vector;
CREATE TABLE IF NOT EXISTS ncit_vectors (
  namespace TEXT NOT NULL,
  ref_id TEXT NOT NULL,
  embedding VECTOR(768) NOT NULL,
  PRIMARY KEY (namespace, ref_id)
);
CREATE INDEX IF NOT EXISTS idx_ncit_vectors_embedding
  ON ncit_vectors USING ivfflat (embedding vector_l2_ops) WITH (lists = 100);
```

## CLI & Workflows
- Build index (idempotent):
```bash
cd code
DFPS_VECTOR_ENABLED=true DFPS_VECTOR_BACKEND=pgvector DFPS_VECTOR_NAMESPACE=ncit_dev \
  DFPS_VECTOR_URL=postgres://vector:vector@localhost:5432/vector \
  cargo run -p dfps_cli -- build-vector-index --force-rebuild
```
- Map codes with vector backend:
```bash
cd code
DFPS_VECTOR_ENABLED=true DFPS_VECTOR_BACKEND=qdrant DFPS_VECTOR_URL=http://localhost:6333 \
  DFPS_VECTOR_NAMESPACE=ncit_dev \
  cargo run -p dfps_cli -- map-codes --explain-top 5 ./codes.ndjson
```
- Idempotency: pgvector path may truncate then bulk insert; Qdrant path uses upsert with collection reset flag; controlled via `--force-rebuild`.

## Fallback Behavior
| Condition | Behavior | Metrics | User note |
| --------------------------- | ----------------- | -------------------- | -------------------- |
| `DFPS_VECTOR_ENABLED=false` | Use mock only | `vector_fallbacks++` | “vector disabled” |
| Health probe fails | Warn; mock | `vector_fallbacks++` | “vector unavailable” |
| Search timeout | One retry → mock | `vector_fallbacks++` | include error code |
| Index missing | Log; lexical+mock | `vector_fallbacks++` | “index missing” |

## Tests & Observability
- Integration: `dfps_test_suite/tests/integration/vector_mapping.rs` boots Docker pgvector/Qdrant or test double; asserts higher recall than mock and deterministic output when disabled.
- Metrics: add `vector_queries`, `vector_hits`, `vector_fallbacks`, mean/p95 latency; exposed via `dfps_observability`.
- Logs: connectivity failures, index build start/finish, per-namespace counts; structured with namespace/backend tags.

## Runbooks & Cross‑References
- FHIR overview: `../../system-design/clinical/fhir/overview.md`
- NCIt architecture: `../../system-design/clinical/ncit/architecture/system-architecture.md`
- Vector layer (new): `../../system-design/clinical/ncit/concepts/vector-layer.md`
- Runbook (new): `../../runbook/vector-store-quickstart.md`

## Risk Log & Mitigations
- Backend unavailability → deterministic fallback + metrics + CI smoke with `DFPS_VECTOR_ENABLED=false`.
- Embedding drift → pin `embedding_version` in index metadata; rebuild on version change.
- Namespace collisions → PK `(namespace, ref_id)`; CLI validates namespace and refuses empty.

## Change Management
- Feature flags: `backend-pgvector`, `backend-qdrant`, `backend-milvus`, `mock`.
- Migration/bootstrap: run DDL or collection init once per namespace; use health probe before enabling.
- Rollout: dev → CI → staging; enable flag after quickstart passes on clean machine.

## Acceptance Criteria
- `dfps_mapping` can run in two modes:
  - **Offline**: pure Rust, no external services (current behavior).
  - **Vector-enabled**: leverages a real vector DB for candidate ranking.
- CLIs (`map_codes`, `map_bundles`) expose a clear UX for enabling/disabling vector search.
- Tests validate deterministic behavior in offline mode and improved ranking in vector-enabled mode.
- Capacity/latency guardrails in CI enforce geometry/capacity thresholds (`geom_rm_sqrt_dm` drift ≤ 10%, `cap_alpha_sim` ≥ baseline − 5%) and mapping uplift (≥ +3% recall on PET/CT fixture) with `vector_queries`/`vector_fallbacks`/latency metrics emitted.
- Failure of the vector backend does **not** crash the pipeline; it falls back cleanly to lexical + mock vector rankers.

## Out of Scope
- Online training / incremental embedding updates.
- Multi-tenant / sharded vector clusters beyond a single-namespace MVP.

## Definition of Done
- Tests under `dfps_test_suite` pass.
- CLIs show clear UX flags.
- New docs exist and link correctly.
