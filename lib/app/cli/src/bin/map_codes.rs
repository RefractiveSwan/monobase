use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;

use clap::Parser;
use dfps_configuration::load_env;
use dfps_core::staging::StgSrCodeExploded;
use dfps_mapping::{
    DeterministicEmbeddingProvider, explain_staging_code, map_staging_codes_with_summary,
    map_staging_codes_with_vector,
};
use dfps_vector_store::{MockVectorStore, QdrantVectorStore, VectorBackend, VectorStoreConfig};

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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_env("app.cli").map_err(|err| format!("dfps_cli env error: {err}"))?;
    let args = Args::parse();
    let reader: Box<dyn BufRead> = match &args.input {
        Some(path) => Box::new(BufReader::new(File::open(path)?)),
        None => Box::new(BufReader::new(io::stdin())),
    };

    let mut codes = Vec::new();
    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let code: StgSrCodeExploded = serde_json::from_str(trimmed)?;
        codes.push(code);
    }

    let vector_mapping = try_vector_mapping(&codes);
    let (results, summary) = match vector_mapping {
        Ok((results, _dims, summary, usage)) => {
            if let Some(usage) = usage {
                eprintln!(
                    "vector_usage queries={} hits={} fallbacks={}",
                    usage.queries, usage.hits, usage.fallbacks
                );
            }
            (results, summary)
        }
        Err(err) => {
            log::warn!("vector mapping disabled or failed ({err}); using offline mock");
            let (results, _, summary) = map_staging_codes_with_summary(codes.clone());
            (results, summary)
        }
    };
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    for result in results {
        writeln!(handle, "{}", serde_json::to_string(&result)?)?;
    }

    if args.explain {
        for code in &codes {
            let explanation = explain_staging_code(code, args.explain_top);
            writeln!(
                handle,
                "{}",
                serde_json::to_string(&serde_json::json!({
                    "kind": "explanation",
                    "value": explanation
                }))?
            )?;
        }
    }

    eprintln!(
        "mapping summary total={} by_code_kind={:?} by_license_tier={:?}",
        summary.total, summary.by_code_kind, summary.by_license_tier
    );

    Ok(())
}

fn try_vector_mapping(
    codes: &[StgSrCodeExploded],
) -> Result<
    (
        Vec<dfps_core::mapping::MappingResult>,
        Vec<dfps_core::mapping::DimNCITConcept>,
        dfps_mapping::MappingSummary,
        Option<dfps_vector_store::VectorUsageSnapshot>,
    ),
    String,
> {
    let config =
        VectorStoreConfig::from_env().map_err(|err| format!("vector config error: {err}"))?;
    if !config.enabled {
        return Err("DFPS_VECTOR_ENABLED=false".into());
    }
    match config.backend {
        VectorBackend::Qdrant => {
            let client = QdrantVectorStore::from_config(&config)
                .map_err(|err| format!("qdrant client: {err}"))?;
            let store = std::sync::Arc::new(client);
            map_staging_codes_with_vector(
                codes.to_owned(),
                store,
                config,
                DeterministicEmbeddingProvider::new(),
                5,
            )
            .map(|(results, dims, summary, usage)| (results, dims, summary, Some(usage)))
            .map_err(|err| format!("vector mapping error: {err}"))
        }
        VectorBackend::PgVector => {
            #[cfg(feature = "backend-pgvector")]
            {
                let client = dfps_vector_store::PgVectorStore::from_config(&config)
                    .map_err(|err| format!("pgvector client: {err}"))?;
                let store = std::sync::Arc::new(client);
                map_staging_codes_with_vector(
                    codes.to_owned(),
                    store,
                    config,
                    DeterministicEmbeddingProvider::new(),
                    5,
                )
                .map(|(results, dims, summary, usage)| (results, dims, summary, Some(usage)))
                .map_err(|err| format!("vector mapping error: {err}"))
            }
            #[cfg(not(feature = "backend-pgvector"))]
            {
                Err("pgvector backend not compiled; enable feature".into())
            }
        }
        VectorBackend::Mock => {
            let store = std::sync::Arc::new(MockVectorStore::new(config.namespace.clone()));
            map_staging_codes_with_vector(
                codes.to_owned(),
                store,
                config,
                DeterministicEmbeddingProvider::new(),
                5,
            )
            .map(|(results, dims, summary, usage)| (results, dims, summary, Some(usage)))
            .map_err(|err| format!("vector mapping error: {err}"))
        }
        other => Err(format!("backend {:?} not supported in CLI", other)),
    }
}
