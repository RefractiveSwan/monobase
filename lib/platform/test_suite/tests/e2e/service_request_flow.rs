//! Service request flow invariants (REFR-14).

use refractive_swan_test_suite::{assertions, fixtures};

#[test]
fn fake_data_roundtrip_and_invariants() {
    let scenario = fixtures::service_request_scenario();
    assertions::assert_scenario_consistency(&scenario);
    assertions::assert_json_roundtrip(&scenario.service_request);
}

#[test]
fn regression_fixture_deserializes() {
    let fixture = refractive_swan_test_suite::regression::baseline_service_request();
    assertions::assert_service_request_integrity(&fixture);
    assertions::assert_json_roundtrip(&fixture);
}
