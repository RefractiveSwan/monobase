//! FHIR ingestion property tests (REFR-05 / FHIR-CONF-015).

use proptest::prelude::*;
use refractive_swan_eval::fake_data::raw_fhir::fake_fhir_bundle_scenario_with_seed;
use refractive_swan_ingestion::{IngestionError, bundle_to_staging};
use refractive_swan_test_suite::regression;

proptest! {
    #[test]
    fn exploded_rows_match_coding_lengths(seed in 0u64..1_000_000) {
        let scenario = fake_fhir_bundle_scenario_with_seed(seed);
        let (flats, exploded) = bundle_to_staging(&scenario.bundle).expect("staging conversion");

        prop_assert_eq!(flats.len(), 1);
        let expected = scenario
            .service_request
            .code
            .as_ref()
            .map(|code| code.coding.len())
            .unwrap_or(0);
        prop_assert_eq!(exploded.len(), expected);
    }
}

#[test]
fn regression_fhir_bundle_ingests() {
    let bundle = regression::baseline_fhir_bundle();
    let (flats, exploded) = bundle_to_staging(&bundle).expect("regression bundle ingestion");
    assert_eq!(flats.len(), 1);
    assert_eq!(exploded.len(), 2);
    assert_eq!(flats[0].sr_id, "SR-000001");
}

#[test]
fn missing_subject_returns_error() {
    let bundle = regression::fhir_bundle_missing_subject();
    let err = bundle_to_staging(&bundle).expect_err("expected missing subject error");
    match err {
        IngestionError::MissingField(field) => assert_eq!(field, "ServiceRequest.subject"),
        other => panic!("unexpected error {other}"),
    }
}

#[test]
fn invalid_status_bubbles_error() {
    let bundle = regression::fhir_bundle_invalid_status();
    let err = bundle_to_staging(&bundle).expect_err("expected invalid status error");
    match err {
        IngestionError::InvalidStatus(value) => assert_eq!(value, "invalid_status"),
        other => panic!("unexpected error {other}"),
    }
}

#[test]
fn extra_codings_are_exploded() {
    let bundle = regression::fhir_bundle_extra_codings();
    let (flats, exploded) = bundle_to_staging(&bundle).expect("extra coding bundle");
    assert_eq!(flats.len(), 1);
    assert_eq!(exploded.len(), 3);
    assert!(exploded.iter().any(|row| {
        row.system
            .as_deref()
            .map(|system| system == "http://example.org/custom")
            .unwrap_or(false)
    }));
}

#[test]
fn uppercase_status_and_intent_are_normalized() {
    let bundle = regression::fhir_bundle_uppercase_status();
    let (flats, exploded) = bundle_to_staging(&bundle).expect("uppercase bundle");
    assert_eq!(flats[0].status, "active");
    assert_eq!(flats[0].intent, "order");
    assert_eq!(exploded.len(), 1);
}
