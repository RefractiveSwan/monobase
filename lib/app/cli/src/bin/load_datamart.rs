use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use clap::Parser;
use dfps_compliance::{assert_export_allowed, load_policy_from_env};
use dfps_configuration::load_env;
use dfps_datamart::{
    LoadSummary, WarehouseConfig, connect_sqlite, load_from_pipeline_output, migrate,
};
use dfps_pipeline::{PipelineOutput, bundle_to_mapped_sr};
use serde::Deserialize;

#[derive(Parser)]
#[command(
    name = "load_datamart",
    about = "Load PipelineOutput rows into the warehouse schema (SQLite)"
)]
struct Args {
    /// NDJSON PipelineOutput, or Bundle NDJSON with --input-kind bundle
    #[arg(long, value_name = "PATH")]
    input: PathBuf,
    /// Input kind: pipeline (PipelineOutput NDJSON) or bundle (FHIR Bundle NDJSON)
    #[arg(long, value_enum, default_value = "pipeline")]
    input_kind: InputKind,
}

#[derive(Debug, Copy, Clone, Deserialize, clap::ValueEnum)]
enum InputKind {
    Pipeline,
    Bundle,
}

#[derive(Default)]
struct AggregateSummary {
    patients: u64,
    encounters: u64,
    codes: u64,
    ncit: u64,
    facts: u64,
}

impl AggregateSummary {
    fn add(&mut self, summary: LoadSummary) {
        self.patients += summary.patients;
        self.encounters += summary.encounters;
        self.codes += summary.codes;
        self.ncit += summary.ncit;
        self.facts += summary.facts;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_env("app.cli").map_err(|err| format!("dfps_cli env error: {err}"))?;
    let args = Args::parse();
    let policy = load_policy_from_env()?;

    let cfg = WarehouseConfig::from_env().map_err(|err| format!("{err}"))?;
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let pool = connect_sqlite(&cfg).await?;
        migrate(&pool).await?;

        let outputs = read_inputs(&args)?;
        let mut agg = AggregateSummary::default();
        for output in outputs {
            enforce_export_policy(&output, &policy)?;
            let summary = load_from_pipeline_output(&pool, &output).await?;
            agg.add(summary);
        }

        let summary_line = serde_json::json!({
            "kind": "load_summary",
            "patients": agg.patients,
            "encounters": agg.encounters,
            "codes": agg.codes,
            "ncit": agg.ncit,
            "facts": agg.facts
        });
        println!("{}", serde_json::to_string(&summary_line)?);
        Ok::<(), Box<dyn std::error::Error>>(())
    })?;

    Ok(())
}

fn read_inputs(args: &Args) -> Result<Vec<PipelineOutput>, Box<dyn std::error::Error>> {
    let mut outputs = Vec::new();
    let file = File::open(&args.input)?;
    let mut reader = BufReader::new(file);
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer)?;
    if buffer.trim().is_empty() {
        return Ok(outputs);
    }

    match args.input_kind {
        InputKind::Pipeline => {
            for line in buffer.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let output: PipelineOutput = serde_json::from_str(trimmed)?;
                outputs.push(output);
            }
        }
        InputKind::Bundle => {
            for line in buffer.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let bundle: dfps_core::fhir::Bundle = serde_json::from_str(trimmed)?;
                let mapped = bundle_to_mapped_sr(&bundle)
                    .map_err(|err| format!("pipeline mapping error: {err}"))?;
                outputs.push(mapped);
            }
        }
    }

    Ok(outputs)
}

fn enforce_export_policy(
    output: &PipelineOutput,
    policy: &dfps_compliance::Policy,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tiers = std::collections::BTreeSet::new();
    for result in &output.mapping_results {
        if let Some(label) = result.license_tier.as_deref() {
            if let Some(tier) = parse_license_tier(label) {
                tiers.insert(tier);
            }
        }
    }
    assert_export_allowed(&tiers.into_iter().collect::<Vec<_>>(), policy)
        .map_err(|err| format!("export blocked by compliance policy: {err}").into())
}

fn parse_license_tier(value: &str) -> Option<dfps_terminology::codesystem::LicenseTier> {
    match value.trim() {
        "licensed" => Some(dfps_terminology::codesystem::LicenseTier::Licensed),
        "open" => Some(dfps_terminology::codesystem::LicenseTier::Open),
        "internal_only" => Some(dfps_terminology::codesystem::LicenseTier::InternalOnly),
        _ => None,
    }
}
