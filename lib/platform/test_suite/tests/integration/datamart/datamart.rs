//! Datamart adapter smoke tests (REFR-16).

use refractive_swan_datamart::from_pipeline_output;
use refractive_swan_pipeline::bundle_to_mapped_sr;

#[test]
fn baseline_bundle_maps_into_datamart() {
    let bundle = refractive_swan_test_suite::regression::baseline_fhir_bundle();
    let output = bundle_to_mapped_sr(&bundle).expect("pipeline output");
    let (dims, facts) = from_pipeline_output(&output);

    assert_eq!(dims.patients.len(), 1);
    assert_eq!(dims.codes.len(), output.exploded_codes.len());
    assert_eq!(facts.len(), output.mapping_results.len());

    for fact in &facts {
        assert!(dims.patients.iter().any(|dim| dim.key == fact.patient_key));
        if let Some(encounter_key) = fact.encounter_key {
            assert!(dims.encounters.iter().any(|dim| dim.key == encounter_key));
        }
        assert!(dims.codes.iter().any(|dim| dim.key == fact.code_key));
    }
}

#[test]
fn unknown_code_creates_no_match_dim() {
    let bundle = refractive_swan_test_suite::regression::fhir_bundle_unknown_code();
    let output = bundle_to_mapped_sr(&bundle).expect("pipeline output");
    let (dims, facts) = from_pipeline_output(&output);
    assert_eq!(facts.len(), 1);
    let fact = &facts[0];
    let no_match_key = fact.ncit_key.expect("no-match key expected");
    let dim = dims
        .ncit
        .iter()
        .find(|dim| dim.key == no_match_key)
        .expect("no-match dim present");
    assert_eq!(dim.ncit_id, "NO_MATCH");
}
