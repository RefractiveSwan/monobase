use super::types::Encounter;
use crate::primitives::value::{EncounterId, PatientId};

impl Encounter {
    pub fn new(id: EncounterId, patient_id: PatientId) -> Self {
        Self { id, patient_id }
    }
}
