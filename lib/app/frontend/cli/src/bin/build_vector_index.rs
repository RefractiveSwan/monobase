use clap::Parser;
use refractive_swan_cli::cli_core::{CliError, CliResult, init_cli_env, load_vector_config, run_bin};
use refractive_swan_core::mapping::CodeElement;
use refractive_swan_mapping::{DeterministicEmbeddingProvider, load_ncit_concepts};
use refractive_swan_vector_store::EmbeddingProvider;
#[cfg(feature = "backend-pgvector")]
use refractive_swan_vector_store::PgVectorStore;
use refractive_swan_vector_store::{
    MockVectorStore, QdrantVectorStore, VectorBackend, VectorItem, VectorStore,
};

#[derive(Parser)]
#[command(
    name = "build_vector_index",
    about = "Build NCIt vector index for mapping"
)]
struct Args {
    /// Override namespace (defaults to refractive_swan_VECTOR_NAMESPACE)
    #[arg(long)]
    namespace: Option<String>,
    /// Limit how many NCIt concepts to index (useful for smoke runs)
    #[arg(long)]
    limit: Option<usize>,
    /// Override embedding version label for auditability
    #[arg(long, default_value = "deterministic-hash-v1")]
    embedding_version: String,
    /// Maximum allowed embedding dimension (errors if exceeded)
    #[arg(long)]
    max_dim: Option<usize>,
    /// Force rebuild (best-effort; skipped for mock backend)
    #[arg(long)]
    force_rebuild: bool,
}

fn main() {
    run_bin("build_vector_index", run);
}

fn run() -> CliResult<()> {
    init_cli_env()?;
    env_logger::init();
    let args = Args::parse();

    let mut config = load_vector_config()?;
    if let Some(namespace) = &args.namespace {
        config.namespace = namespace.clone();
    }
    if !config.enabled {
        return Err(CliError::config(
            "refractive_swan_VECTOR_ENABLED=false; set to true before building the index",
        ));
    }

    let items = build_items(&args)?;
    let namespace = config.namespace.clone();

    match config.backend {
        VectorBackend::Qdrant => {
            let store = QdrantVectorStore::from_config(&config)
                .map_err(|err| CliError::external(format!("qdrant client error: {err}")))?;
            store
                .health(&namespace)
                .map_err(|err| CliError::external(format!("qdrant health failed: {err}")))?;
            if args.force_rebuild {
                log::info!("force rebuild requested; collection will be recreated if needed");
            }
            store
                .index_items(&namespace, &items)
                .map_err(|err| CliError::external(format!("indexing failed: {err}")))?;
            println!(
                "Indexed {} NCIt concepts into namespace '{}' using Qdrant backend",
                items.len(),
                namespace
            );
        }
        VectorBackend::Mock => {
            let store = MockVectorStore::new(namespace.clone());
            store
                .index_items(&namespace, &items)
                .map_err(|err| CliError::external(format!("mock index failed: {err}")))?;
            println!(
                "Mock index built with {} items for namespace '{}'",
                items.len(),
                namespace
            );
            log_embedding_stats(&items, &args.embedding_version);
        }
        VectorBackend::PgVector => {
            #[cfg(feature = "backend-pgvector")]
            {
                let store = PgVectorStore::from_config(&config)
                    .map_err(|err| CliError::external(format!("pgvector client error: {err}")))?;
                store
                    .health(&namespace)
                    .map_err(|err| CliError::external(format!("pgvector health failed: {err}")))?;
                if args.force_rebuild {
                    log::info!(
                        "force rebuild requested; existing rows will be replaced via upsert"
                    );
                }
                store
                    .index_items(&namespace, &items)
                    .map_err(|err| CliError::external(format!("indexing failed: {err}")))?;
                println!(
                    "Indexed {} NCIt concepts into namespace '{}' using pgvector backend",
                    items.len(),
                    namespace
                );
            }
            #[cfg(not(feature = "backend-pgvector"))]
            {
                return Err(CliError::config(
                    "pgvector backend not compiled; rebuild with backend-pgvector feature"
                        .to_string(),
                ));
            }
        }
        other => {
            return Err(CliError::config(format!(
                "backend '{other:?}' not yet supported by build-vector-index"
            )));
        }
    }

    Ok(())
}

fn build_items(args: &Args) -> CliResult<Vec<VectorItem>> {
    let embedder = DeterministicEmbeddingProvider::new();
    let mut items = Vec::new();
    for (concept, _) in load_ncit_concepts()
        .into_iter()
        .take(args.limit.unwrap_or(usize::MAX))
    {
        let code = CodeElement::new(
            concept.ncit_id.clone(),
            Some("NCIT".into()),
            Some(concept.ncit_id.clone()),
            Some(concept.preferred_name.clone()),
        );
        let mut embedding = embedder.embed(&code);
        if let Some(max_dim) = args.max_dim {
            if embedding.vector.len() > max_dim {
                return Err(CliError::invalid(format!(
                    "embedding dimension {} exceeds max_dim {}",
                    embedding.vector.len(),
                    max_dim
                )));
            }
        }
        embedding.metadata.embedding_version = args.embedding_version.clone();
        items.push(VectorItem {
            ref_id: concept.ncit_id,
            embedding,
        });
    }
    Ok(items)
}

fn log_embedding_stats(items: &[VectorItem], version: &str) {
    if let Some(first) = items.first() {
        let norms: Vec<f32> = items
            .iter()
            .map(|item| (item.embedding.vector.iter().map(|v| v * v).sum::<f32>()).sqrt())
            .collect();
        let count = norms.len() as f32;
        let mean = if count > 0.0 {
            norms.iter().sum::<f32>() / count
        } else {
            0.0
        };
        let mut sorted = norms.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = if sorted.is_empty() {
            0.0
        } else {
            sorted[sorted.len() / 2]
        };
        let sum = norms.iter().sum::<f32>();
        let sum_sq = norms.iter().map(|n| n * n).sum::<f32>();
        let participation_ratio = if sum_sq > 0.0 && count > 0.0 {
            (sum * sum) / (count * sum_sq)
        } else {
            0.0
        };
        println!(
            "Embedding stats: count={} mean_norm={:.4} median_norm={:.4} participation_ratio={:.4}",
            norms.len(),
            mean,
            median,
            participation_ratio
        );
        println!(
            "Embedding version='{}' dim={}",
            version, first.embedding.metadata.dim
        );
    }
}
