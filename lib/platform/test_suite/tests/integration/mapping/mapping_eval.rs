//! End-to-end mapping eval harness tests (EVAL-012).

use refractive_swan_core::mapping::MappingState;
use refractive_swan_eval::{self, EvalCase};
use refractive_swan_mapping::map_staging_codes;
use refractive_swan_test_suite::{fixtures, init_environment};
use serde_json::to_vec;

fn custom_no_match_case() -> EvalCase {
    EvalCase {
        system: "http://example.org/custom-system".into(),
        code: "CUSTOM-999".into(),
        display: "Custom imaging code".into(),
        expected_ncit_id: "NCIT:UNMAPPED".into(),
    }
}

#[test]
fn pet_ct_eval_sample_has_high_precision() {
    init_environment().expect("load test env");
    let cases = fixtures::eval_pet_ct_small_cases();
    let summary = eval_with_pipeline(&cases);

    assert_eq!(summary.total_cases, cases.len());
    assert!(summary.precision >= 0.95);
    assert!(summary.recall >= 0.95);
    assert!(summary.f1 >= 0.95);
    assert_eq!(
        summary.state_counts.get("auto_mapped"),
        Some(&(cases.len()))
    );
    let system_metrics = summary
        .by_system
        .get("http://www.ama-assn.org/go/cpt")
        .expect("system metrics for CPT");
    assert_eq!(system_metrics.total_cases, 1);
    assert!(system_metrics.precision >= 0.95);
    let license_metrics = summary
        .by_license_tier
        .get("licensed")
        .expect("licensed tier metrics");
    assert!(license_metrics.precision >= 0.95);
}

#[test]
fn eval_summary_flags_no_match_cases() {
    init_environment().expect("load test env");
    let mut cases = fixtures::eval_pet_ct_small_cases();
    cases.push(custom_no_match_case());

    let summary = eval_with_pipeline(&cases);
    let auto_mapped = summary
        .state_counts
        .get("auto_mapped")
        .copied()
        .unwrap_or(0);
    assert_eq!(auto_mapped, cases.len() - 1);
    assert_eq!(
        summary.state_counts.get("no_match"),
        Some(&1),
        "expected exactly one no_match case"
    );

    let custom_result = summary
        .results
        .iter()
        .find(|res| res.case.code == "CUSTOM-999")
        .expect("custom case should be present");
    assert_eq!(custom_result.mapping.state, MappingState::NoMatch);
    assert!(
        custom_result.mapping.ncit_id.is_none(),
        "custom no-match case should not have an NCIt ID"
    );
}

#[test]
fn tiered_datasets_load() {
    init_environment().expect("load test env");
    for dataset in [
        "bronze_pet_ct_small",
        "bronze_pet_ct_unknowns",
        "silver_pet_ct_small",
        "gold_pet_ct_small",
    ] {
        let cases =
            refractive_swan_eval::load_dataset(dataset).unwrap_or_else(|_| panic!("{dataset} should load"));
        assert!(!cases.is_empty(), "{dataset} should contain rows");
    }
}

fn eval_with_pipeline(cases: &[EvalCase]) -> refractive_swan_eval::EvalSummary {
    refractive_swan_eval::run_eval_with_mapper(cases, |rows| map_staging_codes(rows).0)
}

#[test]
fn eval_summary_is_deterministic() {
    init_environment().expect("load test env");
    let cases = fixtures::eval_pet_ct_small_cases();

    let first = eval_with_pipeline(&cases);
    let second = eval_with_pipeline(&cases);

    // Byte-for-byte stable fingerprints.
    let fp1 = refractive_swan_eval::fingerprint_summary(&first);
    let fp2 = refractive_swan_eval::fingerprint_summary(&second);
    assert_eq!(fp1, fp2, "fingerprints should match across runs");

    let serialized1 = to_vec(&first).expect("serialize summary");
    let serialized2 = to_vec(&second).expect("serialize summary");
    assert_eq!(
        serialized1, serialized2,
        "serialized summaries should match"
    );
}

#[test]
fn run_eval_outputs_ndjson_stable() {
    init_environment().expect("load test env");
    let cases = fixtures::eval_pet_ct_small_cases();

    let capture = |cases: &[EvalCase]| {
        let summary = eval_with_pipeline(cases);
        let fingerprint = refractive_swan_eval::fingerprint_summary(&summary);
        let mut buffer = Vec::new();
        for result in &summary.results {
            serde_json::to_writer(&mut buffer, result).expect("serialize eval result");
            buffer.push(b'\n');
        }
        (fingerprint, buffer)
    };

    let (fp1, bytes1) = capture(&cases);
    let (fp2, bytes2) = capture(&cases);

    assert_eq!(fp1, fp2, "summary fingerprint should be stable");
    assert_eq!(
        bytes1, bytes2,
        "eval_results NDJSON should be byte-for-byte stable"
    );
}
