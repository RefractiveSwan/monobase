use crate::metrics::PipelineMetrics;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Derived ratios and totals for presenting `PipelineMetrics` snapshots.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetricsSnapshot {
    pub metrics: PipelineMetrics,
    pub ratios: MetricsRatios,
}

/// Ratios derived from the raw counters (mapping states, compliance blocks, vector hits).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MetricsRatios {
    pub auto_mapped: Option<f32>,
    pub needs_review: Option<f32>,
    pub no_match: Option<f32>,
    pub license_blocked: Option<f32>,
    pub vector_hit_rate: Option<f32>,
    pub vector_fallback_rate: Option<f32>,
}

impl MetricsSnapshot {
    pub fn new(metrics: PipelineMetrics) -> Self {
        Self {
            ratios: MetricsRatios::from(&metrics),
            metrics,
        }
    }
}

impl From<&PipelineMetrics> for MetricsRatios {
    fn from(metrics: &PipelineMetrics) -> Self {
        let mapping_total = metrics.mapping_count as f32;
        let vector_queries = metrics.vector_queries as f32;

        Self {
            auto_mapped: ratio(metrics.auto_mapped, mapping_total),
            needs_review: ratio(metrics.needs_review, mapping_total),
            no_match: ratio(metrics.no_match, mapping_total),
            license_blocked: ratio(metrics.license_blocked, mapping_total),
            vector_hit_rate: ratio(metrics.vector_hits, vector_queries),
            vector_fallback_rate: ratio(metrics.vector_fallbacks, vector_queries),
        }
    }
}

fn ratio(count: usize, denom: f32) -> Option<f32> {
    if denom <= 0.0 {
        None
    } else {
        Some(count as f32 / denom)
    }
}

/// Produce a structured snapshot (with derived ratios) for API/CLI surfaces.
pub fn metrics_snapshot(metrics: &PipelineMetrics) -> MetricsSnapshot {
    MetricsSnapshot::new(metrics.clone())
}

/// Serialize a snapshot into JSON for `/metrics/summary` or CLI output.
pub fn metrics_snapshot_json(metrics: &PipelineMetrics) -> Value {
    serde_json::to_value(metrics_snapshot(metrics))
        .expect("PipelineMetrics snapshot serialization should not fail")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::PipelineMetrics;

    #[test]
    fn snapshot_reports_ratios() {
        let mut metrics = PipelineMetrics::default();
        metrics.mapping_count = 4;
        metrics.auto_mapped = 2;
        metrics.needs_review = 1;
        metrics.no_match = 1;
        metrics.vector_queries = 2;
        metrics.vector_hits = 1;
        metrics.vector_fallbacks = 1;

        let snapshot = metrics_snapshot(&metrics);
        assert_eq!(snapshot.metrics.mapping_count, 4);
        assert_eq!(snapshot.ratios.auto_mapped, Some(0.5));
        assert_eq!(snapshot.ratios.vector_hit_rate, Some(0.5));
        let json = metrics_snapshot_json(&metrics);
        assert!(json.get("metrics").is_some());
    }
}
