//! Regression fixture integrity tests (REFR-14).

use dfps_test_suite::{assertions, regression};

#[test]
fn baseline_fixture_matches_schema() {
    let fixture = regression::baseline_service_request();
    assertions::assert_service_request_integrity(&fixture);
}
