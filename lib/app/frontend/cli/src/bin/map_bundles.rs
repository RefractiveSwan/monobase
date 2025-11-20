use std::collections::HashSet;
use std::path::PathBuf;

use clap::Parser;
use dfps_cli::cli_core::{
    CliError, CliResult, enforce_metrics_gate, init_cli_env, init_logging, input_reader,
    json_stream, load_policy, pipeline_vector_context_from_env, run_bin, tag_metrics, write_record,
};
use dfps_contracts::{MappingState, PipelineMetrics};
use dfps_core::fhir::Bundle;
use dfps_ingestion::validation::ValidationSeverity;
use dfps_observability::{log_no_match, log_pipeline_output};
use dfps_pipeline::{DefaultPipeline, PipelinePort, PipelineRunConfig};
use log::{info, warn};

#[derive(Parser)]
#[command(
    name = "map_bundles",
    about = "Ingest FHIR bundles and emit staging + mapping rows"
)]

struct Args {
    /// NDJSON file containing FHIR Bundles (defaults to stdin)
    #[arg(value_name = "INPUT")]
    input: Option<PathBuf>,
    /// Log level for env_logger (error,warn,info,debug,trace)
    #[arg(long, value_name = "LEVEL", default_value = "info")]
    log_level: String,
    /// Exit with error if any code is blocked by compliance policy
    #[arg(long)]
    fail_on_license_block: bool,
}

fn main() {
    run_bin("map_bundles", run);
}

fn run() -> CliResult<()> {
    init_cli_env()?;
    let args = Args::parse();
    init_logging(&args.log_level)?;
    let policy = load_policy()?;
    let reader = input_reader(args.input.as_ref())?;
    let mut bundles = json_stream::<Bundle>(reader);

    let mut dims_seen: HashSet<String> = HashSet::new();
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    let mut metrics = PipelineMetrics::default();
    tag_metrics(&mut metrics, &policy);
    let vector_ctx = match pipeline_vector_context_from_env() {
        Ok(ctx) => ctx,
        Err(err) => {
            warn!("vector context unavailable: {err}");
            None
        }
    };
    let pipeline = DefaultPipeline::default();
    let config = PipelineRunConfig::default();

    while let Some(bundle) = bundles.next() {
        let bundle = bundle?;
        let exec = pipeline
            .map_bundle_with_validation(&bundle, &config, vector_ctx.as_ref())
            .map_err(|err| CliError::invalid(format!("pipeline error: {err}")))?;
        if exec.validation.has_errors() {
            warn!(
                "validation detected {} issue(s) ({} errors).",
                exec.validation.issues.len(),
                exec.validation
                    .issues
                    .iter()
                    .filter(|issue| matches!(issue.severity, ValidationSeverity::Error))
                    .count()
            );
        } else if !exec.validation.issues.is_empty() {
            info!(
                "validation reported {} warning(s)/info messages.",
                exec.validation.issues.len()
            );
        }
        for issue in &exec.validation.issues {
            write_record(&mut handle, "validation_issue", issue)?;
        }
        let output = exec.output;
        let vector_usage = output.vector_usage.clone();
        log_pipeline_output(
            &output.flats,
            &output.exploded_codes,
            &output.mapping_results,
            &mut metrics,
            vector_usage,
            None,
        );

        write_record(&mut handle, "pipeline_output", &output)?;

        for flat in &output.flats {
            write_record(&mut handle, "staging_flat", flat)?;
        }
        for code in &output.exploded_codes {
            write_record(&mut handle, "staging_code", code)?;
        }
        for mapping in &output.mapping_results {
            write_record(&mut handle, "mapping_result", mapping)?;
            if matches!(mapping.state, MappingState::NoMatch) {
                log_no_match(mapping);
            }
        }
        for concept in &output.dim_concepts {
            if dims_seen.insert(concept.ncit_id.clone()) {
                write_record(&mut handle, "dim_concept", concept)?;
            }
        }
    }

    info!(
        target: "dfps_pipeline",
        "pipeline_complete bundles={} automap={} review={} nomatch={} license_blocked={} compliance_mode={}",
        metrics.bundle_count,
        metrics.auto_mapped,
        metrics.needs_review,
        metrics.no_match,
        metrics.license_blocked,
        policy.mode.as_str()
    );
    if metrics.license_blocked > 0 {
        warn!(
            target: "dfps_compliance",
            "audit compliance_blocked reason=license_blocked mode={} count={}",
            policy.mode.as_str(),
            metrics.license_blocked
        );
    }
    write_record(&mut handle, "metrics_summary", &metrics)?;

    enforce_metrics_gate(&policy, &metrics, args.fail_on_license_block)?;

    Ok(())
}
