# Kanban — feature/mapping-vector-backend (013)
**Epic:** VEC‑013 – Mapping vector backend  
**Branch:** `feature/VEC-013-mapping-vector-backend` | **Target version:** `v0.1.0`  
**Status:** DOING | **Introduced:** `v0.1.0` | **Last updated:** 2025-11-16  
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

## Kanban
<details><summary>Raw Kanban source</summary>

# Kanban - feature/mapping-vector-backend (013)

**Theme:** External infra & heavy services - real vector DB / vector search
**Goal:** Pluggable vector-store backed ranker for `dfps_mapping` (e.g., pgvector/Qdrant/etc.), wired into `MappingEngine` and CLIs, with clean fallbacks when the backend is unavailable.

### Columns

* **TODO** – Not started yet
* **INPROGRESS** – In progress
* **REVIEW** – Needs code review / refactor / docs polish
* **DONE** – Completed

## TODO

### VEC-01 – VectorStore abstraction & wiring

* [ ] Add a `VectorStore` trait (and minimal `EmbeddingProvider` if needed) under a new platform crate:

  * `lib/app/web/backend/vector_store` -> crate `dfps_vector_store`
  * Trait operations:

    * [ ] `index_items(namespace, items: &[(id, text)]) -> Result<()>`
    * [ ] `search(namespace, query_vec, top_k) -> Result<Vec<(id, score)>>`
  * [ ] Expose a `VectorStoreConfig` struct driven by env (`DFPS_VECTOR_URL`, `DFPS_VECTOR_NAMESPACE`, `DFPS_VECTOR_BACKEND`).
* [ ] In `dfps_mapping`, introduce an optional `VectorRankerBackend` implementing `CandidateRanker` by calling `VectorStore::search`.

### VEC-02 – First concrete backend (pgvector or Qdrant)

* [ ] Implement one concrete backend in `dfps_vector_store` (behind a feature flag, e.g., `backend-pgvector` or `backend-qdrant`):

  * [ ] Connection pool + health probe.
  * [ ] Schema / collection layout for reference codes (namespace per code system / project).
* [ ] Add namespace-aware env wiring:

  * [ ] `.env.domain.mapping.dev` / `.env.platform.vector_store.dev` templates in `data/environment/`.
  * [ ] Document minimal setup in a short runbook section (`docs/runbook/vector-store-quickstart.md`).

### VEC-03 – Reference index builder

* [ ] Add a `dfps_cli` subcommand:

  * `dfps_cli build-vector-index` (or `map-codes --build-index` mode) that:

    * [ ] Loads reference codes from `dfps_mapping::load_umls_xrefs()` + NCIt concepts.
    * [ ] Generates embeddings using your existing pipeline (e.g., TF-IDF/SVD or an external embedder).
    * [ ] Calls `VectorStore::index_items` to (re)build the index for the configured namespace.
* [ ] Ensure it is **idempotent** and safe for local rebuilds (truncate & repopulate or upsert-only, depending on backend).

### VEC-04 – MappingEngine integration & feature flags

* [ ] Extend `MappingEngine` to accept an optional `VectorRankerBackend` in addition to the existing `VectorRankerMock`:

  * [ ] `default_engine()` stays pure-Rust + mock (no network) for tests.
  * [ ] Introduce `vector_engine(store: Arc<dyn VectorStore>) -> MappingEngine<LexicalRanker, VectorRankerBackend>`.
* [ ] Add configuration in mapping pipeline:

  * [ ] `map_staging_codes_with_summary` consults env (`DFPS_VECTOR_ENABLED`) to decide whether to:

    * [ ] Use `VectorRankerBackend` (real vector DB).
    * [ ] Fall back to pure `VectorRankerMock` deterministically when unavailable.

### VEC-05 – Tests & observability

* [ ] Add integration tests in `dfps_test_suite` under `tests/integration/vector_mapping.rs` that:

  * [ ] Stand up a test VectorStore (either real backend in Docker, or a test double).
  * [ ] Compare mapping quality vs. baseline (mock vector ranker) on a small PET/CT fixture.
  * [ ] Assert deterministic behavior when the backend is disabled.
* [ ] Extend `PipelineMetrics` or add a new `VectorMetrics` struct to log:

  * [ ] `vector_queries`, `vector_hits`, `vector_fallbacks`.
  * [ ] Mean search latency (ms) if available.
