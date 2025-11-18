use super::*;
use crate::primitives::value::PatientId;

#[test]
fn patient_new_sets_id() {
    let patient = Patient::new(PatientId::new("PAT-123"));
    assert_eq!(patient.id.0, "PAT-123");
}
