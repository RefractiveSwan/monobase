//! Workspace-wide observability helpers for logging and metrics snapshots.
//!
//! Hooks into the Bundle -> NCIt pipeline so CLIs can emit structured log
//! events and tests can validate mapping state distributions (OBS-01 / OBS-02).

use dfps_core::{
    mapping::{MappingResult, MappingState},
    staging::{StgServiceRequestFlat, StgSrCodeExploded},
};
use log::{info, warn};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

pub mod vector_usage;

pub use vector_usage::{VectorCapacitySnapshot, VectorUsageSnapshot};

static OBS_ENV: Lazy<()> = Lazy::new(|| {
    dfps_configuration::load_env("platform.observability")
        .unwrap_or_else(|err| panic!("dfps_observability env error: {err}"));
});

fn ensure_env() {
    Lazy::force(&OBS_ENV);
}

/// Allow callers to eagerly load environment files.
pub fn init_environment() {
    ensure_env();
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct PipelineMetrics {
    pub bundle_count: usize,
    pub flats_count: usize,
    pub exploded_count: usize,
    pub mapping_count: usize,
    pub auto_mapped: usize,
    pub needs_review: usize,
    pub no_match: usize,
    #[serde(default)]
    pub compliance_mode: Option<String>,
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
    #[serde(default)]
    pub analytics_requests: usize,
    #[serde(default)]
    pub cohort_queries: usize,
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

pub fn log_pipeline_output(
    flats: &[StgServiceRequestFlat],
    codes: &[StgSrCodeExploded],
    mappings: &[MappingResult],
    metrics: &mut PipelineMetrics,
    vector_usage: Option<VectorUsageSnapshot>,
    vector_latency_ms_p95: Option<u64>,
) {
    ensure_env();
    metrics.record(flats, codes, mappings);
    if let Some(usage) = vector_usage {
        metrics.record_vector_usage(usage, vector_latency_ms_p95);
    }
    let capacity_note = metrics
        .vector_capacity_geom_rm_sqrt_dm
        .map(|value| {
            format!(
                "vector_geom_rm_sqrt_dm={value} cap_alpha_sim={:?}",
                metrics.vector_capacity_cap_alpha_sim
            )
        })
        .unwrap_or_else(|| "vector_capacity=None".to_string());
    info!(
        target: "dfps_pipeline",
        "bundle processed; flats={}, mappings={}, automap={}, review={}, nomatch={}, license_blocked={}, vector_queries={}, vector_fallbacks={}, vector_latency_ms_p95={:?}, {capacity_note}",
        flats.len(),
        mappings.len(),
        metrics.auto_mapped,
        metrics.needs_review,
        metrics.no_match,
        metrics.license_blocked,
        metrics.vector_queries,
        metrics.vector_fallbacks,
        metrics.vector_latency_ms_p95,
    );
}

pub fn log_no_match(result: &MappingResult) {
    ensure_env();
    warn!(
        target: "dfps_mapping",
        "no_match code={} reason={}",
        result.code_element_id,
        result
            .reason
            .as_deref()
            .unwrap_or("unknown_reason")
    );
}
