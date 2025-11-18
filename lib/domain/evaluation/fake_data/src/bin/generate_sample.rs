use dfps_fake_data::scenarios::{
    fake_service_request_scenario, fake_service_request_scenario_with_seed,
};
use serde_json::to_string;
use std::env;

fn main() {
    if let Err(err) = dfps_configuration::load_env("domain.fake_data") {
        eprintln!("dfps_fake_data env error: {err}");
        std::process::exit(1);
    }
    let mut args = env::args().skip(1);
    let count = args
        .next()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1);
    let seed = args.next().and_then(|value| value.parse::<u64>().ok());

    if let Some(seed) = seed {
        emit_seeded(count, seed);
    } else {
        emit_random(count);
    }
}

fn emit_seeded(count: usize, seed: u64) {
    for offset in 0..count {
        let scenario = fake_service_request_scenario_with_seed(seed + offset as u64);
        print_scenario(&scenario);
    }
}

fn emit_random(count: usize) {
    for _ in 0..count {
        let scenario = fake_service_request_scenario();
        print_scenario(&scenario);
    }
}

fn print_scenario(scenario: &dfps_fake_data::scenarios::ServiceRequestScenario) {
    match to_string(scenario) {
        Ok(json) => println!("{json}"),
        Err(err) => eprintln!("failed to serialize scenario: {err}"),
    }
}
