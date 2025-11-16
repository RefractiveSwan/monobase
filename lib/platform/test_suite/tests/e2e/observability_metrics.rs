use dfps_fake_data::raw_fhir::fake_fhir_bundle_scenario_with_seed;
use dfps_observability::{PipelineMetrics, log_no_match, log_pipeline_output};
use dfps_vector_store::{CapacityProxies, VectorUsageSnapshot};
use dfps_pipeline::bundle_to_mapped_sr;

#[test]
fn metrics_snapshot_matches_expected_counts() {
    let scenario = fake_fhir_bundle_scenario_with_seed(42);
    let mut metrics = PipelineMetrics::default();
    let output = bundle_to_mapped_sr(&scenario.bundle).expect("pipeline run succeeds");
    log_pipeline_output(
        &output.flats,
        &output.exploded_codes,
        &output.mapping_results,
        &mut metrics,
        output.vector_usage,
        None,
    );
    for result in &output.mapping_results {
        if matches!(result.state, dfps_core::mapping::MappingState::NoMatch) {
            log_no_match(result);
        }
    }

    assert_eq!(metrics.bundle_count, 1);
    assert_eq!(metrics.flats_count, output.flats.len());
    assert_eq!(metrics.exploded_count, output.exploded_codes.len());
    assert_eq!(metrics.mapping_count, output.mapping_results.len());

    assert!(
        metrics.auto_mapped + metrics.needs_review + metrics.no_match
            == output.mapping_results.len()
    );
}

#[test]
fn vector_usage_metrics_are_recorded() {
    let mut metrics = PipelineMetrics::default();
    let usage = VectorUsageSnapshot {
        queries: 3,
        hits: 2,
        fallbacks: 1,
        capacity: Some(CapacityProxies {
            geom_rm: Some(0.1),
            geom_dm: Some(3.0),
            geom_rm_sqrt_dm: Some(0.17),
            cap_alpha_sim: Some(0.85),
        }),
    };
    log_pipeline_output(&[], &[], &[], &mut metrics, Some(usage), Some(120));
    assert_eq!(metrics.vector_queries, 3);
    assert_eq!(metrics.vector_hits, 2);
    assert_eq!(metrics.vector_fallbacks, 1);
    assert_eq!(metrics.vector_latency_ms_p95, Some(120));
    assert_eq!(metrics.vector_capacity_cap_alpha_sim, Some(0.85));
    assert_eq!(metrics.vector_capacity_geom_rm_sqrt_dm, Some(0.17));
}
