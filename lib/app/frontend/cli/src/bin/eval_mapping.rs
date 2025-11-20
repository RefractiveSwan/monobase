use std::env;
use std::fs::{File, create_dir_all};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};

use clap::Parser;
use dfps_configuration::load_env;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_env("app.cli").map_err(|err| format!("dfps_cli env error: {err}"))?;
    let args = Args::parse();
    let dataset_store = dataset_store_from_env();

    let chunk_size = args.chunk_size as usize;
    let mut manifest: Option<DatasetManifest> = None;
    let (summary, dataset_name): (EvalSummary, String) = if let Some(name) = &args.dataset {
        let outcome = dataset_store
            .load_dataset_with_manifest(name)
            .map_err(|err| format!("failed to load dataset {name}: {err}"))?;
        if !outcome.checksum_ok {
            eprintln!(
                "warning: dataset {name} checksum mismatch (expected {}, actual {})",
                outcome.manifest.sha256, outcome.computed_sha256
            );
        }
        manifest = Some(outcome.manifest.clone());
        let file = File::open(&outcome.data_path)?;
        let reader = BufReader::new(file);
        let summary = dfps_eval::run_eval_streaming_with_mapper(
            reader,
            |rows| map_staging_codes(rows).0,
            chunk_size,
        )?;
        (summary, name.clone())
    } else if let Some(path) = &args.input {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let summary = dfps_eval::run_eval_streaming_with_mapper(
            reader,
            |rows| map_staging_codes(rows).0,
            chunk_size,
        )?;
        (summary, path.display().to_string())
    } else {
        return Err("either --input or --dataset must be provided".into());
    };
    let dataset_name = if dataset_name.is_empty() {
        "adhoc_eval".into()
    } else {
        dataset_name
    };
    let response = EvalRunResponse {
        dataset: dataset_name.clone(),
        manifest,
        summary: summary.clone(),
    };

    println!(
        "{}",
        serde_json::to_string(&serde_json::json!({
            "kind": "eval_summary",
            "value": &response
        }))?
    );

    if args.dump_details {
        for result in &summary.results {
            println!(
                "{}",
                serde_json::to_string(&serde_json::json!({
                    "kind": "eval_result",
                    "value": result
                }))?
            );
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
        let file = File::open(path)?;
        let cfg: ThresholdConfig = serde_json::from_reader(file)?;
        enforce_thresholds(&summary, &cfg)
            .map_err(|msg| format!("{msg} (thresholds file: {})", path.display()))?;
    }

    if let Some(path) = &args.compare_to {
        let baseline = std::fs::read_to_string(path)?;
        let baseline: dfps_eval::report::BaselineSnapshot = serde_json::from_str(&baseline)
            .map_err(|err| format!("failed to parse baseline {}: {err}", path.display()))?;
        ensure_not_regressed(&summary, &baseline.summary)
            .map_err(|msg| format!("regression vs baseline {}: {msg}", path.display()))?;
    }

    if let Some(path) = &args.deterministic {
        let fingerprint = dfps_eval::fingerprint_summary(&summary);
        if path.exists() {
            let baseline = std::fs::read_to_string(path)?;
            let baseline = baseline.trim();
            if baseline != fingerprint {
                return Err(format!(
                    "deterministic check failed: baseline {} vs current {}",
                    baseline, fingerprint
                )
                .into());
            }
        } else {
            std::fs::write(path, format!("{fingerprint}\n"))?;
            eprintln!(
                "deterministic baseline written to {}; rerun to verify stability",
                path.display()
            );
        }
    }

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
                "f1 {} fell below configured minimum {}",
                summary.f1, min
            ));
        }
    }
    if let Some(min) = cfg.min_accuracy {
        if summary.accuracy < min {
            return Err(format!(
                "accuracy {} fell below configured minimum {}",
                summary.accuracy, min
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
    if let Some(min) = cfg.min_auto_precision {
        if summary.auto_mapped_precision < min {
            return Err(format!(
                "auto-mapped precision {} fell below configured minimum {}",
                summary.auto_mapped_precision, min
            ));
        }
    }
    if let Some(min) = cfg.min_top1 {
        if summary.top1_accuracy < min {
            return Err(format!(
                "top-1 accuracy {} fell below configured minimum {}",
                summary.top1_accuracy, min
            ));
        }
    }
    if let Some(allowed) = &cfg.allow_no_match_reason {
        let allowed: std::collections::HashSet<_> = allowed.iter().map(|s| s.as_str()).collect();
        for result in &summary.results {
            if result.mapping.state == dfps_core::mapping::MappingState::NoMatch {
                let reason = result
                    .mapping
                    .reason
                    .as_deref()
                    .unwrap_or("missing_system_or_code");
                if !allowed.contains(reason) {
                    return Err(format!(
                        "no_match reason '{reason}' not in allowlist {:?}",
                        allowed
                    ));
                }
            }
        }
    }
    Ok(())
}

fn ensure_not_regressed(
    current: &dfps_eval::EvalSummary,
    baseline: &dfps_eval::EvalSummary,
) -> Result<(), String> {
    if current.precision < baseline.precision {
        return Err(format!(
            "precision {:.3} < baseline {:.3}",
            current.precision, baseline.precision
        ));
    }
    if current.recall < baseline.recall {
        return Err(format!(
            "recall {:.3} < baseline {:.3}",
            current.recall, baseline.recall
        ));
    }
    if current.f1 < baseline.f1 {
        return Err(format!(
            "f1 {:.3} < baseline {:.3}",
            current.f1, baseline.f1
        ));
    }
    if current.accuracy < baseline.accuracy {
        return Err(format!(
            "accuracy {:.3} < baseline {:.3}",
            current.accuracy, baseline.accuracy
        ));
    }
    if current.auto_mapped_precision < baseline.auto_mapped_precision {
        return Err(format!(
            "auto-mapped precision {:.3} < baseline {:.3}",
            current.auto_mapped_precision, baseline.auto_mapped_precision
        ));
    }
    Ok(())
}

fn persist_artifacts(
    base_dir: &Path,
    args: &Args,
    response: &EvalRunResponse,
    results: &[dfps_eval::EvalResult],
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = resolve_out_dir(base_dir, args);
    create_dir_all(&dir)?;
    let summary_payload = serde_json::json!({
        "dataset": &response.dataset,
        "input": args.input.as_ref().map(|p| p.display().to_string()),
        "manifest": &response.manifest,
        "summary": &response.summary
    });

    let mut summary_file = File::create(dir.join("eval_summary.json"))?;
    serde_json::to_writer_pretty(&mut summary_file, &summary_payload)?;
    summary_file.write_all(b"\n")?;

    let mut results_file = File::create(dir.join("eval_results.ndjson"))?;
    for result in results {
        serde_json::to_writer(&mut results_file, result)?;
        results_file.write_all(b"\n")?;
    }
    Ok(())
}

fn resolve_out_dir(base: &Path, args: &Args) -> PathBuf {
    let mut dir = base.to_path_buf();
    if let Some(name) = &args.dataset {
        dir = dir.join(name);
    } else if let Some(input) = &args.input {
        if let Some(stem) = input.file_stem() {
            dir = dir.join(stem);
        }
    }
    dir
}

fn write_report(
    path: &Path,
    summary: &EvalSummary,
    dataset: Option<&str>,
    store: &dfps_eval::FileDatasetStore,
) -> Result<(), Box<dyn std::error::Error>> {
    let baseline = dataset.and_then(|name| {
        match dfps_eval::report::load_baseline_snapshot_from(store.root(), name) {
            Ok(snapshot) => Some(snapshot),
            Err(err) => {
                eprintln!("warning: could not load baseline for {name}: {err}");
                None
            }
        }
    });
    let markdown = dfps_eval::report::render_markdown_with_baseline(
        summary,
        baseline.as_ref().map(|snap| &snap.summary),
    );
    std::fs::write(path, markdown)?;
    Ok(())
}

fn dataset_store_from_env() -> dfps_eval::FileDatasetStore {
    let root = env::var("DFPS_EVAL_DATA_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dfps_eval::default_data_root());
    dfps_eval::FileDatasetStore::new(root)
}
