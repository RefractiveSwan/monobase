# Mesh Hub Quickstart (Design-Level)

**Path:** `code/docs/runbook/mesh-hub-quickstart.md`  
**Scope:** Federated hub deployment (orchestrates nodes)  
**Audience:** Research coordinators, DevOps  
**Status:** Design-level (implementation pending)

This runbook describes how to deploy a **mesh hub** that coordinates multiple nodes for federated research. This is currently a **design-level** document; implementation will follow in a future epic.

---

## Prerequisites

- Multiple mesh nodes deployed (see `mesh-node-quickstart.md`)
- Redis (for node registry + job queue)
- Postgres (for reporting warehouse)

---

## Architecture

The hub consists of:

1. **NodeRegistry**: Tracks available nodes and their capabilities
2. **JobQueue**: Dispatches jobs to nodes, collects results
3. **Reporting Warehouse**: Aggregates node results (read-only)
4. **HTTP API**: Exposes federated analytics/eval endpoints

---

## Configuration (Planned)

### Hub Identity

```bash
refractive_swan_HUB_ID=hub-research-001
```

### Node Registry (Redis)

```bash
refractive_swan_REDIS_URL=redis://localhost:6379
refractive_swan_REDIS_DB=0
```

### Known Nodes

```bash
refractive_swan_NODE_URLS=http://node-a:8080,http://node-b:8080,http://node-c:8080
```

### Reporting Warehouse

```bash
refractive_swan_HUB_WAREHOUSE_URL=postgres://user:pass@localhost:5432/hub_warehouse
```

### HTTP API

```bash
refractive_swan_HUB_HOST=0.0.0.0
refractive_swan_HUB_PORT=9000
```

---

## Running the Hub (Future)

```bash
# Set profile
export refractive_swan_ENV=prod

# Run hub
cd code
cargo run -p refractive_swan_mesh_hub --bin refractive_swan_mesh_hub
```

**Output**:
```
INFO refractive_swan_mesh_hub: Starting hub at http://0.0.0.0:9000
INFO refractive_swan_mesh_hub: Registered 3 nodes
INFO refractive_swan_mesh_hub: JobQueue initialized
```

---

## Hub Operations

### Node Registration

Nodes can register themselves with the hub:

```bash
# Node sends POST /hub/register
curl -X POST http://hub:9000/hub/register \
  -H "Content-Type: application/json" \
  -d '{
    "node_id": "node-a",
    "url": "http://node-a:8080",
    "capabilities": {
      "vector_backend": "Qdrant",
      "warehouse_backend": "Postgres",
      "compliance_mode": "strict",
      "max_dataset_size": 1000000
    }
  }'
```

**Response**:
```json
{
  "status": "registered",
  "hub_id": "hub-research-001"
}
```

### Node Discovery

Hub periodically health-checks nodes:

```bash
# Hub calls GET /mesh/health on each node
for node in $(cat nodes.txt); do
  curl $node/mesh/health
done
```

---

## Federated Analytics

### Global NCIt Summary

```bash
curl http://hub:9000/federated/analytics/ncit-summary
```

**Hub Workflow**:
1. Dispatch `MeshJobDescriptor` to all online nodes
2. Collect `MeshJobResult` from each node
3. Aggregate results (sum counts per NCIt code)
4. Return aggregated response

**Response**:
```json
{
  "rows": [
    {"code": "C123456", "display_name": "Chest Radiography", "count": 120},
    {"code": "C789012", "display_name": "Blood Test", "count": 95}
  ],
  "node_count": 3
}
```

### Cross-Node Cohort

```bash
curl "http://hub:9000/federated/analytics/cohort?category=imaging"
```

**Hub Workflow**:
1. Dispatch cohort query to all nodes
2. Collect results (with DP noise applied by nodes)
3. Merge cohorts (union of rows)
4. Return merged response

---

## Federated Eval

### Baseline Eval Across Nodes

```bash
curl -X POST http://hub:9000/federated/eval/run \
  -H "Content-Type: application/json" \
  -d '{"dataset_name": "baseline_v1", "top_k": 10}'
```

