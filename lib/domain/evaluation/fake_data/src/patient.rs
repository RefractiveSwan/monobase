use crate::value::fake_patient_id_with_rng;
use dfps_core::patient::Patient;
use rand::{Rng, SeedableRng, rng, rngs::StdRng};

pub fn fake_patient() -> Patient {
    let mut rng = rng();
    fake_patient_with_rng(&mut rng)
}

pub fn fake_patient_with_seed(seed: u64) -> Patient {
    let mut rng = StdRng::seed_from_u64(seed);
    fake_patient_with_rng(&mut rng)
}

pub fn fake_patient_with_rng<R: Rng + ?Sized>(rng: &mut R) -> Patient {
    Patient::new(fake_patient_id_with_rng(rng))
}
