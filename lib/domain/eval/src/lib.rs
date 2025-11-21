//! Evaluation types and dataset helpers for NCIt mapping harnesses.
//!
//! See:
//! - docs/system-design/clinical/ncit/architecture.md
//! - docs/runbook/030-mapping-and-terminology/mapping-eval-quickstart.md
//! - docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md (REFR-07)

pub mod config;
pub mod fake_data;

use dfps_core::{
    mapping::{MappingResult, MappingState},
    staging::StgSrCodeExploded,
};
#[cfg(feature = "eval-advanced")]
use rand::{Rng, SeedableRng, rngs::StdRng};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};
use thiserror::Error;

pub const DEFAULT_DATA_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/eval");

pub mod io;
pub mod report;
pub use config::{EvalConfigError, EvalDatasetConfig};

pub const DEFAULT_CHUNK_SIZE: usize = 1_000;

/// Default on-disk dataset root bundled with the workspace.
pub fn default_data_root() -> PathBuf {
    PathBuf::from(DEFAULT_DATA_ROOT)
}

/// File-backed dataset/catalog implementation.
#[derive(Debug, Clone)]
pub struct FileDatasetStore {
    root: PathBuf,
}

impl Default for FileDatasetStore {
    fn default() -> Self {
        EvalDatasetConfig::from_env()
            .map(|cfg| cfg.dataset_store())
            .unwrap_or_else(|_| FileDatasetStore::new(default_data_root()))
    }
}

impl FileDatasetStore {
    /// Create a store rooted at the provided directory.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Build a store from environment-driven configuration.
    pub fn from_env() -> Result<Self, EvalConfigError> {
        EvalDatasetConfig::from_env().map(|cfg| cfg.dataset_store())
    }

    /// Absolute path to the root directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Path to the NDJSON for a dataset.
    pub fn dataset_path(&self, name: &str) -> PathBuf {
        self.root.join(format!("{name}.ndjson"))
    }

    /// Path to the manifest JSON for a dataset.
    pub fn manifest_path(&self, name: &str) -> PathBuf {
        self.root.join(format!("{name}.manifest.json"))
    }

    /// Path to the baseline snapshot JSON for a dataset.
    pub fn baseline_path(&self, dataset: &str) -> PathBuf {
        self.root.join(format!("{dataset}.baseline.json"))
    }

    /// Open a buffered reader over the NDJSON rows for a dataset.
    pub fn open_dataset_reader(&self, name: &str) -> Result<BufReader<File>, DatasetError> {
        let path = self.dataset_path(name);
        let file = File::open(&path).map_err(|source| DatasetError::Io {
            source,
            path: path.clone(),
        })?;
        Ok(BufReader::new(file))
    }

    /// Load a dataset alongside its manifest/metadata.
    pub fn load_dataset_with_manifest(
        &self,
        name: &str,
    ) -> Result<DatasetLoadOutcome, DatasetError> {
        let manifest = load_manifest_from_path(&self.manifest_path(name))?;
        let path = self.dataset_path(&manifest.name);
        let cases = load_cases_from_path(&path)?;
        let computed_sha = compute_sha256(&path)?;
        if manifest.n_cases != cases.len() {
            eprintln!(
                "warning: dataset {} manifest n_cases={} but file contains {} rows",
                manifest.name,
                manifest.n_cases,
                cases.len()
            );
        }
        if manifest
            .license
            .as_deref()
            .map(|value| value.trim().is_empty())
            .unwrap_or(true)
        {
            eprintln!(
                "warning: dataset {} manifest missing license attribution",
                manifest.name
            );
        }
        let checksum_ok = manifest.sha256.eq_ignore_ascii_case(computed_sha.as_str());

        Ok(DatasetLoadOutcome {
            manifest,
            data_path: path,
            cases,
            checksum_ok,
            computed_sha256: computed_sha,
        })
    }

    /// Load a dataset, discarding the manifest metadata.
    pub fn load_dataset(&self, name: &str) -> Result<Vec<EvalCase>, DatasetError> {
        self.load_dataset_with_manifest(name)
            .map(|outcome| outcome.cases)
    }

    /// Enumerate manifests under the root directory.
    pub fn list_manifests(&self) -> Result<Vec<DatasetManifest>, DatasetError> {
        let mut manifests = Vec::new();
        let entries = std::fs::read_dir(&self.root).map_err(|source| DatasetError::Io {
            source,
            path: self.root.clone(),
        })?;
        for entry in entries {
            let entry = entry.map_err(|source| DatasetError::Io {
                source,
                path: self.root.clone(),
            })?;
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|f| f.to_str())
                && name.ends_with(".manifest.json")
            {
                let file = File::open(&path).map_err(|source| DatasetError::Io {
                    source,
                    path: path.clone(),
                })?;
                let manifest: DatasetManifest =
                    serde_json::from_reader(file).map_err(|source| {
                        DatasetError::ManifestParse {
                            path: path.clone(),
                            source,
                        }
                    })?;
                manifests.push(manifest);
            }
        }
        manifests.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(manifests)
    }

    /// Load a baseline snapshot when one lives under the same root.
    pub fn load_baseline_snapshot(
        &self,
        dataset: &str,
    ) -> Result<crate::report::BaselineSnapshot, crate::report::BaselineError> {
        crate::report::load_baseline_snapshot_from(self.root(), dataset)
    }
}