**Hub Workflow**:
1. Dispatch eval job to all nodes with dataset
2. Collect eval metrics from each node
3. Aggregate (average precision, recall, F1)
4. Return federated report

**Response**:
```json
{
  "dataset_name": "baseline_v1",
  "avg_precision": 0.91,
  "avg_recall": 0.88,
  "avg_f1": 0.89,
  "node_count": 3,
  "node_results": [
    {"node_id": "node-a", "precision": 0.92, "recall": 0.89},
    {"node_id": "node-b", "precision": 0.90, "recall": 0.87},
    {"node_id": "node-c", "precision": 0.91, "recall": 0.88}
  ]
}
```

---

## Governance

Hub enforces governance via `refractive_swan_mesh_governance`:

### DP Budget Check

Before dispatching export jobs, hub checks node DP budgets:

```rust
// Pseudocode
for node in nodes {
    let policy = governance.get_policy(node.id)?;
    if policy.dp_budget_consumed >= policy.dp_budget_daily {
        // Skip node (budget exceeded)
    }
}
```

### Job Approval

Hub can require manual approval for certain job types:

```bash
curl -X POST http://hub:9000/federated/export \
  -d '{"target_nodes": ["node-a"], "format": "parquet"}'
```

**Response**:
```json
{
  "status": "pending_approval",
  "approval_url": "http://hub:9000/admin/approve/job-123"
}
```

---

## Monitoring

### Hub Metrics

```bash
curl http://hub:9000/metrics
```

**Response**:
```json
{
  "total_nodes": 3,
  "online_nodes": 3,
  "jobs_dispatched": 45,
  "jobs_completed": 42,
  "jobs_failed": 3,
  "avg_job_duration_ms": 1250
}
```

### Node Health

```bash
curl http://hub:9000/admin/nodes
```

**Response**:
```json
{
  "nodes": [
    {"node_id": "node-a", "status": "online", "last_seen": "2025-11-21T12:00:00Z"},
    {"node_id": "node-b", "status": "online", "last_seen": "2025-11-21T12:00:05Z"},
    {"node_id": "node-c", "status": "degraded", "last_seen": "2025-11-21T11:55:00Z"}
  ]
}
```

---

## Future Enhancements

### Federated Learning

Hub coordinates FL rounds:

1. Hub sends model to nodes
2. Nodes train locally, send gradients back
3. Hub aggregates gradients (FedAvg)
4. Hub broadcasts updated model

### Differential Privacy Aggregation

Hub applies DP noise to aggregated results for additional privacy:

```rust
// Aggregate node results
let total_count = nodes.iter().map(|n| n.count).sum();

// Apply DP noise at hub level
let noised_count = dp_laplace(total_count, epsilon: 0.5);
```

---

## Troubleshooting

### Node Unreachable

**Symptom**: Hub logs `NodeUnavailable` errors

**Solution**: Check node health, network connectivity

```bash
curl http://node-a:8080/mesh/health
```

### Job Timeout

**Symptom**: Jobs stuck in `InProgress` status

**Solution**: Increase `refractive_swan_HUB_JOB_TIMEOUT_SECS`

### Redis Connection Failed

**Symptom**: `Failed to connect to registry`

**Solution**: Verify Redis is running

```bash
redis-cli ping
```

---

## Production Deployment

### Use Kubernetes

Deploy hub + nodes as separate services:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mesh-hub
spec:
  replicas: 1
  template:
    spec:
      containers:
      - name: hub
        image: mesh-hub:latest
        env:
        - name: refractive_swan_ENV
          value: "prod"
        - name: refractive_swan_NODE_URLS
          value: "http://node-a-svc:8080,http://node-b-svc:8080"
```

### Use Load Balancer

If hub needs HA:

```bash
kubectl expose deployment mesh-hub --type=LoadBalancer --port=9000
```

---

## References

- **Hub Design**: `lib/platform/mesh/hub/README.md`
- **Node Runtime**: `docs/system-design/mesh/node-runtime.md`
- **Governance**: `lib/platform/mesh/governance/README.md`
- **Mesh Contracts**: `lib/domain/contracts/src/mesh.rs`
