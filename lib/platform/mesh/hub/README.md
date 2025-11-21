# dfps_mesh_hub

**Conceptual location:** `lib/platform/mesh/hub`  
**Current physical location:** Not yet implemented  
**Scope:** Research orchestrator, federated learning coordinator, cross-node job routing

This directory will host the **mesh hub runtime** that orchestrates jobs across multiple nodes, aggregates results, and manages the node registry.

---

## Purpose

`dfps_mesh_hub` provides:

1. **HubConfig**: List of node URLs/IDs, auth, timeouts
2. **NodeRegistry**: NodeId → NodeMetadata mapping
3. **JobQueue**: Distributes `MeshJobDescriptor` to nodes, collects `MeshJobResult`
4. **Aggregation**: Combines node results for global analytics

**Goal**: Enable federated research deployments where a central hub coordinates node-local computations without accessing raw data.

---

## Design

### HubConfig

```rust
#[derive(Clone, Debug)]
pub struct HubConfig {
    /// Hub identity
    pub hub_id: String,
    
    /// Known node URLs
    pub node_urls: Vec<String>,
    
    /// Auth token (if required)
    pub auth_token: Option<String>,
    
    /// Request timeout (seconds)
    pub timeout_secs: u64,
    
    /// Retry backoff (exponential)
    pub backoff_millis: u64,
    
    /// Max concurrent node requests
    pub max_concurrency: usize,
}
```

### NodeRegistry

```rust
use dfps_contracts::mesh::{MeshNodeId, NodeCapabilities};

#[derive(Clone, Debug)]
pub struct NodeMetadata {
    pub node_id: MeshNodeId,
    pub url: String,
    pub capabilities: NodeCapabilities,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub status: NodeStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeStatus {
    Online,
    Offline,
    Degraded,
}

pub struct NodeRegistry {
    nodes: HashMap<MeshNodeId, NodeMetadata>,
}

impl NodeRegistry {
    pub fn register(&mut self, metadata: NodeMetadata) {
        self.nodes.insert(metadata.node_id.clone(), metadata);
    }
    
    pub fn get(&self, node_id: &MeshNodeId) -> Option<&NodeMetadata> {
        self.nodes.get(node_id)
    }
    
    pub fn list_online(&self) -> Vec<&NodeMetadata> {
        self.nodes.values()
            .filter(|n| n.status == NodeStatus::Online)
            .collect()
    }
}
```

### JobQueue

```rust
use dfps_contracts::mesh::{MeshJobDescriptor, MeshJobResult};

pub struct JobQueue {
    registry: Arc<RwLock<NodeRegistry>>,
    http_client: reqwest::Client,
}

impl JobQueue {
    pub async fn dispatch(
        &self,
        job: MeshJobDescriptor,
        target_nodes: Option<Vec<MeshNodeId>>,
    ) -> Result<Vec<MeshJobResult>, HubError> {
        let nodes = if let Some(targets) = target_nodes {
            targets.into_iter()
                .filter_map(|id| self.registry.read().await.get(&id).cloned())
                .collect()
        } else {
            self.registry.read().await.list_online()
                .into_iter().cloned().collect()
        };
        
        let mut results = Vec::new();
        
        for node in nodes {
            let result = self.send_job_to_node(&node, &job).await?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    async fn send_job_to_node(
        &self,
        node: &NodeMetadata,
        job: &MeshJobDescriptor,
    ) -> Result<MeshJobResult, HubError> {
        let url = format!("{}/mesh/job", node.url);
        
        let response = self.http_client
            .post(&url)
            .json(job)
            .timeout(Duration::from_secs(30))
            .send()
            .await?;
        
        let result: MeshJobResult = response.json().await?;
        Ok(result)
    }
}
```

---

## Hub Operations

### Federated Analytics

