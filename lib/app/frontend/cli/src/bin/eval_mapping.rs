use std::fs::{File, create_dir_all};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};

use clap::Parser;
use dfps_cli::cli_core::{
    CliError, CliResult, dataset_store_from_env, init_cli_env, parse_json_file, read_to_string,
    run_bin, write_record,
};
use dfps_contracts::{DatasetManifest, EvalRunResponse, EvalSummary};
use dfps_mapping::map_staging_codes;
use serde::Deserialize;

#[derive(Parser)]
#[command(
    name = "eval_mapping",
    about = "Run NCIt mapping evaluation over a gold-standard NDJSON file"
)]
struct Args {
    /// NDJSON gold file with EvalCase rows
    #[arg(long, value_name = "PATH", conflicts_with = "dataset")]
    input: Option<PathBuf>,
    /// Named dataset under DFPS_EVAL_DATA_ROOT (e.g., pet_ct_small)
    #[arg(long, value_name = "NAME", conflicts_with = "input")]
    dataset: Option<String>,
    /// Stream chunk size when reading NDJSON
    #[arg(long, value_name = "N", default_value_t = dfps_eval::DEFAULT_CHUNK_SIZE as u32)]
    chunk_size: u32,
    /// Directory for machine-readable artifacts (summary/results)
    #[arg(long, value_name = "DIR")]
    out_dir: Option<PathBuf>,
    /// Markdown report output path
    #[arg(long, value_name = "PATH")]
    report: Option<PathBuf>,
    /// Threshold JSON file enforcing minimum metrics
    #[arg(long, value_name = "PATH")]
    thresholds: Option<PathBuf>,
    /// Emit per-case EvalResult rows after the summary
    #[arg(long)]
    dump_details: bool,
    /// Compare against a deterministic fingerprint file (sha256). Fails if mismatched.
    #[arg(long, value_name = "PATH")]
    deterministic: Option<PathBuf>,
    /// Compare current metrics against a baseline summary JSON (EvalSummary). Exit non-zero on regression.
    #[arg(long, value_name = "PATH")]
    compare_to: Option<PathBuf>,
    /// Desired top-k (placeholder until multi-candidate surfaces). Default: 1.
    #[arg(long, value_name = "N", default_value_t = 1)]
    top_k: usize,
}

#[derive(Debug, Deserialize)]
struct ThresholdConfig {
    min_precision: Option<f32>,
    min_recall: Option<f32>,
    min_f1: Option<f32>,
    min_accuracy: Option<f32>,
    min_auto_precision: Option<f32>,
    min_coverage: Option<f32>,
    min_top1: Option<f32>,
    allow_no_match_reason: Option<Vec<String>>,
}

fn main() {
    run_bin("eval_mapping", run);
}

fn run() -> CliResult<()> {
    init_cli_env()?;
    let args = Args::parse();
    let dataset_store = dataset_store_from_env();

    let chunk_size = args.chunk_size as usize;
    let mut manifest: Option<DatasetManifest> = None;
    let (summary, raw_dataset): (EvalSummary, String) = if let Some(name) = &args.dataset {
        let outcome = dataset_store
            .load_dataset_with_manifest(name)
            .map_err(|err| CliError::invalid(format!("failed to load dataset {name}: {err}")))?;
        if !outcome.checksum_ok {
            eprintln!(
                "warning: dataset {name} checksum mismatch (expected {}, actual {})",
                outcome.manifest.sha256, outcome.computed_sha256
            );
        }
        manifest = Some(outcome.manifest.clone());
        let file = File::open(&outcome.data_path).map_err(|err| {
            CliError::io(format!(
                "failed to open {}: {err}",
                outcome.data_path.display()
            ))
        })?;
        let reader = BufReader::new(file);
        let summary = dfps_eval::run_eval_streaming_with_mapper(
            reader,
            |rows| map_staging_codes(rows).0,
            chunk_size,
        )
        .map_err(|err| CliError::external(format!("eval error: {err}")))?;
        (summary, name.clone())
    } else if let Some(path) = &args.input {
        let file = File::open(path)
            .map_err(|err| CliError::io(format!("failed to open {}: {err}", path.display())))?;
        let reader = BufReader::new(file);
        let summary = dfps_eval::run_eval_streaming_with_mapper(
            reader,
            |rows| map_staging_codes(rows).0,
            chunk_size,
        )
        .map_err(|err| CliError::external(format!("eval error: {err}")))?;
        (summary, path.display().to_string())
    } else {
        return Err(CliError::invalid(
            "either --input or --dataset must be provided",
        ));
    };

    let dataset_name = if raw_dataset.is_empty() {
        "adhoc_eval".into()
    } else {
        raw_dataset
    };
    let response = EvalRunResponse {
        dataset: dataset_name,
        manifest,
        summary: summary.clone(),
    };

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    write_record(&mut handle, "eval_summary", &response)?;

    if args.dump_details {
        for result in &summary.results {
            write_record(&mut handle, "eval_result", result)?;
        }
    }

    if let Some(base_dir) = &args.out_dir {
        persist_artifacts(base_dir, &args, &response, &summary.results)?;
    }

    if let Some(report_path) = &args.report {
        write_report(
            report_path,
            &summary,
            args.dataset.as_deref(),
            &dataset_store,
        )?;
    }

    if args.top_k > 1 && summary.top3_accuracy == summary.top1_accuracy {
        eprintln!(
            "note: --top-k >1 requested, but current engine exposes only top-1; top3 mirrors top1"
        );
    }

    if let Some(path) = &args.thresholds {
        let cfg: ThresholdConfig = parse_json_file(path)?;
        enforce_thresholds(&summary, &cfg).map_err(|msg| {
            CliError::invalid(format!("{msg} (thresholds file: {})", path.display()))
        })?;
    }

    if let Some(path) = &args.compare_to {
        let baseline_str = read_to_string(path.as_path())?;
        let baseline: dfps_eval::report::BaselineSnapshot = serde_json::from_str(&baseline_str)
            .map_err(|err| {
                CliError::invalid(format!(
                    "failed to parse baseline {}: {err}",
                    path.display()
                ))
            })?;
        ensure_not_regressed(&summary, &baseline.summary).map_err(|msg| {
            CliError::invalid(format!("regression vs baseline {}: {msg}", path.display()))
        })?;
    }

    if let Some(path) = &args.deterministic {
        let fingerprint = dfps_eval::fingerprint_summary(&summary);
        if path.exists() {
            let baseline = read_to_string(path.as_path())?;
            let baseline = baseline.trim();
            if baseline != fingerprint {
                return Err(CliError::invalid(format!(
                    "deterministic check failed: baseline {} vs current {}",
                    baseline, fingerprint
                )));
            }
        } else {
            std::fs::write(path, format!("{fingerprint}\n"))
                .map_err(|err| CliError::io(format!("failed to write fingerprint: {err}")))?;
            eprintln!(
                "deterministic baseline written to {}; rerun to verify stability",
                path.display()
            );
        }
    }

    Ok(())
}

