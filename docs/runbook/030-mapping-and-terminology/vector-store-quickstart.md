# Vector Store Quickstart

Purpose: stand up a FOSS vector backend (pgvector, Qdrant, or Milvus), build the NCIt/UMLS index, and run mapping with vector search. GPLv3 posture; no closed SaaS.

Prerequisites: Docker + Docker Compose, Rust toolchain, `psql` for pgvector or access to Qdrant/Milvus containers. Set `refractive_swan_WORKSPACE_ROOT` to repo root when running CLIs.

## Quickstart (pgvector end-to-end)
1) Launch Postgres with pgvector:
   ```bash
   cat > docker-compose.pgvector.yml <<'YML'
   version: "3.8"
   services:
     pgvector:
       image: ankane/pgvector:0.6.2
       environment:
         POSTGRES_PASSWORD: vector
         POSTGRES_USER: vector
         POSTGRES_DB: vector
       ports: ["5432:5432"]
   YML
   docker compose -f docker-compose.pgvector.yml up -d
   ```
2) Create extension + table (example, 768-dim):
   ```bash
   psql postgres://vector:vector@localhost:5432/vector <<'SQL'
   CREATE EXTENSION IF NOT EXISTS vector;
   CREATE TABLE IF NOT EXISTS ncit_vectors (
     namespace TEXT NOT NULL,
     ref_id TEXT NOT NULL,
     embedding VECTOR(768) NOT NULL,
     source_version TEXT,
     embedding_version TEXT,
     PRIMARY KEY (namespace, ref_id)
   );
   CREATE INDEX IF NOT EXISTS idx_ncit_vectors_embedding
     ON ncit_vectors USING ivfflat (embedding vector_l2_ops) WITH (lists = 100);
   SQL
   ```
3) Env config:
   ```bash
   export refractive_swan_VECTOR_ENABLED=true
   export refractive_swan_VECTOR_BACKEND=pgvector
   export refractive_swan_VECTOR_URL=postgres://vector:vector@localhost:5432/vector
   export refractive_swan_VECTOR_NAMESPACE=ncit_dev
   export refractive_swan_VECTOR_POOL_MAX=10
   export refractive_swan_VECTOR_HEALTH_TIMEOUT_MS=500
   ```
   Note: the `build-vector-index` CLI currently supports the Qdrant/mock backends. For pgvector, keep the manual SQL bootstrap above and skip to mapping (or set `refractive_swan_VECTOR_BACKEND=mock` for a dry-run index build).
4) Build index:
   ```bash
   cd code
   cargo run -p refractive_swan_cli --features backend-pgvector -- build-vector-index \
     --namespace ncit_dev --limit 1000 \
     --embedding-version deterministic-hash-v1 --max-dim 768 --force-rebuild
   ```
   The CLI prints embedding stats (count/mean/median norm + participation ratio) and enforces dim/version guardrails; batches with mixed dims are rejected. Keep the stats output as a baseline for drift checks.
5) Map codes with vector search:
   ```bash
   cd code
   cargo run -p refractive_swan_cli -- map-codes --explain --explain-top 5 ./staging_codes.ndjson
   ```

## Alternates (service + env only)
- Qdrant:
  ```bash
  docker run -d --name qdrant -p 6333:6333 -p 6334:6334 qdrant/qdrant:v1.12.1
  export refractive_swan_VECTOR_ENABLED=true
  export refractive_swan_VECTOR_BACKEND=qdrant
  export refractive_swan_VECTOR_URL=http://localhost:6333
  export refractive_swan_VECTOR_NAMESPACE=ncit_dev
  ```
  Build or rebuild index (auto-creates collection if missing):
  ```bash
  cargo run -p refractive_swan_cli -- build-vector-index --namespace ncit_dev --limit 1000 --embedding-version deterministic-hash-v1 --max-dim 768 --force-rebuild
  ```
  Cost/guardrails: collection create is `O(1)` but initial HNSW build behaves like `O(n log n)`; searches are ~`O(kd)` per query. Set `refractive_swan_VECTOR_HEALTH_TIMEOUT_MS` to bound probes; if the backend is down, vector mode falls back to lexical+mock and increments `vector_fallbacks`.

- Milvus:
  ```bash
  cat > docker-compose.milvus.yml <<'YML'
  version: '3.8'
  services:
    milvus:
      image: milvusdb/milvus:v2.4.2
      ports: ["19530:19530","9091:9091"]
  YML
  docker compose -f docker-compose.milvus.yml up -d
  export refractive_swan_VECTOR_ENABLED=true
  export refractive_swan_VECTOR_BACKEND=milvus
  export refractive_swan_VECTOR_URL=tcp://localhost:19530
  export refractive_swan_VECTOR_NAMESPACE=ncit_dev
  ```
  Create collection (example via milvus-cli or SDK) with dim=768, index IVF/HNSW as needed.

## Eval harness vs mock
- Run vector-enabled vs mock to compare recall/precision (example: gold_pet_ct_small):
  ```bash
  refractive_swan_VECTOR_ENABLED=true refractive_swan_VECTOR_BACKEND=pgvector refractive_swan_VECTOR_URL=postgres://vector:vector@localhost:5433/vector \
    cargo run -p refractive_swan_cli --features backend-pgvector -- map-codes -- ./lib/domain/meta/evaluation/data/eval/gold_pet_ct_small.ndjson
  refractive_swan_VECTOR_ENABLED=false cargo run -p refractive_swan_cli -- map-codes -- ./lib/domain/meta/evaluation/data/eval/gold_pet_ct_small.ndjson
  ```
- Use `refractive_swan_test_suite/tests/integration/vector_mapping.rs` parity/recall guard as a CI signal; gate if vector-enabled recall drops >X% vs baseline or latency exceeds the recorded budget.
- For latency drift, watch `vector_latency_ms_p95` in logs; alert if it exceeds your budget.

## Troubleshooting
- Backend down or unhealthy: set `refractive_swan_VECTOR_ENABLED=false` to force deterministic lexical+mock fallback; `vector_fallbacks` should increment and logs should note the reason.
- Index drift or bad norms: re-run `build-vector-index` and compare embedding stats (norm/participation ratio) to prior runs; rebuild if `geom_rm_sqrt_dm` or `cap_alpha_sim` drifts beyond thresholds.
- Dimension mismatch errors: ensure all embeddings in a batch share the same `dim` (CLI rejects mixed dims).
- Collection missing: recreate via `build-vector-index` (pgvector) or `curl`/CLI (Qdrant); rerun after health check.

## Health Checks & Fallback Verification
- Health:
  - pgvector: `psql ... -c "SELECT 1"` and ensure `SELECT count(*) FROM ncit_vectors;`
  - Qdrant: `curl http://localhost:6333/collections`
  - Milvus: `grpcurl -plaintext localhost:19530 milvus.proto. milvus.proto.MilvusService/DescribeCollection` (or SDK call)
- Fallback: `export refractive_swan_VECTOR_ENABLED=false` then rerun `map-codes`; expect deterministic lexical+mock behavior and `vector_fallbacks` metric increments.
- Common errors:
  - `vector extension missing` → run `CREATE EXTENSION vector;`
  - `collection not found` → create collection/DDL first.
  - `timeout/health check failed` → verify URL/ports; falls back if enabled flag remains true but probes fail.

## Clean Up
- Stop services: `docker compose -f docker-compose.pgvector.yml down` (or qdrant/milvus equivalents).
- Drop data: remove volumes or drop tables/collections per backend if you want a fresh rebuild.
