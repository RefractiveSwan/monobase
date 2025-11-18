use super::{ServiceRequest, ServiceRequestIntent, ServiceRequestStatus};
use crate::primitives::value::{EncounterId, PatientId, ServiceRequestId};

impl ServiceRequest {
    /// Simple constructor that enforces some core invariants.
    ///
    /// E.g. you might later restrict which status/intent combos are valid.
    ///
    /// # Examples
    ///
    /// Construct a ServiceRequest from fake IDs, mirroring the diagrams in
    /// `docs/system-design/fhir/models/data-model-er.md`.
    ///
    /// ```
    /// use dfps_core::order::{ServiceRequest, ServiceRequestStatus, ServiceRequestIntent};
    /// use dfps_core::value::{PatientId, EncounterId, ServiceRequestId};
    ///
    /// let sr = ServiceRequest::new(
    ///     ServiceRequestId::new("SR-123"),
    ///     PatientId::new("PAT-1"),
    ///     Some(EncounterId::new("ENC-1")),
    ///     ServiceRequestStatus::Active,
    ///     ServiceRequestIntent::Order,
    ///     "PET/CT staging order",
    /// );
    /// assert_eq!(sr.status, ServiceRequestStatus::Active);
    /// ```
    pub fn new(
        id: ServiceRequestId,
        patient_id: PatientId,
        encounter_id: Option<EncounterId>,
        status: ServiceRequestStatus,
        intent: ServiceRequestIntent,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id,
            patient_id,
            encounter_id,
            status,
            intent,
            description: description.into(),
        }
    }

    /// Convenience constructor for "active order" (most common case).
    pub fn new_active_order(
        id: ServiceRequestId,
        patient_id: PatientId,
        encounter_id: Option<EncounterId>,
        description: impl Into<String>,
    ) -> Self {
        Self::new(
            id,
            patient_id,
            encounter_id,
            ServiceRequestStatus::Active,
            ServiceRequestIntent::Order,
            description,
        )
    }

    /// Simple status transition helper.
    pub fn with_status(mut self, status: ServiceRequestStatus) -> Self {
        self.status = status;
        self
    }
}
