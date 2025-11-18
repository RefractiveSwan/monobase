use dfps_core::{
    order::{ServiceRequestIntent, ServiceRequestStatus},
    value::{EncounterId, PatientId, ServiceRequestId},
};
use fake::{Fake, Faker};
use rand::{Rng, SeedableRng, prelude::IndexedRandom, rng, rngs::StdRng};

const ORDER_DESCRIPTIONS: &[&str] = &[
    "PET/CT for staging",
    "PET/CT restaging",
    "PET/CT to evaluate treatment response",
    "FDG PET for metabolic activity",
    "Whole-body PET/CT follow-up",
];

fn next_numeric_code<R: Rng + ?Sized>(rng: &mut R) -> u32 {
    let raw: u32 = Faker.fake_with_rng(rng);
    100_000 + (raw % 900_000)
}

fn format_id(prefix: &str, value: u32) -> String {
    format!("{prefix}-{value:06}")
}

pub fn fake_patient_id() -> PatientId {
    let mut rng = rng();
    fake_patient_id_with_rng(&mut rng)
}

pub fn fake_patient_id_with_seed(seed: u64) -> PatientId {
    let mut rng = StdRng::seed_from_u64(seed);
    fake_patient_id_with_rng(&mut rng)
}

pub fn fake_patient_id_with_rng<R: Rng + ?Sized>(rng: &mut R) -> PatientId {
    PatientId::new(format_id("PAT", next_numeric_code(rng)))
}

pub fn fake_encounter_id() -> EncounterId {
    let mut rng = rng();
    fake_encounter_id_with_rng(&mut rng)
}

pub fn fake_encounter_id_with_seed(seed: u64) -> EncounterId {
    let mut rng = StdRng::seed_from_u64(seed);
    fake_encounter_id_with_rng(&mut rng)
}

pub fn fake_encounter_id_with_rng<R: Rng + ?Sized>(rng: &mut R) -> EncounterId {
    EncounterId::new(format_id("ENC", next_numeric_code(rng)))
}

pub fn fake_service_request_id() -> ServiceRequestId {
    let mut rng = rng();
    fake_service_request_id_with_rng(&mut rng)
}

pub fn fake_service_request_id_with_seed(seed: u64) -> ServiceRequestId {
    let mut rng = StdRng::seed_from_u64(seed);
    fake_service_request_id_with_rng(&mut rng)
}

pub fn fake_service_request_id_with_rng<R: Rng + ?Sized>(rng: &mut R) -> ServiceRequestId {
    ServiceRequestId::new(format_id("SR", next_numeric_code(rng)))
}

pub fn fake_service_request_status() -> ServiceRequestStatus {
    let mut rng = rng();
    fake_service_request_status_with_rng(&mut rng)
}

pub fn fake_service_request_status_with_seed(seed: u64) -> ServiceRequestStatus {
    let mut rng = StdRng::seed_from_u64(seed);
    fake_service_request_status_with_rng(&mut rng)
}

pub fn fake_service_request_status_with_rng<R: Rng + ?Sized>(rng: &mut R) -> ServiceRequestStatus {
    Faker.fake_with_rng(rng)
}

pub fn fake_service_request_intent() -> ServiceRequestIntent {
    let mut rng = rng();
    fake_service_request_intent_with_rng(&mut rng)
}

pub fn fake_service_request_intent_with_seed(seed: u64) -> ServiceRequestIntent {
    let mut rng = StdRng::seed_from_u64(seed);
    fake_service_request_intent_with_rng(&mut rng)
}

pub fn fake_service_request_intent_with_rng<R: Rng + ?Sized>(rng: &mut R) -> ServiceRequestIntent {
    Faker.fake_with_rng(rng)
}

pub fn fake_order_description() -> String {
    let mut rng = rng();
    fake_order_description_with_rng(&mut rng)
}

pub fn fake_order_description_with_seed(seed: u64) -> String {
    let mut rng = StdRng::seed_from_u64(seed);
    fake_order_description_with_rng(&mut rng)
}

pub fn fake_order_description_with_rng<R: Rng + ?Sized>(rng: &mut R) -> String {
    ORDER_DESCRIPTIONS
        .choose(rng)
        .copied()
        .unwrap_or(ORDER_DESCRIPTIONS[0])
        .to_string()
}
