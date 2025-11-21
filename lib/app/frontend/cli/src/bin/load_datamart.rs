use clap::Parser;
use refractive_swan_cli::cli_core::{
    CliError, CliResult, JsonStream, init_cli_env, input_reader, json_stream, load_policy,
    pipeline_vector_context_from_env, run_bin, write_record,
};
use refractive_swan_cli_dto::LoadSummary;
use refractive_swan_core::fhir::Bundle;
use refractive_swan_datamart::{LoadError, WarehouseConfig, connect_sqlite, load_streaming_iter, migrate};
use refractive_swan_pipeline::{
    DefaultPipeline, PipelineOutput, PipelinePort, PipelineRunConfig, VectorPipelineContext,
};
use serde::Deserialize;
use std::io::BufRead;
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
    let mut outputs = read_inputs_stream(&args, vector_ctx)?;

    let rt = tokio::runtime::Runtime::new()
        .map_err(|err| CliError::external(format!("runtime init failed: {err}")))?;
    let summary = rt.block_on(async {
        let pool = connect_sqlite(&cfg).await?;
        migrate(&pool).await?;
        load_streaming_iter(&pool, &mut outputs, &policy)
            .await
            .map_err(map_load_error)
    })?;
    outputs.finish()?;
    emit_summary(&summary)?;
    Ok(())
}

fn read_inputs_stream(
    args: &Args,
    vector_ctx: Option<VectorPipelineContext>,
) -> CliResult<PipelineOutputStream> {
    let reader = input_reader(Some(&args.input))?;
    match args.input_kind {
        InputKind::Pipeline => Ok(PipelineOutputStream::from_pipeline(reader)),
        InputKind::Bundle => Ok(PipelineOutputStream::from_bundle(reader, vector_ctx)),
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

enum PipelineOutputStreamInner {
    Pipeline(JsonStream<Box<dyn BufRead>, PipelineOutput>),
    Bundle {
        stream: JsonStream<Box<dyn BufRead>, Bundle>,
        pipeline: DefaultPipeline,
        config: PipelineRunConfig<'static>,
        vector: Option<VectorPipelineContext>,
    },
}

struct PipelineOutputStream {
    inner: PipelineOutputStreamInner,
    error: Option<CliError>,
}

impl PipelineOutputStream {
    fn from_pipeline(reader: Box<dyn BufRead>) -> Self {
        Self {
            inner: PipelineOutputStreamInner::Pipeline(json_stream(reader)),
            error: None,
        }
    }

    fn from_bundle(reader: Box<dyn BufRead>, vector: Option<VectorPipelineContext>) -> Self {
        Self {
            inner: PipelineOutputStreamInner::Bundle {
                stream: json_stream(reader),
                pipeline: DefaultPipeline::default(),
                config: PipelineRunConfig::default(),
                vector,
            },
            error: None,
        }
    }

    fn finish(self) -> CliResult<()> {
        match self.error {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}

impl Iterator for PipelineOutputStream {
    type Item = PipelineOutput;

    fn next(&mut self) -> Option<Self::Item> {
        if self.error.is_some() {
            return None;
        }
        match &mut self.inner {
            PipelineOutputStreamInner::Pipeline(stream) => match stream.next() {
                Some(Ok(output)) => Some(output),
                Some(Err(err)) => {
                    self.error = Some(err);
                    None
                }
                None => None,
            },
            PipelineOutputStreamInner::Bundle {
                stream,
                pipeline,
                config,
                vector,
            } => match stream.next() {
                Some(Ok(bundle)) => {
                    match pipeline.map_bundle_with_validation(&bundle, config, vector.as_ref()) {
                        Ok(exec) => Some(exec.output),
                        Err(err) => {
                            self.error =
                                Some(CliError::invalid(format!("pipeline mapping error: {err}")));
                            None
                        }
                    }
                }
                Some(Err(err)) => {
                    self.error = Some(err);
                    None
                }
                None => None,
            },
        }
    }
}
