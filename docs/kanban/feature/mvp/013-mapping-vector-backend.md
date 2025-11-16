# Kanban — feature/mapping-vector-backend (013)
**Epic:** VEC‑013 – Mapping vector backend  
**Branch:** `feature/VEC-013-mapping-vector-backend` • **Target version:** `v0.1.0`  
**Status:** DOING • **Introduced:** `v0.1.0` • **Last updated:** 2025-11-16  
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

  * `lib/platform/vector_store` -> crate `dfps_vector_store`
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

### VEC-02 – First concrete backend (pgvector or Qdrant)
- [ ] Implement one backend behind `backend-pgvector` or `backend-qdrant` feature; pool + health probe.
- [ ] Schema/collection keyed by `(namespace, ref_id)` to avoid collisions.
- [ ] Add `.env.domain.mapping.dev` and `.env.platform.vector_store.dev` in `data/environment/`.
- [ ] Document setup in `docs/runbook/vector-store-quickstart.md`; include FOSS-only dependency statement.

### VEC-03 – Reference index builder
- [ ] Add `dfps_cli build-vector-index` (or `map-codes --build-index`) that loads UMLS xrefs + NCIt concepts and bulk-indexes embeddings.
- [ ] Deterministic embeddings (TF-IDF/SVD or existing embedder) with pinned `embedding_version`.
- [ ] Idempotent rebuild (truncate or upsert per backend); namespace-scoped guardrails.

### VEC-04 – MappingEngine integration & feature flags
- [ ] Extend `MappingEngine` to accept `VectorRankerBackend` alongside mock.
- [ ] `default_engine()` remains offline mock for tests; `vector_engine(store)` uses real backend when `DFPS_VECTOR_ENABLED=true`.
- [ ] `map_staging_codes_with_summary` checks env; if disabled/unhealthy, falls back deterministically to lexical + mock and logs fallback.

### VEC-05 – Tests & observability
- [ ] Add `dfps_test_suite/tests/integration/vector_mapping.rs` using Dockerized backend or test double; assert uplift vs mock and deterministic offline path.
- [ ] Metrics: `vector_queries`, `vector_hits`, `vector_fallbacks`, mean/p95 latency; surfaced via `dfps_observability`.
- [ ] Logs: connectivity failures, index build start/finish, per-namespace counts; structured fields (`backend`, `namespace`, `duration_ms`).

### VEC-06 – Docs & runbooks
- [ ] Author `docs/system-design/clinical/ncit/concepts/vector-layer.md` describing vector layer and fallback.
- [ ] Add `docs/runbook/vector-store-quickstart.md` with local setup and CLI examples.
- [ ] Cross-link FHIR overview and NCIt architecture; note GPLv3/FOSS-only dependency posture.

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