#[cfg(all(test, feature = "eval-advanced"))]
mod advanced_tests {
    use super::*;
    use dfps_core::mapping::{MappingSourceVersion, MappingStrategy, MappingThresholds};

    #[test]
    fn advanced_stats_are_populated_when_feature_enabled() {
        let cases = vec![EvalCase {
            system: "http://www.ama-assn.org/go/cpt".into(),
            code: "78815".into(),
            display: "PET with concurrently acquired CT for tumor imaging".into(),
            expected_ncit_id: "NCIT:C19951".into(),
        }];

        let summary = run_eval_with_mapper(&cases, |rows| {
            rows.into_iter()
                .map(|row| MappingResult {
                    code_element_id: row.sr_id,
                    ncit_id: Some("NCIT:C19951".into()),
                    cui: Some("C19951".into()),
                    score: 0.95,
                    strategy: MappingStrategy::Lexical,
                    state: MappingState::AutoMapped,
                    thresholds: MappingThresholds::default(),
                    source_version: MappingSourceVersion::new("v-test", "v-test"),
                    reason: None,
                    license_tier: Some("licensed".into()),
                    source_kind: Some("fhir".into()),
                })
                .collect()
        });

        assert!(summary.advanced.is_some());
    }
}

#[derive(Debug, Error)]
pub enum DatasetError {
    #[error("failed to read dataset {path}: {source}")]
    Io {
        source: std::io::Error,
        path: PathBuf,
    },
    #[error("failed to parse EvalCase on line {line}: {source}")]
    Parse {
        line: usize,
        source: serde_json::Error,
    },
    #[error("failed to parse dataset manifest {path}: {source}")]
    ManifestParse {
        path: PathBuf,
        source: serde_json::Error,
    },
}

