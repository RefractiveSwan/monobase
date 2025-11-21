# refractive_swan_mesh_governance

**Conceptual location:** `lib/platform/mesh/governance`  
**Current physical location:** Not yet implemented  
**Scope:** Mesh-level policies, query governance, differential privacy budget management

This directory will host the **governance engine** that evaluates policy decisions for mesh operations, integrating with `refractive_swan_compliance` for DP enforcement and node-level access control.

---

## Purpose

`refractive_swan_mesh_governance` provides:

1. **QueryClass**: Categorize mesh jobs (MappingJob, AnalyticsJob, EvalJob, ExportJob, NodeIntrospection)
2. **QueryDescriptor**: Job parameters + expected cardinality
3. **GovernanceDecision**: Allow, Deny, AllowWithNoise
4. **NodePolicy**: Per-node capabilities + compliance mode

**Goal**: Centralize policy evaluation so nodes and hub make consistent governance decisions.

---

## Design

### QueryClass

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryClass {
    /// Mapping job (bundle → NCIt codes)
    MappingJob,
    
    /// Analytics query (NCIt summary, cohort)
    AnalyticsJob,
    
    /// Eval harness run
    EvalJob,
    
    /// Data export (snapshots to lake)
    ExportJob,
    
    /// Node introspection (metrics, health)
    NodeIntrospection,
}
```

### QueryDescriptor

```rust
#[derive(Clone, Debug)]
pub struct QueryDescriptor {
    /// Query classification
    pub class: QueryClass,
    
    /// Expected dataset size (rows)
    pub expected_cardinality: Option<u64>,
    
    /// Time range (for analytics/exports)
    pub time_range: Option<TimeRange>,
    
    /// Cohort filters (for analytics)
    pub filters: Option<serde_json::Value>,
    
    /// Requesting entity (node ID or external user)
    pub requester: String,
}
```

### GovernanceDecision

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum GovernanceDecision {
    /// Allow the query without modifications
    Allow,
    
    /// Deny the query (with reason)
    Deny(String),
    
    /// Allow with DP noise applied (epsilon budget)
    AllowWithNoise { epsilon: f64 },
}
```

### NodePolicy (mesh DTOs)

```rust
use refractive_swan_mesh_dto::NodeCapabilities;

#[derive(Clone, Debug)]
pub struct NodePolicy {
    /// Node capabilities (from mesh contracts)
    pub capabilities: NodeCapabilities,
    
    /// DP budget per day (epsilon)
    pub dp_budget_daily: f64,
    
    /// DP budget consumed today
    pub dp_budget_consumed: f64,
    
    /// Maximum query cardinality (no DP)
    pub max_cardinality_no_dp: u64,
    
    /// Export allowed (to lake)
    pub export_allowed: bool,
    
    /// Hub registration allowed
    pub hub_registration_allowed: bool,
}
```

---

## Governance Engine

### Trait

```rust
#[async_trait]
pub trait GovernanceEngine: Send + Sync {
    /// Evaluate a query descriptor against node policy.
    async fn evaluate(
        &self,
        descriptor: &QueryDescriptor,
        policy: &NodePolicy,
    ) -> Result<GovernanceDecision, GovernanceError>;
    
    /// Update DP budget after query execution.
    async fn consume_budget(
        &self,
        policy: &mut NodePolicy,
        epsilon_used: f64,
    ) -> Result<(), GovernanceError>;
}
```

### Implementation

