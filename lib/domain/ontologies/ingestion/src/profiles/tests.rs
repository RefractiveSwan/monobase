#![cfg(feature = "profile_validation")]
use super::*;

#[test]
fn loads_known_profiles() {
    assert!(load_profile(PATIENT_PROFILE_URL).is_some());
    assert!(load_profile(ENCOUNTER_PROFILE_URL).is_some());
    assert!(load_profile(SERVICE_REQUEST_PROFILE_URL).is_some());
    assert!(load_profile("Unknown").is_none());
}

#[test]
fn parses_snapshot_elements() {
    let profile = load_profile(SERVICE_REQUEST_PROFILE_URL).expect("profile");
    let subject = profile.element_by_path("ServiceRequest.subject").unwrap();
    assert_eq!(subject.min, Some(1));
    assert!(subject.must_support);

    let intent = profile.element_by_path("ServiceRequest.intent").unwrap();
    assert_eq!(intent.min, Some(1));
    assert_eq!(intent.path, "ServiceRequest.intent");
    assert!(intent.binding.is_some());
}

#[test]
fn profile_meta_preserves_urls() {
    let profile = load_profile(PATIENT_PROFILE_URL).unwrap();
    assert_eq!(profile.meta.url, PATIENT_PROFILE_URL.to_string());
    assert_eq!(profile.meta.resource_type, "Patient".to_string());
}
