# Vector Store Quickstart

Purpose: stand up a FOSS vector backend (pgvector, Qdrant, or Milvus), build the NCIt/UMLS index, and run mapping with vector search. GPLv3 posture; no closed SaaS.

Prerequisites: Docker + Docker Compose, Rust toolchain, `psql` for pgvector or access to Qdrant/Milvus containers. Set `DFPS_WORKSPACE_ROOT` to repo root when running CLIs.

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
   export DFPS_VECTOR_ENABLED=true
   export DFPS_VECTOR_BACKEND=pgvector
   export DFPS_VECTOR_URL=postgres://vector:vector@localhost:5432/vector
   export DFPS_VECTOR_NAMESPACE=ncit_dev
   export DFPS_VECTOR_POOL_MAX=10
   export DFPS_VECTOR_HEALTH_TIMEOUT_MS=500
   ```
4) Build index:
   ```bash
   cd code
   cargo run -p dfps_cli -- build-vector-index --namespace ncit_dev --source umls,ncit --force-rebuild
   ```
5) Map codes with vector search:
   ```bash
   cd code
   cargo run -p dfps_cli -- map-codes ./staging_codes.ndjson --top-k 25
   ```

## Alternates (service + env only)
- Qdrant:
  ```bash
  docker run -d --name qdrant -p 6333:6333 -p 6334:6334 qdrant/qdrant:v1.12.1
  export DFPS_VECTOR_ENABLED=true
  export DFPS_VECTOR_BACKEND=qdrant
  export DFPS_VECTOR_URL=http://localhost:6333
  export DFPS_VECTOR_NAMESPACE=ncit_dev
  ```
  Create collection once (example):
  ```bash
  curl -X PUT "http://localhost:6333/collections/ncit_dev" \
    -H 'Content-Type: application/json' \
    -d '{"vectors":{"size":768,"distance":"Cosine"}}'
  ```

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
  export DFPS_VECTOR_ENABLED=true
  export DFPS_VECTOR_BACKEND=milvus
  export DFPS_VECTOR_URL=tcp://localhost:19530
  export DFPS_VECTOR_NAMESPACE=ncit_dev
  ```
  Create collection (example via milvus-cli or SDK) with dim=768, index IVF/HNSW as needed.

## Health Checks & Fallback Verification
- Health:
  - pgvector: `psql ... -c "SELECT 1"` and ensure `SELECT count(*) FROM ncit_vectors;`
  - Qdrant: `curl http://localhost:6333/collections`
  - Milvus: `grpcurl -plaintext localhost:19530 milvus.proto. milvus.proto.MilvusService/DescribeCollection` (or SDK call)
- Fallback: `export DFPS_VECTOR_ENABLED=false` then rerun `map-codes`; expect deterministic lexical+mock behavior and `vector_fallbacks` metric increments.
- Common errors:
  - `vector extension missing` → run `CREATE EXTENSION vector;`
  - `collection not found` → create collection/DDL first.
  - `timeout/health check failed` → verify URL/ports; falls back if enabled flag remains true but probes fail.

## Clean Up
- Stop services: `docker compose -f docker-compose.pgvector.yml down` (or qdrant/milvus equivalents).
- Drop data: remove volumes or drop tables/collections per backend if you want a fresh rebuild.
