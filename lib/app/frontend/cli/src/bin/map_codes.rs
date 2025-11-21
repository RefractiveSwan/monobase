use std::{io::StdoutLock, path::PathBuf};

use clap::Parser;
use refractive_swan_cli::cli_core::{
    CliError, CliResult, enforce_license_blocks, init_cli_env, init_logging, input_reader,
    json_stream, load_policy, load_vector_config, mapping_vector_store, run_bin, write_record,
};
use refractive_swan_core::staging::StgSrCodeExploded;
use refractive_swan_mapping::{
    DeterministicEmbeddingProvider, explain_staging_code,
    map_staging_codes_with_summary_and_policy, map_staging_codes_with_vector_and_policy,
};
use refractive_swan_observability::VectorUsageSnapshot;
use refractive_swan_vector_store::{VectorStore, VectorStoreConfig};
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "map_codes", about = "Map staging codes to NCIt concepts")]
struct Args {
    /// NDJSON file containing staging codes (defaults to stdin)
    #[arg(value_name = "INPUT")]
    input: Option<PathBuf>,
    /// Emit explanation rows (top-N candidates) after each mapping
    #[arg(long)]
    explain: bool,
    /// Number of candidates to include when explaining mappings
    #[arg(long, default_value_t = 5)]
    explain_top: usize,
    /// Exit with error if any code is blocked by compliance policy
    #[arg(long)]
    fail_on_license_block: bool,
    /// Log level for env_logger (error,warn,info,debug,trace)
    #[arg(long, value_name = "LEVEL", default_value = "info")]
    log_level: String,
}

const DEFAULT_BATCH_SIZE: usize = 256;

fn main() {
    run_bin("map_codes", run);
}

fn run() -> CliResult<()> {
    init_cli_env()?;
    let args = Args::parse();
    init_logging(&args.log_level)?;
    let reader = input_reader(args.input.as_ref())?;
    let mut stream = json_stream::<StgSrCodeExploded>(reader);

    let policy = load_policy()?;
    let mapping_engine = match MappingMode::vector_from_env() {
        Ok(engine) => Some(engine),
        Err(err) => {
            log::warn!(
                "vector mapping disabled or failed ({err}); falling back to lexical pipeline"
            );
            None
        }
    };
    let engine = mapping_engine.unwrap_or(MappingMode::Lexical);

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    let mut buffer = Vec::with_capacity(DEFAULT_BATCH_SIZE);
    let mut total_summary = refractive_swan_mapping::MappingSummary::default();
    let mut total_usage: Option<VectorUsageSnapshot> = None;
    let mut license_blocked = 0usize;
    let mut processed = 0usize;

    while let Some(code) = stream.next() {
        buffer.push(code?);
        if buffer.len() >= DEFAULT_BATCH_SIZE {
            processed += process_chunk(
                &mut buffer,
                &engine,
                &policy,
                &args,
                &mut handle,
                &mut total_summary,
                &mut total_usage,
                &mut license_blocked,
            )?;
        }
    }
    processed += process_chunk(
        &mut buffer,
        &engine,
        &policy,
        &args,
        &mut handle,
        &mut total_summary,
        &mut total_usage,
        &mut license_blocked,
    )?;

    if processed == 0 {
        log::warn!("no staging codes detected in input");
    }

    enforce_license_blocks(&policy, license_blocked, args.fail_on_license_block)?;

    if let Some(usage) = total_usage {
        eprintln!(
            "vector_usage queries={} hits={} fallbacks={}",
            usage.queries, usage.hits, usage.fallbacks
        );
    }

    eprintln!(
        "mapping summary total={} by_code_kind={:?} by_license_tier={:?} extern_lookup_success={} extern_lookup_miss={} extern_lookup_error={} license_blocked={} compliance_mode={}",
        total_summary.total,
        total_summary.by_code_kind,
        total_summary.by_license_tier,
        total_summary.extern_lookup_success,
        total_summary.extern_lookup_miss,
        total_summary.extern_lookup_error,
        license_blocked,
        policy.mode.as_str()
    );

    Ok(())
}

