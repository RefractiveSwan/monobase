use dfps_core::order::{ServiceRequest, ServiceRequestIntent, ServiceRequestStatus};
use dfps_eval::fake_data::scenarios::ServiceRequestScenario;
use serde::{Serialize, de::DeserializeOwned};

pub fn assert_json_roundtrip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let json = serde_json::to_string_pretty(value).expect("serialize fixture");
    let decoded: T = serde_json::from_str(&json).expect("deserialize fixture");
    assert_eq!(*value, decoded);
}

pub fn assert_service_request_integrity(service_request: &ServiceRequest) {
    assert!(
        !service_request.id.0.is_empty(),
        "service request id should not be empty"
    );
    assert!(
        !service_request.patient_id.0.is_empty(),
        "patient id should not be empty"
    );
    if let Some(encounter_id) = &service_request.encounter_id {
        assert!(
            !encounter_id.0.is_empty(),
            "encounter id should not be empty when present"
        );
    }
    assert!(
        !service_request.description.is_empty(),
        "description should be present"
    );
    assert_valid_status_intent_combo(service_request);
}

fn assert_valid_status_intent_combo(service_request: &ServiceRequest) {
    match service_request.status {
        ServiceRequestStatus::Draft => assert!(
            matches!(
                service_request.intent,
                ServiceRequestIntent::Plan | ServiceRequestIntent::Proposal
            ),
            "draft orders must be plans or proposals"
        ),
        ServiceRequestStatus::Completed | ServiceRequestStatus::Cancelled => assert!(
            matches!(service_request.intent, ServiceRequestIntent::Order),
            "completed/cancelled orders remain official orders"
        ),
        _ => {}
    }
}

pub fn assert_scenario_consistency(scenario: &ServiceRequestScenario) {
    assert_eq!(
        scenario.encounter.patient_id, scenario.patient.id,
        "encounter patient reference must match patient entity"
    );

    let sr = &scenario.service_request;
    assert_eq!(
        sr.patient_id, scenario.patient.id,
        "service request patient reference must match patient entity"
    );

    let encounter_id = sr
        .encounter_id
        .as_ref()
        .expect("service request scenario should include encounter id");

    assert_eq!(
        encounter_id, &scenario.encounter.id,
        "service request encounter reference must match encounter entity"
    );

    assert_service_request_integrity(sr);
}
