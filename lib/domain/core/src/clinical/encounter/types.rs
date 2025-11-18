use serde::{Deserialize, Serialize};

use crate::primitives::value::{EncounterId, PatientId};

#[cfg(feature = "dummy")]
use fake::Dummy;

/// Minimal encounter entity.
/// Links a patient to a point-in-time encounter/context.
#[cfg_attr(feature = "dummy", derive(Dummy))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Encounter {
    pub id: EncounterId,
    pub patient_id: PatientId,
    // Future: class, type, period, location, etc.
}
