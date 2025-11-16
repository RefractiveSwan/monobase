use dfps_ingestion::validation::validate_bundle;
use dfps_test_suite::regression;

#[test]
fn baseline_bundle_has_no_validation_issues() {
    let bundle = regression::baseline_fhir_bundle();
    let report = validate_bundle(&bundle);
    assert!(!report.has_errors());
    assert!(report.issues.is_empty());
}

#[test]
fn missing_subject_bundle_reports_error() {
    let bundle = regression::fhir_bundle_missing_subject();
    let report = validate_bundle(&bundle);
    assert!(report.has_errors());
    assert!(
        report
            .issues
            .iter()
            .any(|issue| issue.id == "VAL_SR_SUBJECT_MISSING")
    );
}

#[test]
fn invalid_status_bundle_reports_error() {
    let bundle = regression::fhir_bundle_invalid_status();
    let report = validate_bundle(&bundle);
    assert!(report.has_errors());
    assert!(
        report
            .issues
            .iter()
            .any(|issue| issue.id == "VAL_SR_STATUS_INVALID")
    );
}

#[test]
fn missing_encounter_bundle_reports_warning() {
    let bundle = regression::fhir_bundle_missing_encounter();
    let report = validate_bundle(&bundle);
    assert!(!report.has_errors());
    assert!(
        report
            .issues
            .iter()
            .any(|issue| issue.id == "VAL_SR_ENCOUNTER_NOT_FOUND")
    );
}

#[test]
fn profile_violation_bundle_reports_profile_issue() {
    let bundle = regression::fhir_bundle_profile_missing_intent();
    let report = validate_bundle(&bundle);
    assert!(report.has_errors());
    assert!(
        report
            .issues
            .iter()
            .any(|issue| issue.id == "VAL_SR_PROFILE_INTENT_MISSING")
    );
}
