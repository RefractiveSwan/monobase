use serde::{Deserialize, Serialize};

use crate::clinical::order::{ServiceRequestIntent, ServiceRequestStatus};
use crate::primitives::value::{EncounterId, PatientId, ServiceRequestId};

#[cfg(feature = "dummy")]
use fake::Dummy;

/// Core "order" aggregate in refractive_swan, similar to a FHIR ServiceRequest.
#[cfg_attr(feature = "dummy", derive(Dummy))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceRequest {
    pub id: ServiceRequestId,
    pub patient_id: PatientId,
    pub encounter_id: Option<EncounterId>,

    pub status: ServiceRequestStatus,
    pub intent: ServiceRequestIntent,

    /// A human-readable label or code display.
    pub description: String,
    // Future: coding(s), categories, reason codes, supportingInfo, etc.
}
