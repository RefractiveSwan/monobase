use crate::{
    encounter::fake_encounter_for_patient_with_rng, order::fake_service_request_for_with_rng,
    patient::fake_patient_with_rng,
};
use dfps_core::{encounter::Encounter, order::ServiceRequest, patient::Patient};
use rand::{Rng, SeedableRng, rng, rngs::StdRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceRequestScenario {
    pub patient: Patient,
    pub encounter: Encounter,
    pub service_request: ServiceRequest,
}

pub fn fake_service_request_scenario() -> ServiceRequestScenario {
    let mut rng = rng();
    fake_service_request_scenario_with_rng(&mut rng)
}

pub fn fake_service_request_scenario_with_seed(seed: u64) -> ServiceRequestScenario {
    let mut rng = StdRng::seed_from_u64(seed);
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