* [ ] Wire logging into `dfps_observability` for:

  * [ ] Backend connectivity failures.
  * [ ] Index build start/finish events.

### VEC-06 – Docs & runbooks

* [ ] Add `docs/system-design/clinical/ncit/concepts/vector-layer.md` describing:

  * [ ] How the vector store fits between staging codes and `MappingEngine`.
  * [ ] The fallback behavior when the backend is down.
* [ ] Add a runbook `docs/runbook/vector-store-quickstart.md` with:

  * [ ] Local setup instructions (e.g., `docker-compose` or `psql` DDL).
  * [ ] Example commands:

    * [ ] `dfps_cli build-vector-index`
    * [ ] `dfps_cli map-codes` with vector backend enabled.

</details>

### VEC-01 – VectorStore abstraction & wiring
- [ ] Define `dfps_vector_store` crate under `lib/platform/vector_store` with FOSS-only dependencies (pgvector/qdrant/milvus clients).
- [ ] Add `VectorStore` trait with `index_items` and `search`; optional `EmbeddingProvider` wrapper.
- [ ] Provide `VectorStoreConfig` using env (`DFPS_VECTOR_URL`, `DFPS_VECTOR_NAMESPACE`, `DFPS_VECTOR_BACKEND`, `DFPS_VECTOR_POOL_MAX`, `DFPS_VECTOR_HEALTH_TIMEOUT_MS`, `DFPS_VECTOR_ENABLED`).
- [ ] In `dfps_mapping`, add optional `VectorRankerBackend` implementing `CandidateRanker` via `VectorStore::search`.
- Notes: trait must be `Send + Sync`; namespace required on all calls; deterministic ordering on equal scores; license posture GPLv3/FOSS-only.
- Tie-in: Keeps vector layer geometry observable (e.g., radius/dimension proxies per namespace) and enables capacity-aware rankers (Engineering Targets A1–A3, B). Namespace scoping lets us monitor centroid overlap drift and fall back cleanly when capacity collapses.
- [ ] Add capacity/correlation metrics per namespace (e.g., radius proxy `||v||_2`, effective dim via participation ratio) exposed through the trait.
- [ ] Document deterministic embedding function and complexity expectations alongside trait docs in `lib/platform/vector_store`.
- [ ] Wire env parsing and validation into `dfps_configuration` consumers; add unit tests for config parsing and deterministic ordering ties.
- [ ] Add a mock `VectorStore` impl for tests that records query counts to validate `vector_queries`/`vector_fallbacks`.

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

### VEC-02 – First concrete backend (pgvector or Qdrant)
- [ ] Implement one backend behind `backend-pgvector` or `backend-qdrant` feature; pool + health probe.
- [ ] Schema/collection keyed by `(namespace, ref_id)` to avoid collisions.
- [ ] Add `.env.domain.mapping.dev` and `.env.platform.vector_store.dev` in `data/environment/`.
- [ ] Document setup in `docs/runbook/vector-store-quickstart.md`; include FOSS-only dependency statement.
- Tie-in: Backend choice impacts curvature/neighbor quality; track search latency vs. recall uplift to ensure manifold capacity is not degraded by ANN parameters (Targets A2, B, C). Health probes prevent geometry skew from partial indexes.
- [ ] Define backend-specific collection/DDL bootstrap scripts and index parameters (dimensionality, metric type) with comments on expected `O(n log n)` build vs `O(kd)` query cost.
- [ ] Add backend health check integration tests (feature-gated) to assert creation and teardown per namespace.
- [ ] Record backend metadata (`backend`, `dim`, `metric`, `embedding_version`) in the index for observability and drift detection.
- [ ] Add CI skip path when backend features are disabled; document dev Docker compose for pgvector/Qdrant.

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
- [ ] Add `dfps_cli build-vector-index` (or `map-codes --build-index`) that loads UMLS xrefs + NCIt concepts and bulk-indexes embeddings.
- [ ] Deterministic embeddings (TF-IDF/SVD or existing embedder) with pinned `embedding_version`.
- [ ] Idempotent rebuild (truncate or upsert per backend); namespace-scoped guardrails.
- Tie-in: Embedding generation governs manifold radius and effective dimension; pinning versions lets us watch capacity drift and centroid correlations across rebuilds (Targets A1–A3, D). Idempotency keeps graph geometry stable across reruns.
- [ ] Add CLI flags for `--embedding-version` and `--max-dim` to bound `D_M` and track in index metadata.
- [ ] Emit summary metrics after build: count, mean/median norm, participation ratio; write to stdout and structured log.
- [ ] Integration test: run builder against mock store, ensure deterministic embeddings given seed and stable ordering; assert idempotent re-run produces identical payload.
- [ ] Guardrails to refuse empty namespace or mismatched embedding dimension vs. backend collection.

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
- [ ] Extend `MappingEngine` to accept `VectorRankerBackend` alongside mock.
- [ ] `default_engine()` remains offline mock for tests; `vector_engine(store)` uses real backend when `DFPS_VECTOR_ENABLED=true`.
- [ ] `map_staging_codes_with_summary` checks env; if disabled/unhealthy, falls back deterministically to lexical + mock and logs fallback.
- Tie-in: Blending lexical and vector rankers should increase separability when manifold capacity is sufficient; fallback keeps classification stable when curvature/coverage degrade (Targets B, D). Feature flags allow CI to track capacity-aware uplift without breaking offline mode.
- [ ] Add weighted fusion or reranker hook that logs vector vs lexical score gaps; monitor centroid similarity for false merges.
- [ ] Unit/integration tests: offline path parity with baseline; vector-enabled path shows recall/precision uplift on PET/CT fixture with deterministic seeds.
- [ ] Env-driven toggles verified via tests to ensure `DFPS_VECTOR_ENABLED=false` bypasses network calls and increments `vector_fallbacks`.
- [ ] Document expected latency budget per query and allowable slowdown vs. pure lexical in `dfps_mapping` docs.

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

