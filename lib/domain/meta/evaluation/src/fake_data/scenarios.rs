use crate::fake_data::{
    encounter::fake_encounter_for_patient_with_rng, order::fake_service_request_for_with_rng,
    patient::fake_patient_with_rng, rng,
};
use refractive_swan_core::{encounter::Encounter, order::ServiceRequest, patient::Patient};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceRequestScenario {
    pub patient: Patient,
    pub encounter: Encounter,
    pub service_request: ServiceRequest,
}

pub fn fake_service_request_scenario() -> ServiceRequestScenario {
    rng::with_global_rng(fake_service_request_scenario_with_rng)
}

pub fn fake_service_request_scenario_with_seed(seed: u64) -> ServiceRequestScenario {
    let mut rng = rng::rng_from_seed(seed);
    fake_service_request_scenario_with_rng(&mut rng)
}

pub fn fake_service_request_scenario_with_rng<R: Rng + ?Sized>(
    rng: &mut R,
) -> ServiceRequestScenario {
    let patient = fake_patient_with_rng(rng);
    let encounter = fake_encounter_for_patient_with_rng(&patient.id, rng);
    let service_request = fake_service_request_for_with_rng(&patient.id, Some(&encounter.id), rng);

    ServiceRequestScenario {
        patient,
        encounter,
        service_request,
    }
}