fn process_chunk(
    buffer: &mut Vec<StgSrCodeExploded>,
    engine: &MappingMode,
    policy: &refractive_swan_compliance::Policy,
    args: &Args,
    handle: &mut StdoutLock<'_>,
    summary: &mut refractive_swan_mapping::MappingSummary,
    usage: &mut Option<VectorUsageSnapshot>,
    license_blocked: &mut usize,
) -> CliResult<usize> {
    if buffer.is_empty() {
        return Ok(0);
    }
    let chunk: Vec<StgSrCodeExploded> = buffer.drain(..).collect();
    let (results, chunk_summary, chunk_usage) = engine.map_chunk(&chunk, policy)?;
    summary.merge(&chunk_summary);
    accumulate_usage(usage, chunk_usage);

    for result in &results {
        write_record(handle, "mapping_result", result)?;
    }
    *license_blocked += results
        .iter()
        .filter(|res| res.reason.as_deref() == Some("license_blocked"))
        .count();

    if args.explain {
        for code in &chunk {
            let explanation = explain_staging_code(code, args.explain_top);
            write_record(handle, "explanation", &explanation)?;
        }
    }

    Ok(chunk.len())
}

fn accumulate_usage(total: &mut Option<VectorUsageSnapshot>, delta: Option<VectorUsageSnapshot>) {
    if let Some(delta) = delta {
        match total {
            Some(total_usage) => {
                total_usage.queries += delta.queries;
                total_usage.hits += delta.hits;
                total_usage.fallbacks += delta.fallbacks;
                if delta.capacity.is_some() {
                    total_usage.capacity = delta.capacity;
                }
            }
            None => *total = Some(delta),
        }
    }
}

enum MappingMode {
    Vector(VectorMapper),
    Lexical,
}

impl MappingMode {
    fn vector_from_env() -> CliResult<Self> {
        let config = load_vector_config()?;
        if !config.enabled {
            return Err(CliError::config("refractive_swan_VECTOR_ENABLED=false"));
        }
        let store = mapping_vector_store(&config)?;
        Ok(Self::Vector(VectorMapper::new(config, store)))
    }

    fn map_chunk(
        &self,
        codes: &[StgSrCodeExploded],
        policy: &refractive_swan_compliance::Policy,
    ) -> CliResult<(
        Vec<refractive_swan_core::mapping::MappingResult>,
        refractive_swan_mapping::MappingSummary,
        Option<VectorUsageSnapshot>,
    )> {
        match self {
            MappingMode::Vector(runner) => runner
                .map_chunk(codes, policy)
                .map(|(results, summary, usage)| (results, summary, Some(usage))),
            MappingMode::Lexical => {
                let (results, _dims, summary) =
                    map_staging_codes_with_summary_and_policy(codes.to_owned(), policy);
                Ok((results, summary, None))
            }
        }
    }
}

struct VectorMapper {
    store: Arc<ErasedVectorStore>,
    config: VectorStoreConfig,
    embedder: DeterministicEmbeddingProvider,
    top_k: usize,
}

impl VectorMapper {
    fn new(config: VectorStoreConfig, store: Arc<dyn VectorStore>) -> Self {
        let erased = Arc::new(ErasedVectorStore::new(store));
        Self {
            store: erased,
            config,
            embedder: DeterministicEmbeddingProvider::new(),
            top_k: 5,
        }
    }

    fn map_chunk(
        &self,
        codes: &[StgSrCodeExploded],
        policy: &refractive_swan_compliance::Policy,
    ) -> CliResult<(
        Vec<refractive_swan_core::mapping::MappingResult>,
        refractive_swan_mapping::MappingSummary,
        VectorUsageSnapshot,
    )> {
        map_staging_codes_with_vector_and_policy(
            codes.to_owned(),
            Arc::clone(&self.store),
            self.config.clone(),
            self.embedder.clone(),
            self.top_k,
            policy,
        )
        .map(|(results, _dims, summary, usage)| (results, summary, usage))
        .map_err(|err| CliError::external(format!("vector mapping error: {err}")))
    }
}

#[derive(Clone)]
struct ErasedVectorStore(Arc<dyn VectorStore>);

impl ErasedVectorStore {
    fn new(inner: Arc<dyn VectorStore>) -> Self {
        Self(inner)
    }
}

impl VectorStore for ErasedVectorStore {
    fn backend(&self) -> refractive_swan_vector_store::VectorBackend {
        self.0.backend()
    }

    fn health(&self, namespace: &str) -> Result<(), refractive_swan_vector_store::VectorStoreError> {
        self.0.health(namespace)
    }

    fn index_items(
        &self,
        namespace: &str,
        items: &[refractive_swan_vector_store::VectorItem],
    ) -> Result<(), refractive_swan_vector_store::VectorStoreError> {
        self.0.index_items(namespace, items)
    }

    fn search(
        &self,
        namespace: &str,
        query_vec: &[f32],
        top_k: usize,
    ) -> Result<refractive_swan_vector_store::VectorSearchResult, refractive_swan_vector_store::VectorStoreError> {
        self.0.search(namespace, query_vec, top_k)
    }
}
