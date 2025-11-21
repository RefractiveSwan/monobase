# Mesh Node Quickstart

**Path:** `code/docs/runbook/mesh-node-quickstart.md`  
**Scope:** Standalone node deployment (no hub)  
**Audience:** DevOps, developers

This runbook describes how to deploy a **standalone mesh node** using the current crates (`dfps_api`, `dfps_datamart`, `dfps_vector_store`). Once `dfps_mesh_node` is extracted (Phase 4 of MESH-025), this runbook will be updated.

---

## Prerequisites

- Rust toolchain (>=1.70)
- SQLite (bundled) or Postgres
- Qdrant or PGVector (optional, for vector search)
- Environment profiles configured (see `data/environment/`)

---

## Environment Profiles

The node uses `dfps_configuration` to load environment-specific settings. Profiles:

- **dev**: SQLite + in-memory vector store
- **test**: SQLite + mock vector store
- **staging**: Postgres + Qdrant
- **prod**: Postgres + Qdrant/PGVector

---

## Configuration

### Warehouse (SQLite)

Create `.env.domain.datamart.dev`:

```bash
DFPS_WAREHOUSE_URL=sqlite:./data/warehouse.db
DFPS_WAREHOUSE_MAX_CONNECTIONS=5
```

### Vector Store (Qdrant)

Create `.env.platform.vector_store.dev`:

```bash
DFPS_VECTOR_BACKEND=Qdrant
DFPS_VECTOR_URL=http://localhost:6333
DFPS_VECTOR_NAMESPACE=dev
DFPS_VECTOR_POOL_SIZE=5
DFPS_VECTOR_TIMEOUT_SECS=30
```

### API Server

Create `.env.app.web.api.dev`:

```bash
DFPS_API_HOST=0.0.0.0
DFPS_API_PORT=8080
```

### Single Env File (Alternative)

Instead of multiple files, create `.env.mesh.node.dev`:

```bash
# Node identity (optional, will generate UUID if missing)
DFPS_NODE_ID=node-dev-001

# Warehouse
DFPS_WAREHOUSE_URL=sqlite:./data/warehouse.db
DFPS_WAREHOUSE_MAX_CONNECTIONS=5

# Vector store
DFPS_VECTOR_BACKEND=Qdrant
DFPS_VECTOR_URL=http://localhost:6333
DFPS_VECTOR_NAMESPACE=dev

# API
DFPS_API_HOST=0.0.0.0
DFPS_API_PORT=8080

# Compliance
DFPS_COMPLIANCE_MODE=permissive

# Dataset root (for eval)
DFPS_DATASET_ROOT=./data/datasets
```

---

## Running the Node

### Start Dependencies

#### Qdrant (Docker)

```bash
docker run -d -p 6333:6333 -p 6334:6334 qdrant/qdrant
```

#### Postgres + PGVector (Alternative)

```bash
docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres ankane/pgvector
```

### Start the Node

```bash
# Set profile
export DFPS_ENV=dev

# Run node (via dfps_api)
cd code
cargo run -p dfps_api --bin dfps_api
```

**Output**:
```
INFO dfps_api: Starting API server at http://0.0.0.0:8080
INFO dfps_api: NodeDataPlane initialized
INFO dfps_datamart: Connected to warehouse: sqlite:./data/warehouse.db
INFO dfps_vector_store: Connected to Qdrant at http://localhost:6333
```

---

## Health Check

```bash
curl http://localhost:8080/health
```

**Response**:
```json
{
  "status": "healthy",
  "warehouse": "enabled",
  "vector_store": "online"
}
```

---

## Running a Mapping Job

### via HTTP API

```bash
curl -X POST http://localhost:8080/api/map-bundles \
  -H "Content-Type: application/json" \
  -d '[{
    "id": "req-001",
    "code": "12345",
    "system": "CPT",
    "text": "Chest X-ray"
  }]'
```

**Response**:
```json
{
  "results": [
    {
      "request_id": "req-001",
      "ncit_code": "C123456",
      "ncit_display_name": "Chest Radiography",
      "confidence_score": 0.95
    }
  ]
}
```

### via CLI

```bash
cargo run -p dfps_cli -- map --file data/bundles.ndjson --output results.ndjson
```

---

## Running Analytics

### NCIt Summary

```bash
curl http://localhost:8080/analytics/ncit-summary
```

**Response**:
```json
{
  "rows": [
    {"code": "C123456", "display_name": "Chest Radiography", "count": 42},
    {"code": "C789012", "display_name": "Blood Test", "count": 38}
  ]
}
```

### Cohort Explorer

```bash
curl "http://localhost:8080/analytics/cohort?category=imaging&min_confidence=0.8"
```

---

## Running Eval

```bash
curl -X POST http://localhost:8080/api/eval/run \
  -H "Content-Type: application/json" \
  -d '{"dataset_name": "baseline_v1", "top_k": 10}'
```

**Response**:
```json
{
  "dataset_name": "baseline_v1",
  "metrics": {
    "precision": 0.92,
    "recall": 0.89,
    "f1": 0.90
  }
}
```

---

## Monitoring

### Metrics Endpoint

```bash
curl http://localhost:8080/metrics/summary
```

**Response**:
```json
{
  "pipeline_runs": 150,
  "vector_queries": 450,
  "warehouse_persists": 150,
  "uptime_secs": 3600
}
```

---

## Troubleshooting

### Warehouse Disabled

**Symptom**: `DatamartError::Disabled` in logs

**Solution**: Check `DFPS_WAREHOUSE_URL` is set and database is accessible

```bash
sqlite3 data/warehouse.db ".tables"
```

### Vector Store Unavailable

**Symptom**: `VectorStoreError::BackendUnavailable`

**Solution**: Verify Qdrant/PGVector is running

```bash
curl http://localhost:6333/health  # Qdrant
# or
psql -h localhost -U postgres -c "SELECT * FROM pg_extension WHERE extname='vector';"  # PGVector
```

### Port Already in Use

**Symptom**: `Address already in use (os error 98)`

**Solution**: Change `DFPS_API_PORT` or kill existing process

```bash
export DFPS_API_PORT=8081
# or
lsof -ti:8080 | xargs kill
```

---

## Production Deployment

### Use Postgres

```bash
DFPS_WAREHOUSE_URL=postgres://user:pass@localhost:5432/warehouse
```

### Use PGVector

```bash
DFPS_VECTOR_BACKEND=PgVector
DFPS_VECTOR_URL=postgres://user:pass@localhost:5432/vector_db
```

### Enable Compliance

```bash
DFPS_COMPLIANCE_MODE=strict
DFPS_DP_EPSILON=1.0
```

### Use Systemd

Create `/etc/systemd/system/mesh-node.service`:

```ini
[Unit]
Description=Mesh Node
After=network.target

[Service]
Type=simple
User=meshnode
WorkingDirectory=/opt/mesh-node
Environment="DFPS_ENV=prod"
ExecStart=/opt/mesh-node/dfps_api
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable mesh-node
sudo systemctl start mesh-node
```

---

## References

- **Mesh Layout**: `docs/system-design/base/mesh-data-plane-layout.md`
- **Node Runtime**: `docs/system-design/mesh/node-runtime.md`
- **Migration Plan**: `docs/system-design/mesh/migration-plan.md`
