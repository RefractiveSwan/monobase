//! Full ingestion → mapping end-to-end flow (REFR-06 / REFR-16).

use dfps_pipeline::bundle_to_mapped_sr;
use dfps_test_suite::regression;

#[test]
fn bundle_maps_to_ncit_concepts_end_to_end() {
    let bundle = regression::baseline_fhir_bundle();
    let output = bundle_to_mapped_sr(&bundle).expect("pipeline output");

    assert_eq!(output.flats.len(), 1);
    assert_eq!(output.exploded_codes.len(), 2);
    assert_eq!(output.mapping_results.len(), 2);
    assert!(!output.dim_concepts.is_empty());

    let ncit_ids: Vec<_> = output
        .mapping_results
        .iter()
        .filter_map(|result| result.ncit_id.as_deref())
        .collect();
    assert!(ncit_ids.iter().any(|id| *id == "NCIT:C19951"));
}
