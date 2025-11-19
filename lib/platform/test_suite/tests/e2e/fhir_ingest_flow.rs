//! FHIR ingestion → staging → domain flow (REFR-05).

use dfps_eval::fake_data::raw_fhir::fake_fhir_bundle_scenario;
use dfps_ingestion::{bundle_to_domain, bundle_to_staging};

#[test]
fn bundle_to_staging_counts_match_codings() {
    let scenario = fake_fhir_bundle_scenario();
    let (flats, exploded) = bundle_to_staging(&scenario.bundle).expect("staging conversion");

    assert_eq!(flats.len(), 1);
    let flat = &flats[0];
    let sr_id = scenario
        .service_request
        .id
        .as_ref()
        .expect("fake ServiceRequest has id");
    assert_eq!(&flat.sr_id, sr_id);

    let expected_coding_len = scenario
        .service_request
        .code
        .as_ref()
        .map(|code| code.coding.len())
        .unwrap_or(0);
    assert_eq!(exploded.len(), expected_coding_len);
    assert!(exploded.iter().all(|row| row.sr_id == *sr_id));
}

#[test]
fn bundle_to_domain_normalizes_ids_and_status() {
    let scenario = fake_fhir_bundle_scenario();
    let domain = bundle_to_domain(&scenario.bundle).expect("domain conversion succeeds");
    assert_eq!(domain.len(), 1);
    let order = &domain[0];

    let sr_id = scenario
        .service_request
        .id
        .as_ref()
        .expect("fake ServiceRequest has id");
    assert_eq!(&order.id.0, sr_id);

    let patient_id = scenario.patient.id.as_ref().expect("fake patient has id");
    assert_eq!(&order.patient_id.0, patient_id);

    let encounter_id = scenario
        .encounter
        .id
        .as_ref()
        .expect("fake encounter has id");
    assert_eq!(
        order.encounter_id.as_ref().map(|id| &id.0),
        Some(encounter_id)
    );

    assert_eq!(order.description, "PET/CT order from fake data");
}