```rust
pub struct DefaultGovernanceEngine {
    compliance: Arc<refractive_swan_compliance::Policy>,
}

impl GovernanceEngine for DefaultGovernanceEngine {
    async fn evaluate(
        &self,
        descriptor: &QueryDescriptor,
        policy: &NodePolicy,
    ) -> Result<GovernanceDecision, GovernanceError> {
        match descriptor.class {
            QueryClass::MappingJob => {
                // Always allow mapping jobs (no data export)
                Ok(GovernanceDecision::Allow)
            }
            QueryClass::AnalyticsJob => {
                // Check cardinality
                if let Some(cardinality) = descriptor.expected_cardinality {
                    if cardinality > policy.max_cardinality_no_dp {
                        // Require DP noise
                        if policy.dp_budget_consumed + 1.0 > policy.dp_budget_daily {
                            return Ok(GovernanceDecision::Deny(
                                "DP budget exceeded".to_string()
                            ));
                        }
                        Ok(GovernanceDecision::AllowWithNoise { epsilon: 1.0 })
                    } else {
                        Ok(GovernanceDecision::Allow)
                    }
                } else {
                    Ok(GovernanceDecision::Allow)
                }
            }
            QueryClass::ExportJob => {
                // Require DP noise for exports
                if !policy.export_allowed {
                    return Ok(GovernanceDecision::Deny("Exports disabled".to_string()));
                }
                if policy.dp_budget_consumed + 2.0 > policy.dp_budget_daily {
                    return Ok(GovernanceDecision::Deny("DP budget exceeded".to_string()));
                }
                Ok(GovernanceDecision::AllowWithNoise { epsilon: 2.0 })
            }
            QueryClass::EvalJob | QueryClass::NodeIntrospection => {
                Ok(GovernanceDecision::Allow)
            }
        }
    }

    async fn consume_budget(
        &self,
        policy: &mut NodePolicy,
        epsilon_used: f64,
    ) -> Result<(), GovernanceError> {
        policy.dp_budget_consumed += epsilon_used;
        Ok(())
    }
}
```

---

## Integration Points

### With refractive_swan_mesh_node

```rust
// In NodeDataPlane
pub async fn run_job_with_governance(
    &self,
    job: &MeshJobDescriptor,
    governance: &dyn GovernanceEngine,
    policy: &mut NodePolicy,
) -> Result<MeshJobResult, NodeError> {
    let descriptor = QueryDescriptor::from_job(job);
    
    let decision = governance.evaluate(&descriptor, policy).await?;
    
    match decision {
        GovernanceDecision::Allow => {
            let output = self.execute_job(job).await?;
            Ok(MeshJobResult::success(job.job_id.clone(), output))
        }
        GovernanceDecision::AllowWithNoise { epsilon } => {
            let raw_output = self.execute_job(job).await?;
            let noised_output = self.compliance_policy.apply_dp_noise(&raw_output, epsilon)?;
            governance.consume_budget(policy, epsilon).await?;
            Ok(MeshJobResult::success(job.job_id.clone(), noised_output))
        }
        GovernanceDecision::Deny(reason) => {
            Ok(MeshJobResult::denied(job.job_id.clone(), reason))
        }
    }
}
```

### With refractive_swan_compliance

Governance delegates DP enforcement to `refractive_swan_compliance`:

```rust
// In GovernanceEngine
async fn apply_dp_if_required(
    &self,
    data: &serde_json::Value,
    decision: &GovernanceDecision,
    compliance: &refractive_swan_compliance::Policy,
) -> Result<serde_json::Value, GovernanceError> {
    match decision {
        GovernanceDecision::AllowWithNoise { epsilon } => {
            compliance.apply_dp_noise(data, *epsilon)
        }
        _ => Ok(data.clone()),
    }
}
```

---

## DP Budget Management

### Daily Budget Reset

```rust
pub struct BudgetTracker {
    daily_budget: f64,
    consumed: f64,
    last_reset: chrono::DateTime<chrono::Utc>,
}

impl BudgetTracker {
    pub fn check_and_reset(&mut self) {
        let now = chrono::Utc::now();
        if now.date_naive() != self.last_reset.date_naive() {
            // New day, reset budget
            self.consumed = 0.0;
            self.last_reset = now;
        }
    }
    
    pub fn can_allocate(&self, epsilon: f64) -> bool {
        self.consumed + epsilon <= self.daily_budget
    }
    
    pub fn consume(&mut self, epsilon: f64) {
        self.consumed += epsilon;
    }
}
```

---

## Testing

### Unit Tests

- `GovernanceDecision` serialization
- `QueryDescriptor` construction from `MeshJobDescriptor`
- Budget tracking (allocation, consumption, reset)

### Integration Tests

- Evaluate mapping job → always allow
- Evaluate analytics query with high cardinality → require DP
- Evaluate export job when budget exceeded → deny
- Budget reset after midnight

---

## References

- **Mesh Contracts**: `lib/domain/contracts/src/mesh.rs`
- **Compliance**: `lib/platform/compliance` (existing)
- **Node Runtime**: `docs/system-design/mesh/node-runtime.md`
- **Hub**: `lib/platform/mesh/hub/README.md` (to be created)
