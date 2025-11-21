use crate::fake_data::{rng, value::fake_patient_id_with_rng};
use dfps_core::patient::Patient;
use rand::Rng;

pub fn fake_patient() -> Patient {
    rng::with_global_rng(fake_patient_with_rng)
}

pub fn fake_patient_with_seed(seed: u64) -> Patient {
    let mut rng = rng::rng_from_seed(seed);
    fake_patient_with_rng(&mut rng)
}

pub fn fake_patient_with_rng<R: Rng + ?Sized>(rng: &mut R) -> Patient {
    Patient::new(fake_patient_id_with_rng(rng))
}
