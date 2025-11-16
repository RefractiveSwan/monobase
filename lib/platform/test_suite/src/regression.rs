use dfps_core::{fhir, order::ServiceRequest};
use dfps_fake_data::fixtures::{self, Registry};
use once_cell::sync::Lazy;

static REGRESSION_ENV: Lazy<()> = Lazy::new(|| {
    crate::init_environment();
});

static FIXTURE_REGISTRY: Lazy<Registry> = Lazy::new(|| Registry::default());

fn ensure_env_loaded() {
    Lazy::force(&REGRESSION_ENV);
}

fn registry() -> &'static Registry {
    ensure_env_loaded();
    Lazy::force(&FIXTURE_REGISTRY)
}

pub fn baseline_service_request() -> ServiceRequest {
    let file = registry()
        .open_regression("service_request_active")
        .expect("regression service request fixture should exist");
    serde_json::from_reader(file).expect("regression service request fixture should be valid JSON")
}

pub fn baseline_fhir_bundle() -> fhir::Bundle {
    fixtures::bundles::load(registry(), "fhir_bundle_sr")
        .expect("regression FHIR bundle fixture should be valid JSON")
}

pub fn fhir_bundle_missing_subject() -> fhir::Bundle {
    fixtures::bundles::load(registry(), "fhir_bundle_missing_subject")
        .expect("missing-subject bundle should be valid JSON")
}

pub fn fhir_bundle_invalid_status() -> fhir::Bundle {
    fixtures::bundles::load(registry(), "fhir_bundle_invalid_status")
        .expect("invalid-status bundle should be valid JSON")
}

pub fn fhir_bundle_extra_codings() -> fhir::Bundle {
    fixtures::bundles::load(registry(), "fhir_bundle_extra_codings")
        .expect("extra-codings bundle should be valid JSON")
}

pub fn fhir_bundle_uppercase_status() -> fhir::Bundle {
    fixtures::bundles::load(registry(), "fhir_bundle_uppercase_status")
        .expect("uppercase-status bundle should be valid JSON")
}

pub fn fhir_bundle_unknown_code() -> fhir::Bundle {
    fixtures::bundles::load(registry(), "fhir_bundle_unknown_code")
        .expect("unknown-code bundle should be valid JSON")
}

pub fn fhir_bundle_missing_encounter() -> fhir::Bundle {
    fixtures::bundles::load(registry(), "fhir_bundle_missing_encounter")
        .expect("missing-encounter bundle should be valid JSON")
}

pub fn fhir_bundle_profile_missing_intent() -> fhir::Bundle {
    fixtures::bundles::load(registry(), "fhir_bundle_profile_missing_intent")
        .expect("profile-violating bundle should be valid JSON")
}
