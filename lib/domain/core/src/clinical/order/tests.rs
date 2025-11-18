use super::*;
use crate::primitives::value::{EncounterId, PatientId, ServiceRequestId};

#[test]
fn service_request_builders_work() {
    let base = ServiceRequest::new(
        ServiceRequestId::new("SR-123"),
        PatientId::new("PAT-1"),
        Some(EncounterId::new("ENC-1")),
        ServiceRequestStatus::Active,
        ServiceRequestIntent::Order,
        "PET/CT staging order",
    );
    assert_eq!(base.description, "PET/CT staging order");

    let active = ServiceRequest::new_active_order(
        ServiceRequestId::new("SR-456"),
        PatientId::new("PAT-2"),
        None,
        "active order",
    );
    assert_eq!(active.intent, ServiceRequestIntent::Order);
    assert_eq!(active.status, ServiceRequestStatus::Active);

    let on_hold = active.with_status(ServiceRequestStatus::OnHold);
    assert_eq!(on_hold.status, ServiceRequestStatus::OnHold);
}
