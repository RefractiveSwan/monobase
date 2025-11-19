use serde::{Deserialize, Serialize};

/// Snapshot of vector-store capacity metrics (geometry/manifold approximations).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VectorCapacitySnapshot {
    pub geom_rm: Option<f32>,
    pub geom_dm: Option<f32>,
    pub geom_rm_sqrt_dm: Option<f32>,
    pub cap_alpha_sim: Option<f32>,
}

/// Aggregated usage counters emitted by vector-store implementations.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VectorUsageSnapshot {
    pub queries: usize,
    pub hits: usize,
    pub fallbacks: usize,
    #[serde(default)]
    pub capacity: Option<VectorCapacitySnapshot>,
}
