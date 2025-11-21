//! REFR-14 – Fixture round-trip properties for service request scenarios.
//! Ensures `refractive_swan_test_suite::fixtures` stay deterministic and serialization-safe.

use proptest::prelude::*;
use refractive_swan_test_suite::{assertions, fixtures};

proptest! {
    #[test]
    fn seeded_scenarios_preserve_invariants(seed in any::<u64>()) {
        let scenario = fixtures::service_request_scenario_with_seed(seed);
        assertions::assert_scenario_consistency(&scenario);
        assertions::assert_json_roundtrip(&scenario.service_request);
    }
}
