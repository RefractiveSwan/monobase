use clap::Parser;
use dfps_fake_data::raw_fhir::{fake_fhir_bundle_scenario, fake_fhir_bundle_scenario_with_seed};
use serde_json::to_string;

#[derive(Parser)]
#[command(
    name = "generate_fhir_bundle",
    about = "Emit fake FHIR Bundles as NDJSON for local testing"
)]
struct Args {
    /// Number of bundles to emit
    #[arg(short, long, default_value_t = 1)]
    count: usize,
    /// Seed for reproducible bundles
    #[arg(short, long)]
    seed: Option<u64>,
}

fn main() {
    let args = Args::parse();
    if let Some(seed) = args.seed {
        emit_seeded(args.count, seed);
    } else {
        emit_random(args.count);
    }
}

fn emit_seeded(count: usize, seed: u64) {
    for offset in 0..count {
        let scenario = fake_fhir_bundle_scenario_with_seed(seed + offset as u64);
        print_bundle(&scenario.bundle);
    }
}

fn emit_random(count: usize) {
    for _ in 0..count {
        let scenario = fake_fhir_bundle_scenario();
        print_bundle(&scenario.bundle);
    }
}

fn print_bundle(bundle: &dfps_core::fhir::Bundle) {
    match to_string(bundle) {
        Ok(json) => println!("{json}"),
        Err(err) => eprintln!("failed to serialize bundle: {err}"),
    }
}
