use super::types::Patient;
use crate::primitives::value::PatientId;

impl Patient {
    pub fn new(id: PatientId) -> Self {
        Self { id }
    }
}