impl DatasetError {
    pub fn context(&self) -> String {
        match self {
            DatasetError::Io { path, .. } => format!("dataset {}", path.display()),
            DatasetError::Parse { line, .. } => format!("line {}", line),
            DatasetError::ManifestParse { path, .. } => format!("manifest {}", path.display()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DatasetManifest {
    pub name: String,
    pub version: String,
    pub license: Option<String>,
    pub source: Option<String>,
    pub n_cases: usize,
    pub sha256: String,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug)]
pub struct DatasetLoadOutcome {
    pub manifest: DatasetManifest,
    pub data_path: PathBuf,
    pub cases: Vec<EvalCase>,
    pub checksum_ok: bool,
    pub computed_sha256: String,
}

/// Trait abstraction so callers can swap in alternative dataset providers.
pub trait DatasetStore: Send + Sync {
    fn data_root(&self) -> &Path;
    fn load_dataset_with_manifest(&self, name: &str) -> Result<DatasetLoadOutcome, DatasetError>;
    fn load_dataset(&self, name: &str) -> Result<Vec<EvalCase>, DatasetError>;
    fn list_manifests(&self) -> Result<Vec<DatasetManifest>, DatasetError>;
}

impl DatasetStore for FileDatasetStore {
    fn data_root(&self) -> &Path {
        self.root()
    }

    fn load_dataset_with_manifest(&self, name: &str) -> Result<DatasetLoadOutcome, DatasetError> {
        FileDatasetStore::load_dataset_with_manifest(self, name)
    }

    fn load_dataset(&self, name: &str) -> Result<Vec<EvalCase>, DatasetError> {
        FileDatasetStore::load_dataset(self, name)
    }

    fn list_manifests(&self) -> Result<Vec<DatasetManifest>, DatasetError> {
        FileDatasetStore::list_manifests(self)
    }
}

pub fn load_dataset_with_manifest(name: &str) -> Result<DatasetLoadOutcome, DatasetError> {
    FileDatasetStore::default().load_dataset_with_manifest(name)
}

pub fn load_dataset(name: &str) -> Result<Vec<EvalCase>, DatasetError> {
    FileDatasetStore::default().load_dataset(name)
}

pub fn list_manifests() -> Result<Vec<DatasetManifest>, DatasetError> {
    FileDatasetStore::default().list_manifests()
}

fn load_manifest_from_path(path: &Path) -> Result<DatasetManifest, DatasetError> {
    let file = File::open(path).map_err(|source| DatasetError::Io {
        source,
        path: path.to_path_buf(),
    })?;
    serde_json::from_reader(file).map_err(|source| DatasetError::ManifestParse {
        path: path.to_path_buf(),
        source,
    })
}

fn compute_sha256(path: &Path) -> Result<String, DatasetError> {
    let mut file = File::open(path).map_err(|source| DatasetError::Io {
        source,
        path: path.to_path_buf(),
    })?;
    let mut hasher = Sha256::new();
    use std::io::Read;
    let mut buffer = [0u8; 8192];
    loop {
        let read = file.read(&mut buffer).map_err(|source| DatasetError::Io {
            source,
            path: path.to_path_buf(),
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn load_cases_from_path(path: impl AsRef<Path>) -> Result<Vec<EvalCase>, DatasetError> {
    let path = path.as_ref().to_path_buf();
    let file = File::open(&path).map_err(|source| DatasetError::Io {
        source,
        path: path.clone(),
    })?;
    load_cases_from_reader(BufReader::new(file))
}

pub fn load_cases_from_reader<R: BufRead>(reader: R) -> Result<Vec<EvalCase>, DatasetError> {
    let mut cases = Vec::new();
    let mut stream = crate::io::EvalCaseStream::new(reader);
    while let Some(case) = stream.next_case()? {
        cases.push(case);
    }
    Ok(cases)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EvalCase {
    pub system: String,
    pub code: String,
    pub display: String,
    pub expected_ncit_id: String,
}

impl EvalCase {
    pub fn to_staging_row(&self, sr_id: String) -> StgSrCodeExploded {
        StgSrCodeExploded {
            sr_id,
            system: Some(self.system.clone()),
            code: Some(self.code.clone()),
            display: Some(self.display.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EvalResult {
    pub case: EvalCase,
    pub mapping: dfps_core::mapping::MappingResult,
    pub correct: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvalSummary {
    pub total_cases: usize,
    pub predicted_cases: usize,
    pub correct: usize,
    pub incorrect: usize,
    pub precision: f32,
    pub recall: f32,
    pub f1: f32,
    pub accuracy: f32,
    pub coverage: f32,
    pub top1_accuracy: f32,
    pub top3_accuracy: f32,
    pub auto_mapped_total: usize,
    pub auto_mapped_correct: usize,
    pub auto_mapped_precision: f32,
    pub state_counts: BTreeMap<String, usize>,
    pub by_system: BTreeMap<String, StratifiedMetrics>,
    pub by_license_tier: BTreeMap<String, StratifiedMetrics>,
    pub score_buckets: Vec<ScoreBucket>,
    pub reason_counts: BTreeMap<String, usize>,
    pub system_confusion: BTreeMap<String, SystemConfusion>,
    pub advanced: Option<AdvancedStats>,
    pub results: Vec<EvalResult>,
}

/// Metrics-only view of an evaluation summary (no `EvalResult` payloads).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvalMetrics {
    pub total_cases: usize,
    pub predicted_cases: usize,
    pub correct: usize,
    pub incorrect: usize,
    pub precision: f32,
    pub recall: f32,
    pub f1: f32,
    pub accuracy: f32,
    pub coverage: f32,
    pub top1_accuracy: f32,
    pub top3_accuracy: f32,
    pub auto_mapped_total: usize,
    pub auto_mapped_correct: usize,
    pub auto_mapped_precision: f32,
    pub state_counts: BTreeMap<String, usize>,
    pub by_system: BTreeMap<String, StratifiedMetrics>,
    pub by_license_tier: BTreeMap<String, StratifiedMetrics>,
    pub score_buckets: Vec<ScoreBucket>,
    pub reason_counts: BTreeMap<String, usize>,
    pub system_confusion: BTreeMap<String, SystemConfusion>,
    pub advanced: Option<AdvancedStats>,
}

impl Default for EvalSummary {
    fn default() -> Self {
        Self {
            total_cases: 0,
            predicted_cases: 0,
            correct: 0,
            incorrect: 0,
            precision: 0.0,
            recall: 0.0,
            f1: 0.0,
            accuracy: 0.0,
            coverage: 0.0,
            top1_accuracy: 0.0,
            top3_accuracy: 0.0,
            auto_mapped_total: 0,
            auto_mapped_correct: 0,
            auto_mapped_precision: 0.0,
            state_counts: BTreeMap::new(),
            by_system: BTreeMap::new(),
            by_license_tier: BTreeMap::new(),
            score_buckets: Vec::new(),
            reason_counts: BTreeMap::new(),
            system_confusion: BTreeMap::new(),
            advanced: None,
            results: Vec::new(),
        }
    }
}

impl EvalSummary {
    fn from_metrics(metrics: EvalMetrics, results: Vec<EvalResult>) -> Self {
        Self {
            total_cases: metrics.total_cases,
            predicted_cases: metrics.predicted_cases,
            correct: metrics.correct,
            incorrect: metrics.incorrect,
            precision: metrics.precision,
            recall: metrics.recall,
            f1: metrics.f1,
            accuracy: metrics.accuracy,
            coverage: metrics.coverage,
            top1_accuracy: metrics.top1_accuracy,
            top3_accuracy: metrics.top3_accuracy,
            auto_mapped_total: metrics.auto_mapped_total,
            auto_mapped_correct: metrics.auto_mapped_correct,
            auto_mapped_precision: metrics.auto_mapped_precision,
            state_counts: metrics.state_counts,
            by_system: metrics.by_system,
            by_license_tier: metrics.by_license_tier,
            score_buckets: metrics.score_buckets,
            reason_counts: metrics.reason_counts,
            system_confusion: metrics.system_confusion,
            advanced: metrics.advanced,
            results,
        }
    }

    fn apply_metrics(&mut self, metrics: &EvalMetrics) {
        self.total_cases = metrics.total_cases;
        self.predicted_cases = metrics.predicted_cases;
        self.correct = metrics.correct;
        self.incorrect = metrics.incorrect;
        self.precision = metrics.precision;
        self.recall = metrics.recall;
        self.f1 = metrics.f1;
        self.accuracy = metrics.accuracy;
        self.coverage = metrics.coverage;
        self.top1_accuracy = metrics.top1_accuracy;
        self.top3_accuracy = metrics.top3_accuracy;
        self.auto_mapped_total = metrics.auto_mapped_total;
        self.auto_mapped_correct = metrics.auto_mapped_correct;
        self.auto_mapped_precision = metrics.auto_mapped_precision;
        self.state_counts = metrics.state_counts.clone();
        self.by_system = metrics.by_system.clone();
        self.by_license_tier = metrics.by_license_tier.clone();
        self.score_buckets = metrics.score_buckets.clone();
        self.reason_counts = metrics.reason_counts.clone();
        self.system_confusion = metrics.system_confusion.clone();
        self.advanced = metrics.advanced.clone();
    }
}

impl Default for EvalMetrics {
    fn default() -> Self {
        Self {
            total_cases: 0,
            predicted_cases: 0,
            correct: 0,
            incorrect: 0,
            precision: 0.0,
            recall: 0.0,
            f1: 0.0,
            accuracy: 0.0,
            coverage: 0.0,
            top1_accuracy: 0.0,
            top3_accuracy: 0.0,
            auto_mapped_total: 0,
            auto_mapped_correct: 0,
            auto_mapped_precision: 0.0,
            state_counts: BTreeMap::new(),
            by_system: BTreeMap::new(),
            by_license_tier: BTreeMap::new(),
            score_buckets: Vec::new(),
            reason_counts: BTreeMap::new(),
            system_confusion: BTreeMap::new(),
            advanced: None,
        }
    }
}

impl From<&EvalSummary> for EvalMetrics {
    fn from(summary: &EvalSummary) -> Self {
        Self {
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
            state_counts: summary.state_counts.clone(),
            by_system: summary.by_system.clone(),
            by_license_tier: summary.by_license_tier.clone(),
            score_buckets: summary.score_buckets.clone(),
            reason_counts: summary.reason_counts.clone(),
            system_confusion: summary.system_confusion.clone(),
            advanced: summary.advanced.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScoreBucket {
    pub bucket: String,
    pub lower_bound: Option<f32>,
    pub upper_bound: Option<f32>,
    pub total: usize,
    pub correct: usize,
    pub accuracy: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AdvancedStats {
    pub precision_ci: (f32, f32),
    pub recall_ci: (f32, f32),
    pub f1_ci: (f32, f32),
    pub bootstrap_iterations: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StratifiedMetrics {
    pub total_cases: usize,
    pub predicted_cases: usize,
    pub correct: usize,
    pub precision: f32,
    pub recall: f32,
    pub f1: f32,
}

impl Default for StratifiedMetrics {
    fn default() -> Self {
        Self {
            total_cases: 0,
            predicted_cases: 0,
            correct: 0,
            precision: 0.0,
            recall: 0.0,
            f1: 0.0,
        }
    }
}

impl StratifiedMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, predicted: bool, correct: bool) {
        self.total_cases += 1;
        if predicted {
            self.predicted_cases += 1;
        }
        if correct {
            self.correct += 1;
        }
    }

    pub fn finalize(&mut self) {
        let (precision, recall, f1) =
            compute_metrics(self.correct, self.predicted_cases, self.total_cases);
        self.precision = precision;
        self.recall = recall;
        self.f1 = f1;
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SystemConfusion {
    pub total_cases: usize,
    pub predicted_cases: usize,
    pub correct: usize,
    pub auto_mapped: usize,
    pub needs_review: usize,
    pub no_match: usize,
    pub coverage: f32,
    pub accuracy: f32,
}

pub fn compute_metrics(correct: usize, predicted: usize, total: usize) -> (f32, f32, f32) {
    let precision = if predicted > 0 {
        correct as f32 / predicted as f32
    } else {
        0.0
    };
    let recall = if total > 0 {
        correct as f32 / total as f32
    } else {
        0.0
    };
    let f1 = if precision + recall > 0.0 {
        2.0 * precision * recall / (precision + recall)
    } else {
        0.0
    };
    (precision, recall, f1)
}

#[cfg(feature = "eval-advanced")]
pub fn bootstrap_metrics(samples: &[(bool, bool)], iterations: usize) -> AdvancedStats {
    let mut rng = StdRng::seed_from_u64(42 + samples.len() as u64);
    let mut precisions = Vec::with_capacity(iterations);
    let mut recalls = Vec::with_capacity(iterations);
    let mut f1s = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let mut correct = 0usize;
        let mut predicted = 0usize;
        for _ in 0..samples.len() {
            let &(pred, corr) = samples.get(rng.random_range(0..samples.len())).unwrap();
            if pred {
                predicted += 1;
            }
            if corr {
                correct += 1;
            }
        }
        let (precision, recall, f1) = compute_metrics(correct, predicted, samples.len());
        precisions.push(precision);
        recalls.push(recall);
        f1s.push(f1);
    }

    AdvancedStats {
        precision_ci: percentile_bounds(&mut precisions),
        recall_ci: percentile_bounds(&mut recalls),
        f1_ci: percentile_bounds(&mut f1s),
        bootstrap_iterations: iterations,
    }
}

#[cfg(feature = "eval-advanced")]
fn percentile_bounds(values: &mut [f32]) -> (f32, f32) {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if values.is_empty() {
        return (0.0, 0.0);
    }
    let lower_idx = ((values.len() as f32) * 0.05).floor() as usize;
    let upper_idx = ((values.len() as f32) * 0.95).ceil() as usize - 1;
    let lower = values.get(lower_idx).copied().unwrap_or(0.0);
    let upper = values
        .get(upper_idx.min(values.len() - 1))
        .copied()
        .unwrap_or(0.0);
    (lower, upper)
}

pub fn run_eval_with_mapper<F>(cases: &[EvalCase], mut mapper: F) -> EvalSummary
where
    F: FnMut(Vec<StgSrCodeExploded>) -> Vec<MappingResult>,
{
    if cases.is_empty() {
        return EvalSummary::default();
    }
    let mappings = invoke_mapper_with_offset(cases, &mut mapper, 0);
    assemble_summary(cases, mappings)
}

/// Stream an NDJSON dataset in chunks to keep memory bounded while producing a combined summary.
pub fn run_eval_streaming_with_mapper<R, F>(
    reader: R,
    mut mapper: F,
    chunk_size: usize,
) -> Result<EvalSummary, DatasetError>
where
    R: BufRead,
    F: FnMut(Vec<StgSrCodeExploded>) -> Vec<MappingResult>,
{
    let mut stream = crate::io::EvalCaseStream::new(reader);
    let mut aggregated = EvalSummary::default();
    let mut processed = 0usize;
    loop {
        let mut chunk = Vec::with_capacity(chunk_size);
        while chunk.len() < chunk_size {
            match stream.next_case()? {
                Some(case) => chunk.push(case),
                None => break,
            }
        }
        if chunk.is_empty() {
            break;
        }
        let mappings = invoke_mapper_with_offset(&chunk, &mut mapper, processed);
        processed += chunk.len();
        let summary = assemble_summary(&chunk, mappings);
        aggregate_summaries(&mut aggregated, summary);
    }
    Ok(aggregated)
}

/// Stream NDJSON datasets but only keep aggregate metrics (no `EvalResult` payloads).
pub fn run_eval_streaming_metrics_with_mapper<R, F>(
    reader: R,
    mut mapper: F,
    chunk_size: usize,
) -> Result<EvalMetrics, DatasetError>
where
    R: BufRead,
    F: FnMut(Vec<StgSrCodeExploded>) -> Vec<MappingResult>,
{
    let mut stream = crate::io::EvalCaseStream::new(reader);
    let mut aggregated = EvalMetrics::default();
    let mut processed = 0usize;
    loop {
        let mut chunk = Vec::with_capacity(chunk_size);
        while chunk.len() < chunk_size {
            match stream.next_case()? {
                Some(case) => chunk.push(case),
                None => break,
            }
        }
        if chunk.is_empty() {
            break;
        }
        let mappings = invoke_mapper_with_offset(&chunk, &mut mapper, processed);
        processed += chunk.len();
        let metrics = assemble_metrics(&chunk, mappings);
        aggregate_metrics(&mut aggregated, metrics);
    }
    Ok(aggregated)
}

fn invoke_mapper_with_offset<F>(
    cases: &[EvalCase],
    mapper: &mut F,
    start_index: usize,
) -> Vec<MappingResult>
where
    F: FnMut(Vec<StgSrCodeExploded>) -> Vec<MappingResult>,
{
    let staging_rows: Vec<_> = cases
        .iter()
        .enumerate()
        .map(|(idx, case)| {
            case.to_staging_row(format!("eval-{index:04}", index = start_index + idx))
        })
        .collect();
    mapper(staging_rows)
}

/// Merge a chunk summary into an aggregate summary (recomputes derived metrics).
pub fn aggregate_summaries(base: &mut EvalSummary, chunk: EvalSummary) {
    let mut base_metrics = EvalMetrics::from(&*base);
    let chunk_metrics = EvalMetrics::from(&chunk);
    aggregate_metrics(&mut base_metrics, chunk_metrics);
    base.apply_metrics(&base_metrics);
    base.results.extend(chunk.results);
}

pub fn aggregate_metrics(base: &mut EvalMetrics, chunk: EvalMetrics) {
    base.total_cases += chunk.total_cases;
    base.predicted_cases += chunk.predicted_cases;
    base.correct += chunk.correct;
    base.incorrect += chunk.incorrect;
    base.auto_mapped_total += chunk.auto_mapped_total;
    base.auto_mapped_correct += chunk.auto_mapped_correct;

    for (state, count) in chunk.state_counts {
        *base.state_counts.entry(state).or_default() += count;
    }
    for (reason, count) in chunk.reason_counts {
        *base.reason_counts.entry(reason).or_default() += count;
    }
    for (system, mut metrics) in chunk.by_system {
        let entry = base.by_system.entry(system).or_default();
        entry.total_cases += metrics.total_cases;
        entry.predicted_cases += metrics.predicted_cases;
        entry.correct += metrics.correct;
        metrics.finalize();
    }
    for (tier, mut metrics) in chunk.by_license_tier {
        let entry = base.by_license_tier.entry(tier).or_default();
        entry.total_cases += metrics.total_cases;
        entry.predicted_cases += metrics.predicted_cases;
        entry.correct += metrics.correct;
        metrics.finalize();
    }

    let mut bucket_map: BTreeMap<String, (usize, usize, Option<f32>, Option<f32>)> = base
        .score_buckets
        .iter()
        .map(|bucket| {
            (
                bucket.bucket.clone(),
                (
                    bucket.total,
                    bucket.correct,
                    bucket.lower_bound,
                    bucket.upper_bound,
                ),
            )
        })
        .collect();
    for bucket in chunk.score_buckets {
        let entry = bucket_map.entry(bucket.bucket.clone()).or_insert((
            0,
            0,
            bucket.lower_bound,
            bucket.upper_bound,
        ));
        entry.0 += bucket.total;
        entry.1 += bucket.correct;
    }
    base.score_buckets = bucket_map
        .into_iter()
        .map(|(bucket, (total, correct, lower, upper))| ScoreBucket {
            bucket,
            lower_bound: lower,
            upper_bound: upper,
            total,
            correct,
            accuracy: if total > 0 {
                correct as f32 / total as f32
            } else {
                0.0
            },
        })
        .collect();

    for (system, confusion) in chunk.system_confusion {
        let entry = base.system_confusion.entry(system).or_default();
        entry.total_cases += confusion.total_cases;
        entry.predicted_cases += confusion.predicted_cases;
        entry.correct += confusion.correct;
        entry.auto_mapped += confusion.auto_mapped;
        entry.needs_review += confusion.needs_review;
        entry.no_match += confusion.no_match;
    }
    base.system_confusion = finalize_confusion(base.system_confusion.clone());

    let (precision, recall, f1) =
        compute_metrics(base.correct, base.predicted_cases, base.total_cases);
    base.precision = precision;
    base.recall = recall;
    base.f1 = f1;
    base.accuracy = if base.total_cases > 0 {
        base.correct as f32 / base.total_cases as f32
    } else {
        0.0
    };
    base.coverage = if base.total_cases > 0 {
        base.predicted_cases as f32 / base.total_cases as f32
    } else {
        0.0
    };
    base.top1_accuracy = base.precision;
    base.top3_accuracy = base.precision;
    base.auto_mapped_precision = if base.auto_mapped_total > 0 {
        base.auto_mapped_correct as f32 / base.auto_mapped_total as f32
    } else {
        0.0
    };
    base.by_system = finalize_stratified(base.by_system.clone());
    base.by_license_tier = finalize_stratified(base.by_license_tier.clone());
    base.advanced = None;
}

fn compute_chunk_metrics(
    cases: &[EvalCase],
    mappings: Vec<MappingResult>,
    collect_results: bool,
) -> (EvalMetrics, Vec<EvalResult>) {
    let mut metrics = EvalMetrics {
        total_cases: cases.len(),
        ..EvalMetrics::default()
    };

    let mut by_system: BTreeMap<String, StratifiedMetrics> = BTreeMap::new();
    let mut by_license: BTreeMap<String, StratifiedMetrics> = BTreeMap::new();
    let mut system_confusion: BTreeMap<String, SystemConfusion> = BTreeMap::new();
    let mut score_bucket_map: BTreeMap<BucketKey, BucketTally> = BTreeMap::new();
    let mut reason_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut advanced_samples = Vec::with_capacity(cases.len());
    let mut auto_mapped_total = 0usize;
    let mut auto_mapped_correct = 0usize;
    let mut results = if collect_results {
        Some(Vec::with_capacity(cases.len()))
    } else {
        None
    };

    for (case, mapping) in cases.iter().cloned().zip(mappings.into_iter()) {
        let predicted = mapping.ncit_id.is_some();
        if predicted {
            metrics.predicted_cases += 1;
        }
        let is_correct = mapping
            .ncit_id
            .as_ref()
            .map(|ncit| ncit == &case.expected_ncit_id)
            .unwrap_or(false);
        if is_correct {
            metrics.correct += 1;
        }

        let label = state_label(mapping.state).to_string();
        *metrics.state_counts.entry(label).or_default() += 1;
        if matches!(mapping.state, MappingState::AutoMapped) && predicted {
            auto_mapped_total += 1;
            if is_correct {
                auto_mapped_correct += 1;
            }
        }

        record_stratified(&mut by_system, case.system.clone(), predicted, is_correct);
        record_confusion(
            &mut system_confusion,
            case.system.clone(),
            mapping.state,
            predicted,
            is_correct,
        );
        record_stratified(
            &mut by_license,
            mapping
                .license_tier
                .clone()
                .unwrap_or_else(|| "unknown".into()),
            predicted,
            is_correct,
        );

        if predicted {
            let bucket = bucket_key(mapping.score);
            let entry = score_bucket_map.entry(bucket).or_default();
            entry.total += 1;
            if is_correct {
                entry.correct += 1;
            }
        }

        let reason_key = mapping.reason.clone().unwrap_or_else(|| "none".to_string());
        *reason_counts.entry(reason_key).or_default() += 1;

        advanced_samples.push((predicted, is_correct));

        if let Some(results_vec) = results.as_mut() {
            results_vec.push(EvalResult {
                case,
                mapping,
                correct: is_correct,
            });
        }
    }

    metrics.incorrect = metrics.total_cases.saturating_sub(metrics.correct);
    let (precision, recall, f1) = compute_metrics(
        metrics.correct,
        metrics.predicted_cases,
        metrics.total_cases,
    );
    metrics.precision = precision;
    metrics.recall = recall;
    metrics.f1 = f1;
    metrics.accuracy = if metrics.total_cases > 0 {
        metrics.correct as f32 / metrics.total_cases as f32
    } else {
        0.0
    };
    metrics.coverage = if metrics.total_cases > 0 {
        metrics.predicted_cases as f32 / metrics.total_cases as f32
    } else {
        0.0
    };
    metrics.top1_accuracy = metrics.precision;
    metrics.top3_accuracy = metrics.precision;
    metrics.auto_mapped_total = auto_mapped_total;
    metrics.auto_mapped_correct = auto_mapped_correct;
    metrics.auto_mapped_precision = if auto_mapped_total > 0 {
        auto_mapped_correct as f32 / auto_mapped_total as f32
    } else {
        0.0
    };
    metrics.by_system = finalize_stratified(by_system);
    metrics.by_license_tier = finalize_stratified(by_license);
    metrics.score_buckets = finalize_score_buckets(score_bucket_map);
    metrics.reason_counts = reason_counts;
    metrics.system_confusion = finalize_confusion(system_confusion);
    #[cfg(feature = "eval-advanced")]
    {
        metrics.advanced = Some(crate::bootstrap_metrics(&advanced_samples, 100));
    }
    let result_vec = results.unwrap_or_default();
    (metrics, result_vec)
}
fn assemble_summary(cases: &[EvalCase], mappings: Vec<MappingResult>) -> EvalSummary {
    let (metrics, results) = compute_chunk_metrics(cases, mappings, true);
    EvalSummary::from_metrics(metrics, results)
}

fn assemble_metrics(cases: &[EvalCase], mappings: Vec<MappingResult>) -> EvalMetrics {
    compute_chunk_metrics(cases, mappings, false).0
}

/// Produce a deterministic fingerprint (sha256 hex) for an EvalSummary.
pub fn fingerprint_summary(summary: &EvalSummary) -> String {
    let mut hasher = Sha256::new();
    let bytes = serde_json::to_vec(summary).unwrap_or_default();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn state_label(state: MappingState) -> &'static str {
    match state {
        MappingState::AutoMapped => "auto_mapped",
        MappingState::NeedsReview => "needs_review",
        MappingState::NoMatch => "no_match",
    }
}

fn record_stratified(
    map: &mut BTreeMap<String, StratifiedMetrics>,
    key: String,
    predicted: bool,
    correct: bool,
) {
    map.entry(key).or_default().record(predicted, correct);
}

fn finalize_stratified(
    mut map: BTreeMap<String, StratifiedMetrics>,
) -> BTreeMap<String, StratifiedMetrics> {
    for metrics in map.values_mut() {
        metrics.finalize();
    }
    map
}

fn finalize_score_buckets(map: BTreeMap<BucketKey, BucketTally>) -> Vec<ScoreBucket> {
    map.into_iter()
        .map(|(key, tally)| {
            let (bucket, lower, upper) = match key {
                BucketKey::Nan => ("nan".to_string(), None, None),
                BucketKey::Range(idx) => {
                    let lower = (idx as f32) / 10.0;
                    let upper = ((idx + 1) as f32 / 10.0).min(1.0);
                    (format!("{lower:.1}–{upper:.1}"), Some(lower), Some(upper))
                }
            };
            ScoreBucket {
                bucket,
                lower_bound: lower,
                upper_bound: upper,
                total: tally.total,
                correct: tally.correct,
                accuracy: if tally.total > 0 {
                    tally.correct as f32 / tally.total as f32
                } else {
                    0.0
                },
            }
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum BucketKey {
    Range(u8),
    Nan,
}

#[derive(Debug, Default, Clone, Copy)]
struct BucketTally {
    total: usize,
    correct: usize,
}

fn bucket_key(score: f32) -> BucketKey {
    if !score.is_finite() {
        return BucketKey::Nan;
    }
    let normalized = score.clamp(0.0, 0.999);
    let idx = (normalized * 10.0).floor() as u8;
    BucketKey::Range(idx)
}

fn record_confusion(
    map: &mut BTreeMap<String, SystemConfusion>,
    system: String,
    state: MappingState,
    predicted: bool,
    correct: bool,
) {
    let entry = map.entry(system).or_default();
    entry.total_cases += 1;
    if predicted {
        entry.predicted_cases += 1;
    }
    if correct {
        entry.correct += 1;
    }
    match state {
        MappingState::AutoMapped => entry.auto_mapped += 1,
        MappingState::NeedsReview => entry.needs_review += 1,
        MappingState::NoMatch => entry.no_match += 1,
    }
}

fn finalize_confusion(
    mut map: BTreeMap<String, SystemConfusion>,
) -> BTreeMap<String, SystemConfusion> {
    for entry in map.values_mut() {
        entry.coverage = if entry.total_cases > 0 {
            entry.predicted_cases as f32 / entry.total_cases as f32
        } else {
            0.0
        };
        entry.accuracy = if entry.total_cases > 0 {
            entry.correct as f32 / entry.total_cases as f32
        } else {
            0.0
        };
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use dfps_core::mapping::{MappingSourceVersion, MappingStrategy, MappingThresholds};
    use std::{fs::File, io::BufReader};

    #[test]
    fn load_sample_datasets() {
        for dataset in [
            "pet_ct_small",
            "bronze_pet_ct_small",
            "silver_pet_ct_small",
            "gold_pet_ct_small",
        ] {
            let cases =
                load_dataset(dataset).unwrap_or_else(|_| panic!("dataset {dataset} should load"));
            assert!(!cases.is_empty(), "dataset {dataset} should include cases");
        }
    }

    #[test]
    fn run_eval_counts_correct_cases_and_states() {
        let cases = vec![
            EvalCase {
                system: "http://www.ama-assn.org/go/cpt".into(),
                code: "78815".into(),
                display: "PET with concurrently acquired CT for tumor imaging".into(),
                expected_ncit_id: "NCIT:C19951".into(),
            },
            EvalCase {
                system: "http://loinc.org".into(),
                code: "24606-6".into(),
                display: "FDG uptake PET".into(),
                expected_ncit_id: "NCIT:C17747".into(),
            },
        ];
        let summary = run_eval_with_mapper(&cases, |rows| {
            rows.into_iter()
                .map(|row| {
                    let expected = match row.code.as_deref() {
                        Some("24606-6") => "NCIT:C17747",
                        _ => "NCIT:C19951",
                    };
                    MappingResult {
                        code_element_id: row.sr_id,
                        ncit_id: Some(expected.into()),
                        cui: Some(expected.into()),
                        score: 0.95,
                        strategy: MappingStrategy::Lexical,
                        state: MappingState::AutoMapped,
                        thresholds: MappingThresholds::default(),
                        source_version: MappingSourceVersion::new("v-test", "v-test"),
                        reason: None,
                        license_tier: Some("licensed".into()),
                        source_kind: Some("fhir".into()),
                    }
                })
                .collect()
        });

        assert_eq!(summary.total_cases, 2);
        assert_eq!(summary.correct, 2);
        assert_eq!(summary.incorrect, 0);
        assert!(summary.precision >= 0.99);
        assert!(summary.recall >= 0.99);
        assert!(summary.f1 >= 0.99);
        assert!(summary.accuracy >= 0.99);
        assert!(summary.auto_mapped_total >= 2);
        assert!(summary.auto_mapped_precision >= 0.99);
        assert_eq!(summary.state_counts.get("auto_mapped"), Some(&2));
        assert_eq!(summary.results.len(), 2);
    }

    #[test]
    fn streaming_metrics_match_summary() {
        let cases = FileDatasetStore::default()
            .load_dataset("pet_ct_small")
            .expect("load dataset");
        let summary = run_eval_with_mapper(&cases, map_stub);
        let path = FileDatasetStore::default().dataset_path("pet_ct_small");
        let file = File::open(path).expect("open dataset");
        let metrics = run_eval_streaming_metrics_with_mapper(BufReader::new(file), map_stub, 1)
            .expect("stream metrics");
        let summary_metrics = EvalMetrics::from(&summary);
        assert_eq!(metrics.total_cases, summary_metrics.total_cases);
        assert_eq!(metrics.correct, summary_metrics.correct);
        assert_eq!(metrics.state_counts, summary_metrics.state_counts);
        assert_eq!(metrics.reason_counts, summary_metrics.reason_counts);
    }

    #[test]
    fn fingerprint_consistent_across_chunk_sizes() {
        let store = FileDatasetStore::default();
        let path = store.dataset_path("pet_ct_small");
        let file_full = File::open(&path).expect("open dataset");
        let file_chunked = File::open(&path).expect("open dataset");
        let summary_full =
            run_eval_streaming_with_mapper(BufReader::new(file_full), map_stub, DEFAULT_CHUNK_SIZE)
                .expect("stream eval full");
        let summary_chunked =
            run_eval_streaming_with_mapper(BufReader::new(file_chunked), map_stub, 1)
                .expect("stream eval chunked");
        assert_eq!(
            fingerprint_summary(&summary_full),
            fingerprint_summary(&summary_chunked)
        );
    }

    fn map_stub(rows: Vec<StgSrCodeExploded>) -> Vec<MappingResult> {
        rows.into_iter()
            .map(|row| MappingResult {
                code_element_id: row.sr_id.clone(),
                ncit_id: row.code.clone(),
                cui: None,
                score: 0.9,
                strategy: MappingStrategy::Lexical,
                state: MappingState::AutoMapped,
                thresholds: MappingThresholds::default(),
                source_version: MappingSourceVersion::new("stub", "stub"),
                reason: None,
                license_tier: None,
                source_kind: None,
            })
            .collect()
    }
}
