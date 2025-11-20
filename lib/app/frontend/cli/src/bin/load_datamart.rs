use clap::Parser;
use dfps_cli::cli_core::{
    CliError, CliResult, init_cli_env, input_reader, json_stream, load_policy,
    pipeline_vector_context_from_env, run_bin, write_record,
};
use dfps_contracts::{LoadSummary, PipelineOutput};
use dfps_core::fhir::Bundle;
use dfps_datamart::{
    LoadError, WarehouseConfig, connect_sqlite, load_from_pipeline_output, migrate,
};
use dfps_pipeline::bundle_to_mapped_sr_with_vector_context;
use serde::Deserialize;
use std::path::PathBuf;

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

fn main() {
    run_bin("load_datamart", run);
}

fn run() -> CliResult<()> {
    init_cli_env()?;
    let args = Args::parse();
    let policy = load_policy()?;
    let vector_ctx = match pipeline_vector_context_from_env() {
        Ok(ctx) => ctx,
        Err(err) => {
            log::warn!(
                "vector context unavailable ({err}); bundle inputs will use lexical mapping"
            );
            None
        }
    };
    let cfg = WarehouseConfig::from_env()
        .map_err(|err| CliError::config(format!("warehouse config error: {err}")))?;
    let outputs = read_inputs(&args, vector_ctx.as_ref())?;

    let rt = tokio::runtime::Runtime::new()
        .map_err(|err| CliError::external(format!("runtime init failed: {err}")))?;
    rt.block_on(async move {
        let pool = connect_sqlite(&cfg).await?;
        migrate(&pool).await?;

        let mut agg = LoadSummary::default();
        for output in outputs {
            let summary = load_from_pipeline_output(&pool, &output, &policy)
                .await
                .map_err(map_load_error)?;
            agg.accumulate(&summary);
        }

        emit_summary(&agg)?;
        Ok::<(), CliError>(())
    })?;
    Ok(())
}

fn read_inputs(
    args: &Args,
    vector_ctx: Option<&dfps_pipeline::VectorPipelineContext>,
) -> CliResult<Vec<PipelineOutput>> {
    let reader = input_reader(Some(&args.input))?;
    match args.input_kind {
        InputKind::Pipeline => {
            let mut stream = json_stream::<PipelineOutput>(reader);
            let mut outputs = Vec::new();
            while let Some(record) = stream.next() {
                outputs.push(record?);
            }
            Ok(outputs)
        }
        InputKind::Bundle => {
            let mut stream = json_stream::<Bundle>(reader);
            let mut outputs = Vec::new();
            while let Some(bundle) = stream.next() {
                let bundle = bundle?;
                let output = bundle_to_mapped_sr_with_vector_context(&bundle, vector_ctx)
                    .map_err(|err| CliError::invalid(format!("pipeline mapping error: {err}")))?;
                outputs.push(output);
            }
            Ok(outputs)
        }
    }
}

fn emit_summary(summary: &LoadSummary) -> CliResult<()> {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    write_record(&mut handle, "load_summary", summary)
}

fn map_load_error(err: LoadError) -> CliError {
    match err {
        LoadError::Compliance(msg) => CliError::compliance(msg),
        LoadError::Sql(inner) => CliError::external(inner.to_string()),
    }
}
