use serde::{Deserialize, Serialize};

use crate::primitives::value::PatientId;

#[cfg(feature = "dummy")]
use fake::Dummy;

/// Minimal patient entity.
///
/// Intentionally small for now; expand with demographics as needed.
#[cfg_attr(feature = "dummy", derive(Dummy))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Patient {
    pub id: PatientId,
    // Future: MRN, name, birth date, etc.
}
