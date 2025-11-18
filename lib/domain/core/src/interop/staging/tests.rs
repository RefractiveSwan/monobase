use super::*;

#[test]
fn staging_rows_round_trip_with_missing_fields() {
    let flat = StgServiceRequestFlat {
        sr_id: "SR-1".into(),
        patient_id: "PAT-1".into(),
        encounter_id: None,
        status: "active".into(),
        intent: "order".into(),
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
