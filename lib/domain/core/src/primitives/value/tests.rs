use super::*;

#[test]
fn patient_id_constructor_wraps_string() {
    let patient = PatientId::new("PAT-1");
    assert_eq!(patient.0, "PAT-1");
}

#[test]
fn encounter_and_service_request_ids_construct() {
    let encounter = EncounterId::new("ENC-9");
    let service_request = ServiceRequestId::new("SR-42");

    assert_eq!(encounter.0, "ENC-9");
    assert_eq!(service_request.0, "SR-42");
}

#[test]
#[should_panic(expected = "PatientId must not be empty")]
fn id_constructors_reject_empty_strings() {
    let _ = PatientId::new("   ");
}
