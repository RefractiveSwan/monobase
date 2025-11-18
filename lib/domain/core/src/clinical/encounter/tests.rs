use super::*;
use crate::primitives::value::{EncounterId, PatientId};

#[test]
fn encounter_links_patient() {
    let encounter = Encounter::new(EncounterId::new("ENC-123"), PatientId::new("PAT-123"));
    assert_eq!(encounter.id.0, "ENC-123");
    assert_eq!(encounter.patient_id.0, "PAT-123");
}
