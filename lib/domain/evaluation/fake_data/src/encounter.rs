use crate::value::fake_encounter_id_with_rng;
use dfps_core::{encounter::Encounter, value::PatientId};
use rand::{Rng, SeedableRng, rng, rngs::StdRng};

pub fn fake_encounter_for_patient(patient_id: &PatientId) -> Encounter {
    let mut rng = rng();
    fake_encounter_for_patient_with_rng(patient_id, &mut rng)
}

pub fn fake_encounter_for_patient_with_seed(seed: u64, patient_id: &PatientId) -> Encounter {
    let mut rng = StdRng::seed_from_u64(seed);
    fake_encounter_for_patient_with_rng(patient_id, &mut rng)
}

pub fn fake_encounter_for_patient_with_rng<R: Rng + ?Sized>(
    patient_id: &PatientId,
    rng: &mut R,
) -> Encounter {
    Encounter::new(fake_encounter_id_with_rng(rng), patient_id.clone())
}