fn persist_artifacts(
    base_dir: &Path,
    args: &Args,
    response: &EvalRunResponse,
    results: &[dfps_eval::EvalResult],
) -> CliResult<()> {
    create_dir_all(base_dir)
        .map_err(|err| CliError::io(format!("failed to create {}: {err}", base_dir.display())))?;
    let summary_path = base_dir.join("eval_summary.json");
    let mut file = File::create(&summary_path).map_err(|err| {
        CliError::io(format!(
            "failed to create {}: {err}",
            summary_path.display()
        ))
    })?;
    serde_json::to_writer_pretty(&mut file, response)?;

    if args.dump_details {
        let results_path = base_dir.join("eval_results.ndjson");
        let mut file = File::create(&results_path).map_err(|err| {
            CliError::io(format!(
                "failed to create {}: {err}",
                results_path.display()
            ))
        })?;
        for result in results {
            serde_json::to_writer(&mut file, result)?;
            file.write_all(b"\n").map_err(CliError::from)?;
        }
    }

    Ok(())
}

fn write_report(
    report_path: &Path,
    summary: &EvalSummary,
    dataset: Option<&str>,
    store: &dfps_eval::FileDatasetStore,
) -> CliResult<()> {
    let baseline = dataset
        .and_then(|name| dfps_eval::report::load_baseline_snapshot_from(store.root(), name).ok());
    let html = dfps_eval::report::render_html(summary, baseline.as_ref().map(|snap| &snap.summary));
    let mut file = File::create(report_path).map_err(|err| {
        CliError::io(format!("failed to create {}: {err}", report_path.display()))
    })?;
    file.write_all(html.as_bytes())
        .map_err(|err| CliError::io(format!("failed to write {}: {err}", report_path.display())))?;
    Ok(())
}

fn enforce_thresholds(
    summary: &dfps_eval::EvalSummary,
    cfg: &ThresholdConfig,
) -> Result<(), String> {
    if let Some(min) = cfg.min_precision {
        if summary.precision < min {
            return Err(format!(
                "precision {} fell below configured minimum {}",
                summary.precision, min
            ));
        }
    }
    if let Some(min) = cfg.min_recall {
        if summary.recall < min {
            return Err(format!(
                "recall {} fell below configured minimum {}",
                summary.recall, min
            ));
        }
    }
    if let Some(min) = cfg.min_f1 {
        if summary.f1 < min {
            return Err(format!(
                "F1 {} fell below configured minimum {}",
                summary.f1, min
            ));
        }
    }
    if let Some(min) = cfg.min_accuracy {
        if summary.top1_accuracy < min {
            return Err(format!(
                "accuracy {} fell below configured minimum {}",
                summary.top1_accuracy, min
            ));
        }
    }
    if let Some(min) = cfg.min_auto_precision {
        if summary.auto_mapped_precision < min {
            return Err(format!(
                "auto-mapped precision {} fell below configured minimum {}",
                summary.auto_mapped_precision, min
            ));
        }
    }
    if let Some(min) = cfg.min_coverage {
        if summary.coverage < min {
            return Err(format!(
                "coverage {} fell below configured minimum {}",
                summary.coverage, min
            ));
        }
    }
    if let Some(min) = cfg.min_top1 {
        if summary.top1_accuracy < min {
            return Err(format!(
                "top1 accuracy {} fell below configured minimum {}",
                summary.top1_accuracy, min
            ));
        }
    }
    if let Some(allowed) = &cfg.allow_no_match_reason {
        for (reason, _) in &summary.reason_counts {
            if !allowed.contains(reason) {
                return Err(format!(
                    "NoMatch reason '{reason}' not permitted by thresholds"
                ));
            }
        }
    }
    Ok(())
}

fn ensure_not_regressed(
    summary: &dfps_eval::EvalSummary,
    baseline: &dfps_eval::EvalSummary,
) -> Result<(), String> {
    if summary.top1_accuracy + f32::EPSILON < baseline.top1_accuracy {
        return Err(format!(
            "top1 accuracy {} regressed vs baseline {}",
            summary.top1_accuracy, baseline.top1_accuracy
        ));
    }
    if summary.precision + f32::EPSILON < baseline.precision {
        return Err(format!(
            "precision {} regressed vs baseline {}",
            summary.precision, baseline.precision
        ));
    }
    if summary.recall + f32::EPSILON < baseline.recall {
        return Err(format!(
            "recall {} regressed vs baseline {}",
            summary.recall, baseline.recall
        ));
    }
    Ok(())
}
