#[cfg(test)]
use crate::env::reset_env_state_for_tests;
use crate::{
    env::ensure_env,
    metrics::{PipelineMetrics, apply_vector_usage},
};
use log::{info, warn};
#[cfg(test)]
use refractive_swan_core::mapping::MappingState;
use refractive_swan_core::{
    mapping::MappingResult,
    staging::{StgServiceRequestFlat, StgSrCodeExploded},
};
#[cfg(test)]
use refractive_swan_vector_port::VectorCapacitySnapshot;
use refractive_swan_vector_port::VectorUsageSnapshot;

pub fn log_pipeline_output(
    flats: &[StgServiceRequestFlat],
    codes: &[StgSrCodeExploded],
    mappings: &[MappingResult],
    metrics: &mut PipelineMetrics,
    vector_usage: Option<VectorUsageSnapshot>,
    vector_latency_ms_p95: Option<u64>,
) {
    if let Err(err) = ensure_env() {
        warn!(
            target: "refractive_swan_observability",
            "observability env not loaded: {err}"
        );
    }
    metrics.record(flats, codes, mappings);
    if let Some(usage) = vector_usage {
        apply_vector_usage(metrics, usage, vector_latency_ms_p95);
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
        target: "refractive_swan_pipeline",
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

/// Log a pipeline run when metrics have already been computed upstream.
pub fn log_pipeline_output_with_summary(
    flats: &[StgServiceRequestFlat],
    _codes: &[StgSrCodeExploded],
    mappings: &[MappingResult],
    run_metrics: &PipelineMetrics,
    metrics: &mut PipelineMetrics,
) {
    if let Err(err) = ensure_env() {
        warn!(
            target: "refractive_swan_observability",
            "observability env not loaded: {err}"
        );
    }
    metrics.merge(run_metrics);
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
        target: "refractive_swan_pipeline",
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
    if let Err(err) = ensure_env() {
        warn!(
            target: "refractive_swan_observability",
            "observability env not loaded: {err}"
        );
    }
    warn!(
        target: "refractive_swan_mapping",
        "no_match code={} reason={}",
        result.code_element_id,
        result
            .reason
            .as_deref()
            .unwrap_or("unknown_reason")
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::PipelineMetrics;
    use refractive_swan_core::{
        clinical::order::{ServiceRequestIntent, ServiceRequestStatus},
        mapping::{MappingSourceVersion, MappingStrategy, MappingThresholds},
        staging::{StgServiceRequestFlat, StgSrCodeExploded},
    };
    use std::{
        env,
        sync::{Mutex, OnceLock},
    };

    static ENV_GUARD: OnceLock<Mutex<()>> = OnceLock::new();

    fn env_guard() -> &'static Mutex<()> {
        ENV_GUARD.get_or_init(|| Mutex::new(()))
    }

    fn sample_flat() -> StgServiceRequestFlat {
        StgServiceRequestFlat {
            sr_id: "sr-1".into(),
            patient_id: "pat-1".into(),
            encounter_id: None,
            status: "active".into(),
            status_enum: ServiceRequestStatus::Active,
            intent: "plan".into(),
            intent_enum: ServiceRequestIntent::Plan,
            description: "sample".into(),
            ordered_at: None,
        }
    }

    fn sample_code() -> StgSrCodeExploded {
        StgSrCodeExploded {
            sr_id: "sr-1".into(),
            system: Some("http://loinc.org".into()),
            code: Some("1234-5".into()),
            display: Some("Sample".into()),
        }
    }

    fn sample_mapping(state: MappingState, reason: Option<&str>) -> MappingResult {
        let thresholds = MappingThresholds::default();
        let version = MappingSourceVersion::new("ncit_v1", "umls_v1");
        let reason = reason.map(|value| value.to_string());
        match state {
            MappingState::AutoMapped => MappingResult::auto_mapped(
                "code-1",
                "C0001",
                0.99,
                thresholds,
                version.clone(),
                MappingStrategy::Lexical,
                reason,
                None,
                None,
                None,
            ),
            MappingState::NeedsReview => MappingResult::needs_review(
                "code-2",
                "C0002",
                0.7,
                thresholds,
                version.clone(),
                MappingStrategy::Lexical,
                reason,
                None,
                None,
                None,
            ),
            MappingState::NoMatch => MappingResult::no_match(
                "code-3",
                0.0,
                thresholds,
                version,
                MappingStrategy::Unmapped,
                reason,
                None,
                None,
            ),
        }
    }

    #[test]
    fn log_helpers_continue_when_env_missing() {
        let _lock = env_guard().lock().unwrap();
        reset_env_state_for_tests();
        unsafe {
            env::set_var("refractive_swan_ENV_FILE", "missing.observability.env");
        }
        let flat = sample_flat();
        let code = sample_code();
        let auto = sample_mapping(MappingState::AutoMapped, None);
        let mut metrics = PipelineMetrics::default();
        log_pipeline_output(&[flat], &[code], &[auto], &mut metrics, None, None);
        assert_eq!(metrics.bundle_count, 1);
        assert_eq!(metrics.mapping_count, 1);
        let no_match = sample_mapping(MappingState::NoMatch, Some("no_match"));
        log_no_match(&no_match);
        unsafe {
            env::remove_var("refractive_swan_ENV_FILE");
        }
    }

    #[test]
    fn apply_vector_usage_sets_capacity_fields() {
        let _lock = env_guard().lock().unwrap();
        reset_env_state_for_tests();
        let mut metrics = PipelineMetrics::default();
        let usage = VectorUsageSnapshot {
            queries: 5,
            hits: 3,
            fallbacks: 2,
            capacity: Some(VectorCapacitySnapshot {
                geom_rm: Some(0.1),
                geom_dm: Some(3.0),
                geom_rm_sqrt_dm: Some(0.18),
                cap_alpha_sim: Some(0.82),
            }),
        };
        apply_vector_usage(&mut metrics, usage, Some(150));
        assert_eq!(metrics.vector_queries, 5);
        assert_eq!(metrics.vector_hits, 3);
        assert_eq!(metrics.vector_latency_ms_p95, Some(150));
        assert_eq!(metrics.vector_capacity_cap_alpha_sim, Some(0.82));
    }

    #[test]
    fn log_with_summary_merges_metrics() {
        let _lock = env_guard().lock().unwrap();
        reset_env_state_for_tests();
        let mut aggregate = PipelineMetrics::default();
        let flat = sample_flat();
        let code = sample_code();
        let auto = sample_mapping(MappingState::AutoMapped, None);
        let mut run = PipelineMetrics::default();
        run.record(&[flat.clone()], &[code.clone()], &[auto.clone()]);
        run.vector_queries = 2;
        run.vector_hits = 1;
        log_pipeline_output_with_summary(&[flat], &[code], &[auto], &run, &mut aggregate);
        assert_eq!(aggregate.bundle_count, 1);
        assert_eq!(aggregate.auto_mapped, 1);
        assert_eq!(aggregate.vector_queries, 2);
    }
}
