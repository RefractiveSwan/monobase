use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;

use clap::Parser;
use dfps_compliance::load_policy_from_env;
use dfps_configuration::load_env;
use dfps_core::fhir::Bundle;
use dfps_ingestion::validation::{ValidationSeverity, validate_bundle};
use dfps_observability::{PipelineMetrics, log_no_match, log_pipeline_output};
use dfps_pipeline::bundle_to_mapped_sr;
use log::{LevelFilter, info, warn};
use serde::Serialize;

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

#[derive(Serialize)]
struct OutputRecord<'a, T> {
    kind: &'a str,
    #[serde(flatten)]
    value: T,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_env("app.cli").map_err(|err| format!("dfps_cli env error: {err}"))?;
    let args = Args::parse();
    init_logging(&args.log_level)?;
    let policy = load_policy_from_env()?;
    let reader: Box<dyn BufRead> = match &args.input {
        Some(path) => Box::new(BufReader::new(File::open(path)?)),
        None => Box::new(BufReader::new(io::stdin())),
    };

    let mut dims_seen: HashSet<String> = HashSet::new();
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let mut metrics = PipelineMetrics::default();
    metrics.compliance_mode = Some(policy.mode.as_str().to_string());

    for line in reader.lines() {
        let raw = line?;
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let bundle: Bundle = serde_json::from_str(trimmed)?;
        let validation = validate_bundle(&bundle);
        if validation.has_errors() {
            warn!(
                "validation detected {} issue(s) ({} errors).",
                validation.issues.len(),
                validation
                    .issues
                    .iter()
                    .filter(|issue| matches!(issue.severity, ValidationSeverity::Error))
                    .count()
            );
        } else if !validation.issues.is_empty() {
            info!(
                "validation reported {} warning(s)/info messages.",
                validation.issues.len()
            );
        }
        for issue in &validation.issues {
            write_json(&mut handle, "validation_issue", issue)?;
        }
        let output = bundle_to_mapped_sr(&bundle)?;
        log_pipeline_output(
            &output.flats,
            &output.exploded_codes,
            &output.mapping_results,
            &mut metrics,
            output.vector_usage,
            None,
        );

        for flat in &output.flats {
            write_json(&mut handle, "staging_flat", flat)?;
        }
        for code in &output.exploded_codes {
            write_json(&mut handle, "staging_code", code)?;
        }
        for mapping in &output.mapping_results {
            write_json(&mut handle, "mapping_result", mapping)?;
            if matches!(mapping.state, dfps_core::mapping::MappingState::NoMatch) {
                log_no_match(mapping);
            }
        }
        for concept in &output.dim_concepts {
            if dims_seen.insert(concept.ncit_id.clone()) {
                write_json(&mut handle, "dim_concept", concept)?;
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
    write_json(&mut handle, "metrics_summary", &metrics)?;

    if args.fail_on_license_block && metrics.license_blocked > 0 {
        return Err(format!(
            "{} mapping result(s) blocked by compliance mode {}; rerun without --fail-on-license-block or adjust DFPS_COMPLIANCE_MODE",
            metrics.license_blocked,
            policy.mode.as_str()
        )
        .into());
    }

    Ok(())
}

fn init_logging(level: &str) -> Result<(), Box<dyn std::error::Error>> {
    let filter = level
        .parse::<LevelFilter>()
        .map_err(|_| format!("invalid log level '{level}'"))?;
    env_logger::Builder::from_default_env()
        .filter_level(filter)
        .try_init()?;
    Ok(())
}

fn write_json<T: Serialize>(
    handle: &mut impl Write,
    kind: &'static str,
    value: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    let record = serde_json::to_string(&OutputRecord { kind, value })?;
    writeln!(handle, "{record}")?;
    Ok(())
}