```rust
pub async fn global_ncit_summary(
    &self,
    queue: &JobQueue,
) -> Result<AnalyticsSummaryResponse, HubError> {
    let job = MeshJobDescriptor {
        job_id: Uuid::new_v4().to_string(),
        job_type: MeshJobType::AnalyticsQuery,
        parameters: serde_json::json!({ "query_type": "ncit_summary" }),
        governance_context: None,
    };
    
    // Dispatch to all nodes
    let results = queue.dispatch(job, None).await?;
    
    // Aggregate results
    let mut aggregated = HashMap::new();
    
    for result in results {
        if result.status != MeshJobStatus::Success {
            continue; // Skip failed nodes
        }
        
        if let Some(output) = result.output {
            let summary: AnalyticsSummaryResponse = serde_json::from_value(output)?;
            for row in summary.rows {
                *aggregated.entry(row.code).or_insert(0) += row.count;
            }
        }
    }
    
    let rows = aggregated.into_iter()
        .map(|(code, count)| AnalyticsSummaryRow { code, count })
        .collect();
    
    Ok(AnalyticsSummaryResponse { rows })
}
```

### Federated Eval

```rust
pub async fn federated_eval(
    &self,
    queue: &JobQueue,
    dataset_name: &str,
) -> Result<FederatedEvalReport, HubError> {
    let job = MeshJobDescriptor {
        job_id: Uuid::new_v4().to_string(),
        job_type: MeshJobType::EvalDataset,
        parameters: serde_json::json!({ "dataset_name": dataset_name }),
        governance_context: None,
    };
    
    let results = queue.dispatch(job, None).await?;
    
    // Aggregate eval metrics across nodes
    let mut total_precision = 0.0;
    let mut total_recall = 0.0;
    let mut node_count = 0;
    
    for result in results {
        if result.status == MeshJobStatus::Success {
            if let Some(output) = result.output {
                let eval_response: EvalRunResponse = serde_json::from_value(output)?;
                total_precision += eval_response.metrics.precision;
                total_recall += eval_response.metrics.recall;
                node_count += 1;
            }
        }
    }
    
    Ok(FederatedEvalReport {
        avg_precision: total_precision / node_count as f64,
        avg_recall: total_recall / node_count as f64,
        node_count,
    })
}
```

---

## Node Registration

### Registration Flow

```rust
// Node sends capabilities to hub
pub async fn register_node(
    &self,
    registry: &mut NodeRegistry,
    capabilities: NodeCapabilities,
) -> Result<(), HubError> {
    let metadata = NodeMetadata {
        node_id: capabilities.node_id.clone(),
        url: capabilities.url.clone(),
        capabilities,
        last_seen: chrono::Utc::now(),
        status: NodeStatus::Online,
    };
    
    registry.register(metadata);
    Ok(())
}

// Hub periodically health-checks nodes
pub async fn health_check_nodes(
    &self,
    registry: &mut NodeRegistry,
) -> Result<(), HubError> {
    for (node_id, metadata) in registry.nodes.iter_mut() {
        let url = format!("{}/mesh/health", metadata.url);
        
        match self.http_client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                metadata.status = NodeStatus::Online;
                metadata.last_seen = chrono::Utc::now();
            }
            _ => {
                metadata.status = NodeStatus::Offline;
            }
        }
    }
    Ok(())
}
```

---

## How Hub Calls Node APIs

The hub uses existing node HTTP endpoints (from `dfps_api`, future `dfps_mesh_node`):

### Existing Endpoints (Reused)

- `POST /api/map-bundles` → mapping jobs
- `GET /analytics/ncit-summary` → analytics queries
- `POST /api/eval/run` → eval jobs

### New Mesh Endpoints

- `POST /mesh/job` → generic job dispatcher (takes `MeshJobDescriptor`)
- `GET /mesh/capabilities` → return `NodeCapabilities`
- `GET /mesh/health` → health check

**No breaking changes**: Existing apps/CLIs continue using `/api/*` and `/analytics/*`. Hub uses `/mesh/*` for coordination.

---

## Testing

### Unit Tests

- `NodeRegistry` registration/lookup
- Job queue dispatching logic
- Aggregation algorithms (NCIt summary, eval metrics)

### Integration Tests

- Mock node HTTP server
- Dispatch job → receive result
- Handle node failures (timeouts, errors)

### End-to-End Tests

- Real nodes + hub deployment
- Federated analytics (2+ nodes)
- Governance enforcement (hub denies export jobs)

---

## References

- **Mesh Contracts**: `lib/domain/contracts/src/mesh.rs`
- **Node Runtime**: `docs/system-design/mesh/node-runtime.md`
- **Governance**: `lib/platform/mesh/governance/README.md`
- **Mesh Layout**: `docs/system-design/base/mesh-data-plane-layout.md`
