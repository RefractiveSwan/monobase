use crate::value::{
    fake_order_description_with_rng, fake_service_request_id_with_rng,
    fake_service_request_intent_with_rng, fake_service_request_status_with_rng,
};
use dfps_core::{
    order::{ServiceRequest, ServiceRequestIntent, ServiceRequestStatus},
    value::{EncounterId, PatientId},
};
use rand::{Rng, SeedableRng, rng, rngs::StdRng};

pub fn fake_service_request_for(
    patient_id: &PatientId,
    encounter_id: Option<&EncounterId>,
) -> ServiceRequest {
    let mut rng = rng();
    fake_service_request_for_with_rng(patient_id, encounter_id, &mut rng)
}

pub fn fake_service_request_for_with_seed(
    seed: u64,
    patient_id: &PatientId,
    encounter_id: Option<&EncounterId>,
) -> ServiceRequest {
    let mut rng = StdRng::seed_from_u64(seed);
    fake_service_request_for_with_rng(patient_id, encounter_id, &mut rng)
}

pub fn fake_service_request_for_with_rng<R: Rng + ?Sized>(
    patient_id: &PatientId,
    encounter_id: Option<&EncounterId>,
    rng: &mut R,
) -> ServiceRequest {
    let id = fake_service_request_id_with_rng(rng);
    let status = fake_service_request_status_with_rng(rng);
    let intent = normalize_intent(status, fake_service_request_intent_with_rng(rng));
    let description = fake_order_description_with_rng(rng);
    let encounter_id = encounter_id.cloned();

    ServiceRequest::new(
        id,
        patient_id.clone(),
        encounter_id,
        status,
        intent,
        description,
    )
}

fn normalize_intent(
    status: ServiceRequestStatus,
    intent: ServiceRequestIntent,
) -> ServiceRequestIntent {
    match status {
        ServiceRequestStatus::Draft => match intent {
            ServiceRequestIntent::Plan | ServiceRequestIntent::Proposal => intent,
            _ => ServiceRequestIntent::Plan,
        },
        ServiceRequestStatus::Completed | ServiceRequestStatus::Cancelled => {
            ServiceRequestIntent::Order
        }
        _ => intent,
    }
}
