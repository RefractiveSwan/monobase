use std::path::PathBuf;

use clap::Parser;
use dfps_cli::cli_core::{
    CliError, CliResult, enforce_license_blocks, init_cli_env, input_reader, json_stream,
    load_policy, load_vector_config, run_bin, write_record,
};
use dfps_core::staging::StgSrCodeExploded;
use dfps_mapping::{
    DeterministicEmbeddingProvider, explain_staging_code,
    map_staging_codes_with_summary_and_policy, map_staging_codes_with_vector_and_policy,
};
use dfps_observability::VectorUsageSnapshot;
#[cfg(feature = "backend-pgvector")]
use dfps_vector_store::PgVectorStore;
use dfps_vector_store::{MockVectorStore, QdrantVectorStore, VectorBackend};

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
}

fn main() {
    run_bin("map_codes", run);
}

fn run() -> CliResult<()> {
    init_cli_env()?;
    let args = Args::parse();
    let reader = input_reader(args.input.as_ref())?;
    let mut stream = json_stream::<StgSrCodeExploded>(reader);
    let mut codes = Vec::new();
    while let Some(code) = stream.next() {
        codes.push(code?);
    }

    if codes.is_empty() {
        log::warn!("no staging codes detected in input");
    }

    let policy = load_policy()?;
    let vector_mapping = try_vector_mapping(&codes, &policy);
    let (results, summary, usage) = match vector_mapping {
        Ok(value) => value,
        Err(err) => {
            log::warn!(
                "vector mapping disabled or failed ({err}); falling back to lexical pipeline"
            );
            let (results, _, summary) =
                map_staging_codes_with_summary_and_policy(codes.clone(), &policy);
            (results, summary, None)
        }
    };

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    for result in &results {
        write_record(&mut handle, "mapping_result", result)?;
    }

    if args.explain {
        for code in &codes {
            let explanation = explain_staging_code(code, args.explain_top);
            write_record(&mut handle, "explanation", &explanation)?;
        }
    }

    let license_blocked = results
        .iter()
        .filter(|res| res.reason.as_deref() == Some("license_blocked"))
        .count();

    enforce_license_blocks(&policy, license_blocked, args.fail_on_license_block)?;

    if let Some(usage) = usage {
        eprintln!(
            "vector_usage queries={} hits={} fallbacks={}",
            usage.queries, usage.hits, usage.fallbacks
        );
    }

    eprintln!(
        "mapping summary total={} by_code_kind={:?} by_license_tier={:?} extern_lookup_success={} extern_lookup_miss={} extern_lookup_error={} license_blocked={} compliance_mode={}",
        summary.total,
        summary.by_code_kind,
        summary.by_license_tier,
        summary.extern_lookup_success,
        summary.extern_lookup_miss,
        summary.extern_lookup_error,
        license_blocked,
        policy.mode.as_str()
    );

    Ok(())
}

fn try_vector_mapping(
    codes: &[StgSrCodeExploded],
    policy: &dfps_compliance::Policy,
) -> CliResult<(
    Vec<dfps_core::mapping::MappingResult>,
    dfps_mapping::MappingSummary,
    Option<VectorUsageSnapshot>,
)> {
    let config = load_vector_config()?;
    if !config.enabled {
        return Err(CliError::config("DFPS_VECTOR_ENABLED=false"));
    }

    match config.backend {
        VectorBackend::Qdrant => {
            let client = QdrantVectorStore::from_config(&config)
                .map_err(|err| CliError::external(format!("qdrant client: {err}")))?;
            let store = std::sync::Arc::new(client);
            map_staging_codes_with_vector_and_policy(
                codes.to_owned(),
                store,
                config,
                DeterministicEmbeddingProvider::new(),
                5,
                policy,
            )
            .map(|(results, _dims, summary, usage)| (results, summary, Some(usage)))
            .map_err(|err| CliError::external(format!("vector mapping error: {err}")))
        }
        VectorBackend::Mock => {
            let store = std::sync::Arc::new(MockVectorStore::new(config.namespace.clone()));
            map_staging_codes_with_vector_and_policy(
                codes.to_owned(),
                store,
                config,
                DeterministicEmbeddingProvider::new(),
                5,
                policy,
            )
            .map(|(results, _dims, summary, usage)| (results, summary, Some(usage)))
            .map_err(|err| CliError::external(format!("vector mapping error: {err}")))
        }
        VectorBackend::PgVector => {
            #[cfg(feature = "backend-pgvector")]
            {
                let client = PgVectorStore::from_config(&config)
                    .map_err(|err| CliError::external(format!("pgvector client: {err}")))?;
                let store = std::sync::Arc::new(client);
                map_staging_codes_with_vector_and_policy(
                    codes.to_owned(),
                    store,
                    config,
                    DeterministicEmbeddingProvider::new(),
                    5,
                    policy,
                )
                .map(|(results, _dims, summary, usage)| (results, summary, Some(usage)))
                .map_err(|err| CliError::external(format!("vector mapping error: {err}")))
            }
            #[cfg(not(feature = "backend-pgvector"))]
            {
                Err(CliError::config(
                    "pgvector backend not compiled; enable backend-pgvector feature",
                ))
            }
        }
        other => Err(CliError::config(format!(
            "backend '{other:?}' not supported in CLI"
        ))),
    }
}
