use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use clap::Parser;
use dfps_compliance::ComplianceConfig;
use dfps_configuration::load_env;
use dfps_contracts::{LoadSummary, PipelineOutput};
use dfps_datamart::{WarehouseConfig, connect_sqlite, load_from_pipeline_output, migrate};
use dfps_pipeline::{VectorPipelineContext, bundle_to_mapped_sr_with_vector_context};
use serde::{Deserialize, Serialize};

mod vector_ctx;
use vector_ctx::pipeline_vector_context_from_env;

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

#[derive(Serialize)]
struct OutputRecord<'a, T> {
    kind: &'a str,
    #[serde(flatten)]
    value: &'a T,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_env("app.cli").map_err(|err| format!("dfps_cli env error: {err}"))?;
    let args = Args::parse();
    let compliance = ComplianceConfig::from_env()?;
    let policy = compliance.load_policy()?;
    let vector_ctx = pipeline_vector_context_from_env();
    let cfg = WarehouseConfig::from_env().map_err(|err| format!("{err}"))?;
    let outputs = read_inputs(&args, vector_ctx.as_ref())?;
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let pool = connect_sqlite(&cfg).await?;
        migrate(&pool).await?;

        let mut agg = LoadSummary::default();
        for output in outputs {
            let summary = load_from_pipeline_output(&pool, &output, &policy).await?;
            agg.accumulate(&summary);
        }

        emit_summary(&agg)?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })?;

    Ok(())
}

fn read_inputs(
    args: &Args,
    vector_ctx: Option<&VectorPipelineContext>,
) -> Result<Vec<PipelineOutput>, Box<dyn std::error::Error>> {
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
                let mapped = bundle_to_mapped_sr_with_vector_context(&bundle, vector_ctx)
                    .map_err(|err| format!("pipeline mapping error: {err}"))?;
                outputs.push(mapped);
            }
        }
    }

    Ok(outputs)
}

fn emit_summary(summary: &LoadSummary) -> Result<(), Box<dyn std::error::Error>> {
    let record = OutputRecord {
        kind: "load_summary",
        value: summary,
    };
    println!("{}", serde_json::to_string(&record)?);
    Ok(())
}