### VEC-05 – Tests & observability
- [ ] Add `dfps_test_suite/tests/integration/vector_mapping.rs` using Dockerized backend or test double; assert uplift vs mock and deterministic offline path.
- [ ] Metrics: `vector_queries`, `vector_hits`, `vector_fallbacks`, mean/p95 latency; surfaced via `dfps_observability`.
- [ ] Logs: connectivity failures, index build start/finish, per-namespace counts; structured fields (`backend`, `namespace`, `duration_ms`).
- Tie-in: Observability must capture capacity proxies (norms, participation ratios) and correlation drift; tests ensure mapping quality tracks capacity and catches regressions (Targets A3, B, D).
- [ ] Add test asserting capacity proxy (e.g., average norm) remains within tolerance between runs for same seed.
- [ ] Emit histogram snapshots for score distributions and hit@k per backend; ensure logs include timeout/error codes.
- [ ] Add CI gating to fail if vector-enabled recall falls below baseline by >X% or if latency exceeds budget.
- [ ] Provide Prometheus-friendly metrics wiring in `dfps_observability` for vector counters and capacity proxies.

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
- [ ] Author `docs/system-design/clinical/ncit/concepts/vector-layer.md` describing vector layer and fallback.
- [ ] Add `docs/runbook/vector-store-quickstart.md` with local setup and CLI examples.
- [ ] Cross-link FHIR overview and NCIt architecture; note GPLv3/FOSS-only dependency posture.
- Tie-in: Docs should clarify how vector geometry (radius/dimension, centroid overlap) affects mapping states and how Leiden/Louvain choices alter graph conditioning (Targets A–D). Runbook must outline how to monitor capacity drift and switch to fallback safely.
- [ ] Include a minimal “capacity checklist” in the concept doc: record embedding_version, dim, norm stats, and community health notes.
- [ ] Add troubleshooting steps for backend downtime and capacity regressions (switch to mock, rebuild index).
- [ ] Document how to run eval harness to compare vector-enabled vs mock mapping quality, including expected metrics.
- [ ] Note FOSS-only dependencies and configuration snippets for pgvector and Qdrant in the quickstart.

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

## Configuration
| Variable | Example | Purpose |
| ------------------------------- | ---------------------------------------------------------------------- | -------------------------- |
| `DFPS_VECTOR_ENABLED` | `true` | Toggle real vector backend |
| `DFPS_VECTOR_BACKEND` | `pgvector` | `qdrant` | `milvus` | `mock` | Select backend |
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
- Failure of the vector backend does **not** crash the pipeline; it falls back cleanly to lexical + mock vector rankers.

## Out of Scope
- Online training / incremental embedding updates.
- Multi-tenant / sharded vector clusters beyond a single-namespace MVP.

## Definition of Done
- Tests under `dfps_test_suite` pass.
- CLIs show clear UX flags.
- New docs exist and link correctly.
