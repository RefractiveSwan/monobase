use dfps_core::{order::ServiceRequest, staging::StgSrCodeExploded};
use dfps_eval::EvalCase;
use dfps_eval::fake_data::{
    ServiceRequestScenario, fake_service_request_for,
    fixtures::{self, Registry},
    scenarios::{fake_service_request_scenario, fake_service_request_scenario_with_seed},
};
use once_cell::sync::Lazy;

static MAPPING_REGISTRY: Lazy<Registry> = Lazy::new(Registry::default);

pub fn service_request_scenario() -> ServiceRequestScenario {
    fake_service_request_scenario()
}

pub fn service_request_scenario_with_seed(seed: u64) -> ServiceRequestScenario {
    fake_service_request_scenario_with_seed(seed)
}

pub fn service_request() -> ServiceRequest {
    service_request_scenario().service_request
}

pub fn service_request_with_seed(seed: u64) -> ServiceRequest {
    service_request_scenario_with_seed(seed).service_request
}

pub fn standalone_service_request() -> ServiceRequest {
    let scenario = service_request_scenario();
    fake_service_request_for(&scenario.patient.id, Some(&scenario.encounter.id))
}

fn mapping_registry() -> &'static Registry {
    Lazy::force(&MAPPING_REGISTRY)
}

fn load_mapping_code(name: &str) -> StgSrCodeExploded {
    let case = fixtures::mapping::load(mapping_registry(), name)
        .unwrap_or_else(|err| panic!("mapping fixture {name} should load: {err}"));
    StgSrCodeExploded {
        sr_id: case.sr_id,
        system: case.system,
        code: case.code,
        display: case.display,
    }
}

pub fn mapping_cpt_code() -> StgSrCodeExploded {
    load_mapping_code("mapping_cpt_78815")
}

pub fn mapping_snomed_code() -> StgSrCodeExploded {
    load_mapping_code("mapping_snomed_pet")
}

pub fn mapping_unknown_code() -> StgSrCodeExploded {
    load_mapping_code("mapping_unknown")
}

pub fn mapping_unknown_system_code() -> StgSrCodeExploded {
    load_mapping_code("mapping_unknown_system")
}

pub fn mapping_ncit_obo_code() -> StgSrCodeExploded {
    load_mapping_code("mapping_ncit_obo")
}

pub fn eval_pet_ct_small_cases() -> Vec<EvalCase> {
    dfps_eval::load_dataset("pet_ct_small")
        .expect("pet_ct_small dataset should load from lib/domain/eval/data/eval")
}
