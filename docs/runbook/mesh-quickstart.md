# Mesh Dev Quickstart (2 nodes + 1 hub)

**Path:** `code/docs/runbook/mesh-quickstart.md`  
**Scope:** Local multi-node mesh (node-a, node-b, hub) for dashboard/demo  
**Audience:** Developers running the mesh dashboard and hub jobs locally

---

## Topology

- `node-a` – API at `http://127.0.0.1:8081`, SQLite mart, vector disabled, cache in-memory.
- `node-b` – API at `http://127.0.0.1:8082`, SQLite mart, vector disabled, cache in-memory.
- `hub-1` – API at `http://127.0.0.1:8080`, hub endpoints enabled (`/hub/jobs/*`).
- `redis` – shared infra (future registry/cache; optional for now).

Env namespaces:
- Nodes: `app.web.api.node_a`, `app.web.api.node_b`
- Hub: `platform.mesh.hub` (shared), hub process uses `refractive_swan_API_ENABLE_HUB=true`.

---

## Start the stack (docker-compose)

```bash
cd code
cargo make mesh-dev-up     # runs docker compose -f ci/mesh-dev/docker-compose.yml up
```

Services:
- `node-a`: port 8081
- `node-b`: port 8082
- `hub`: port 8080
- `redis`: port 6379

Stop:

```bash
cargo make mesh-dev-down   # docker compose down
```

Notes:
- The compose file mounts the repo into each container; builds use `cargo run` inside `rustlang/rust:nightly` (edition 2024 support).
- SQLite files live under `/tmp/node-a.db`, `/tmp/node-b.db`, `/tmp/hub.db` inside containers (compose creates the parent path on startup).
- Each service uses its own `CARGO_TARGET_DIR` under `/tmp/mesh-target/*` to avoid Cargo lock contention in dev.
- Vector is disabled for simplicity; cache backend is in-memory.
- Hub seeds nodes via `refractive_swan_HUB_NODES=node-a=http://node-a:8081,node-b=http://node-b:8082` so federated jobs dispatch to the node containers (not back to the hub itself).
- Docker must be in **Linux containers** mode and have access to `rust:1.82-bookworm`. On Windows, `cargo make mesh-dev-up` invokes `docker compose` via PowerShell.

---

## Manual (no Docker)

Open three shells:

```bash
# node-a
refractive_swan_API_HOST=0.0.0.0 \
refractive_swan_API_PORT=8081 \
refractive_swan_API_ENABLE_HUB=false \
refractive_swan_MESH_NODE_ID=node-a \
refractive_swan_WAREHOUSE_URL=sqlite:./target/node-a.db \
cargo run -p refractive_swan_api --bin refractive_swan_api

# node-b
refractive_swan_API_HOST=0.0.0.0 \
refractive_swan_API_PORT=8082 \
refractive_swan_API_ENABLE_HUB=false \
refractive_swan_MESH_NODE_ID=node-b \
refractive_swan_WAREHOUSE_URL=sqlite:./target/node-b.db \
cargo run -p refractive_swan_api --bin refractive_swan_api

# hub
refractive_swan_API_HOST=0.0.0.0 \
refractive_swan_API_PORT=8080 \
refractive_swan_API_ENABLE_HUB=true \
refractive_swan_API_BASE_URL=http://127.0.0.1:8080 \
refractive_swan_HUB_NODES=node-a=http://127.0.0.1:8081,node-b=http://127.0.0.1:8082 \
refractive_swan_WAREHOUSE_URL=sqlite:./target/hub.db \
cargo run -p refractive_swan_api --bin refractive_swan_api
```

---

## What to try

1) **Mapping at each node**  
   ```bash
   curl -X POST http://127.0.0.1:8081/api/map-bundles -d @sample.ndjson
   curl -X POST http://127.0.0.1:8082/api/map-bundles -d @sample.ndjson
   ```

2) **Mesh dashboard**  
   - Start frontend locally (`cargo run -p refractive_swan_web_frontend --bin refractive_swan_web_frontend`).
   - Open `http://127.0.0.1:8090/mesh`: see node list, hub analytics/eval panels, admin events.

3) **Federated NCIt summary**  
   ```bash
   curl -X POST http://127.0.0.1:8080/hub/jobs/analytics/ncit-summary
   ```
   - UI: check “Hub NCIt summary” panel (auto-refresh via HTMX).

4) **Federated eval**  
   ```bash
   curl -X POST "http://127.0.0.1:8080/hub/jobs/eval?dataset=bronze_pet_ct_small"
   ```
   - UI: see “Hub federated eval” panel; metrics cards update.

5) **Governance-denied scenario**  
   - Set `refractive_swan_DP_BUDGET_DAILY=0.0` on node-a, restart container.
   - Re-run hub eval; expect `MeshJobStatus::Denied` and DP budget alert in UI.

---

## Cleanup

```bash
cargo make mesh-dev-down
# If needed, prune volumes:
docker volume prune
```

---

## Troubleshooting

- **Prefer a dev container**: launch via VS Code / devcontainers using `.devcontainer/devcontainer.json`. The container already has nightly Rust, clippy/fmt, cargo-make, and Docker-in-Docker. It sets `CARGO_TARGET_DIR=/workspace/target` and `refractive_swan_EVAL_DATA_ROOT=/workspace/lib/domain/meta/evaluation/data` so the eval fixtures resolve consistently.
- Containers rebuilding slowly? Pre-build deps locally (`cargo build`) before `mesh-dev-up`.
- Ports in use? Adjust `refractive_swan_API_PORT` in `ci/mesh-dev/docker-compose.yml`.
- Missing datasets? Ensure `refractive_swan_EVAL_DATA_ROOT` is available to the frontend/API.
