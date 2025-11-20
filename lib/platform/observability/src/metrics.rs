use dfps_core::{
    mapping::{MappingResult, MappingState},
    staging::{StgServiceRequestFlat, StgSrCodeExploded},
};
use dfps_vector_port::VectorUsageSnapshot;
use serde::{Deserialize, Serialize};

/// Workspace-wide pipeline counters shared across CLI/API surfaces.
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct PipelineMetrics {
    pub bundle_count: usize,
    pub flats_count: usize,
    pub exploded_count: usize,
    pub mapping_count: usize,
    pub auto_mapped: usize,
    pub needs_review: usize,
    pub no_match: usize,
    /// Compliance mode recorded at the app/CLI layer; domain crates leave this unset.
    #[serde(default)]
    pub compliance_mode: Option<String>,
    /// Number of mappings blocked by policy (set via `dfps_compliance` in pipeline/app layers).
    #[serde(default)]
    pub license_blocked: usize,
    #[serde(default)]
    pub vector_queries: usize,
    #[serde(default)]
    pub vector_hits: usize,
    #[serde(default)]
    pub vector_fallbacks: usize,
    #[serde(default)]
    pub vector_latency_ms_p95: Option<u64>,
    #[serde(default)]
    pub vector_capacity_geom_rm: Option<f32>,
    #[serde(default)]
    pub vector_capacity_geom_dm: Option<f32>,
    #[serde(default)]
    pub vector_capacity_geom_rm_sqrt_dm: Option<f32>,
    #[serde(default)]
    pub vector_capacity_cap_alpha_sim: Option<f32>,
    /// Number of `/analytics/*` requests served by API/frontend surfaces.
    #[serde(default)]
    pub analytics_requests: usize,
    /// Number of cohort queries issued via HTTP surfaces; not touched by domain crates.
    #[serde(default)]
    pub cohort_queries: usize,
    /// Total cohort rows emitted for `/analytics/cohort`; API/frontends own this counter.
    #[serde(default)]
    pub cohort_results_total: usize,
    #[serde(default)]
    pub avg_cohort_size: Option<f32>,
}

impl PipelineMetrics {
    pub fn record(
        &mut self,
        flats: &[StgServiceRequestFlat],
        codes: &[StgSrCodeExploded],
        mappings: &[MappingResult],
    ) {
        self.bundle_count += 1;
        self.flats_count += flats.len();
        self.exploded_count += codes.len();
        self.mapping_count += mappings.len();
        for result in mappings {
            match result.state {
                MappingState::AutoMapped => self.auto_mapped += 1,
                MappingState::NeedsReview => self.needs_review += 1,
                MappingState::NoMatch => self.no_match += 1,
            }
            if result.reason.as_deref() == Some("license_blocked") {
                self.license_blocked += 1;
            }
        }
    }

    pub fn record_vector_usage(&mut self, usage: VectorUsageSnapshot, latency_ms_p95: Option<u64>) {
        self.vector_queries += usage.queries;
        self.vector_hits += usage.hits;
        self.vector_fallbacks += usage.fallbacks;
        if let Some(p95) = latency_ms_p95 {
            self.vector_latency_ms_p95 = Some(p95);
        }
        if let Some(capacity) = usage.capacity {
            self.vector_capacity_geom_rm = capacity.geom_rm;
            self.vector_capacity_geom_dm = capacity.geom_dm;
            self.vector_capacity_geom_rm_sqrt_dm = capacity.geom_rm_sqrt_dm;
            self.vector_capacity_cap_alpha_sim = capacity.cap_alpha_sim;
        }
    }
}

/// Merge a vector usage snapshot (queries/hits/capacity) into the metrics summary.
pub fn apply_vector_usage(
    metrics: &mut PipelineMetrics,
    usage: VectorUsageSnapshot,
    vector_latency_ms_p95: Option<u64>,
) {
    metrics.record_vector_usage(usage, vector_latency_ms_p95);
}
