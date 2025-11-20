use super::*;
use crate::clinical::order::{ServiceRequestIntent, ServiceRequestStatus};

#[test]
fn staging_rows_round_trip_with_missing_fields() {
    let flat = StgServiceRequestFlat {
        sr_id: "SR-1".into(),
        patient_id: "PAT-1".into(),
        encounter_id: None,
        status: "active".into(),
        status_enum: ServiceRequestStatus::Active,
        intent: "order".into(),
        intent_enum: ServiceRequestIntent::Order,
        description: "PET".into(),
        ordered_at: None,
    };
    let encoded = serde_json::to_string(&flat).expect("serialize flat");
    let decoded: StgServiceRequestFlat = serde_json::from_str(&encoded).expect("deserialize flat");
    assert_eq!(decoded, flat);

    let exploded = StgSrCodeExploded {
        sr_id: "SR-1".into(),
        system: None,
        code: None,
        display: Some("unknown".into()),
    };
    let encoded = serde_json::to_string(&exploded).expect("serialize exploded");
    let decoded: StgSrCodeExploded = serde_json::from_str(&encoded).expect("deserialize exploded");
    assert_eq!(decoded, exploded);
}

#[test]
fn staging_flat_defaults_enum_fields_when_absent() {
    let legacy = r#"{
        "sr_id": "SR-legacy",
        "patient_id": "PAT-legacy",
        "encounter_id": null,
        "status": "draft",
        "intent": "order",
        "description": "legacy",
        "ordered_at": null
    }"#;
    let decoded: StgServiceRequestFlat =
        serde_json::from_str(legacy).expect("decode legacy flat without enums");
    assert_eq!(decoded.status_enum, ServiceRequestStatus::Draft);
    assert_eq!(decoded.intent_enum, ServiceRequestIntent::Order);
}
