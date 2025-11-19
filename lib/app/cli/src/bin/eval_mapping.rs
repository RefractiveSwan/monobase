use std::env;
use std::fs::{File, create_dir_all};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};

use clap::Parser;
use dfps_configuration::load_env;
use dfps_eval::{self, AdvancedStats, StratifiedMetrics};
use dfps_mapping::map_staging_codes;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize)]
struct SummaryView<'a> {
    total_cases: usize,
    predicted_cases: usize,
    correct: usize,
    incorrect: usize,
    precision: f32,
    recall: f32,
    f1: f32,
    accuracy: f32,
    coverage: f32,
    top1_accuracy: f32,
    top3_accuracy: f32,
    auto_mapped_total: usize,
    auto_mapped_correct: usize,
    auto_mapped_precision: f32,
    system_confusion: &'a std::collections::BTreeMap<String, dfps_eval::SystemConfusion>,
    #[serde(rename = "state_counts")]
    states: &'a std::collections::BTreeMap<String, usize>,
    by_system: &'a std::collections::BTreeMap<String, StratifiedMetrics>,
    #[serde(rename = "by_license_tier")]
    by_license: &'a std::collections::BTreeMap<String, StratifiedMetrics>,
    #[serde(rename = "score_buckets")]
    buckets: &'a [dfps_eval::ScoreBucket],
    #[serde(rename = "reason_counts")]
    reasons: &'a std::collections::BTreeMap<String, usize>,
    advanced: &'a Option<AdvancedStats>,
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
    let summary = if let Some(name) = &args.dataset {
        let outcome = dataset_store
            .load_dataset_with_manifest(name)
            .map_err(|err| format!("failed to load dataset {name}: {err}"))?;
        if !outcome.checksum_ok {
            eprintln!(
                "warning: dataset {name} checksum mismatch (expected {}, actual {})",
                outcome.manifest.sha256, outcome.computed_sha256
            );
        }
        let file = File::open(&outcome.data_path)?;
        let reader = BufReader::new(file);
        dfps_eval::run_eval_streaming_with_mapper(
            reader,
            |rows| map_staging_codes(rows).0,
            chunk_size,
        )?
    } else if let Some(path) = &args.input {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        dfps_eval::run_eval_streaming_with_mapper(
            reader,
            |rows| map_staging_codes(rows).0,
            chunk_size,
        )?
    } else {
        return Err("either --input or --dataset must be provided".into());
    };
    let summary_view = SummaryView {
        total_cases: summary.total_cases,
        predicted_cases: summary.predicted_cases,
        correct: summary.correct,
        incorrect: summary.incorrect,
        precision: summary.precision,
        recall: summary.recall,
        f1: summary.f1,
        accuracy: summary.accuracy,
        coverage: summary.coverage,
        top1_accuracy: summary.top1_accuracy,
        top3_accuracy: summary.top3_accuracy,
        auto_mapped_total: summary.auto_mapped_total,
        auto_mapped_correct: summary.auto_mapped_correct,
        auto_mapped_precision: summary.auto_mapped_precision,
        system_confusion: &summary.system_confusion,
        states: &summary.state_counts,
        by_system: &summary.by_system,
        by_license: &summary.by_license_tier,
        buckets: &summary.score_buckets,
        reasons: &summary.reason_counts,
        advanced: &summary.advanced,
    };

    println!(
        "{}",
        serde_json::to_string(&serde_json::json!({
            "kind": "eval_summary",
            "value": summary_view
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
        persist_artifacts(base_dir, &args, &summary_view, &summary.results)?;
    }

    if let Some(report_path) = &args.report {
        write_report(
            report_path,
            &summary_view,
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
    summary_view: &SummaryView,
    results: &[dfps_eval::EvalResult],
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = resolve_out_dir(base_dir, args);
    create_dir_all(&dir)?;
    let summary_payload = serde_json::json!({
        "dataset": args.dataset,
        "input": args.input.as_ref().map(|p| p.display().to_string()),
        "summary": summary_view
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
    summary: &SummaryView,
    dataset: Option<&str>,
    store: &dfps_eval::FileDatasetStore,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut summary_owned = dfps_eval::EvalSummary::default();
    summary_owned.total_cases = summary.total_cases;
    summary_owned.predicted_cases = summary.predicted_cases;
    summary_owned.correct = summary.correct;
    summary_owned.incorrect = summary.incorrect;
    summary_owned.precision = summary.precision;
    summary_owned.recall = summary.recall;
    summary_owned.f1 = summary.f1;
    summary_owned.accuracy = summary.accuracy;
    summary_owned.auto_mapped_total = summary.auto_mapped_total;
    summary_owned.auto_mapped_correct = summary.auto_mapped_correct;
    summary_owned.auto_mapped_precision = summary.auto_mapped_precision;
    summary_owned.score_buckets = summary.buckets.to_vec();
    summary_owned.reason_counts = summary.reasons.clone();
    summary_owned.advanced = summary.advanced.clone();
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
        &summary_owned,
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
