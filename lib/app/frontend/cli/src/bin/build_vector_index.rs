use std::error::Error;

use clap::Parser;
use dfps_configuration::load_env;
use dfps_core::mapping::CodeElement;
use dfps_mapping::{DeterministicEmbeddingProvider, load_ncit_concepts};
#[cfg(feature = "backend-pgvector")]
use dfps_vector_store::PgVectorStore;
use dfps_vector_store::{
    EmbeddingProvider, MockVectorStore, QdrantVectorStore, VectorBackend, VectorItem, VectorStore,
    VectorStoreConfig,
};

#[derive(Parser)]
#[command(
    name = "build_vector_index",
    about = "Build NCIt vector index for mapping"
)]
struct Args {
    /// Override namespace (defaults to DFPS_VECTOR_NAMESPACE)
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

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    load_env("app.cli").map_err(|err| format!("dfps_cli env error: {err}"))?;
    let args = Args::parse();

    let mut config =
        VectorStoreConfig::from_env().map_err(|err| format!("vector config error: {err}"))?;
    if let Some(namespace) = args.namespace {
        config.namespace = namespace;
    }

    let namespace = config.namespace.clone();
    let embedder = DeterministicEmbeddingProvider::new();
    let concepts = load_ncit_concepts();
    let items: Vec<VectorItem> = concepts
        .into_iter()
        .take(args.limit.unwrap_or(usize::MAX))
        .map(|(concept, _)| {
            let code = CodeElement::new(
                concept.ncit_id.clone(),
                Some("NCIT".into()),
                Some(concept.ncit_id.clone()),
                Some(concept.preferred_name.clone()),
            );
            let mut embedding = embedder.embed(&code);
            embedding.metadata.embedding_version = args.embedding_version.clone();
            if let Some(max_dim) = args.max_dim {
                if embedding.vector.len() > max_dim {
                    panic!(
                        "embedding dimension {} exceeds max_dim {}",
                        embedding.vector.len(),
                        max_dim
                    );
                }
            }
            VectorItem {
                ref_id: concept.ncit_id,
                embedding,
            }
        })
        .collect();

    match config.backend {
        VectorBackend::Qdrant => {
            let store = QdrantVectorStore::from_config(&config)
                .map_err(|err| format!("qdrant client error: {err}"))?;
            store
                .health(&namespace)
                .map_err(|err| format!("qdrant health failed: {err}"))?;
            if args.force_rebuild {
                log::info!("force rebuild requested; collection will be recreated if missing");
            }
            store
                .index_items(&namespace, &items)
                .map_err(|err| format!("indexing failed: {err}"))?;
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
                .map_err(|err| format!("mock index failed: {err}"))?;
            println!(
                "Mock index built with {} items for namespace '{}'",
                items.len(),
                namespace
            );
            // stats for determinism/debugging
            if let Some(first) = items.first() {
                let norms: Vec<f32> = items
                    .iter()
                    .map(|item| (item.embedding.vector.iter().map(|v| v * v).sum::<f32>()).sqrt())
                    .collect();
                let count = norms.len() as f32;
                let mean = norms.iter().sum::<f32>() / count.max(1.0);
                let mut sorted = norms.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let median = if sorted.is_empty() {
                    0.0
                } else {
                    let mid = sorted.len() / 2;
                    sorted[mid]
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
                    args.embedding_version, first.embedding.metadata.dim
                );
            }
        }
        VectorBackend::PgVector => {
            #[cfg(feature = "backend-pgvector")]
            {
                let store = PgVectorStore::from_config(&config)
                    .map_err(|err| format!("pgvector client error: {err}"))?;
                store
                    .health(&namespace)
                    .map_err(|err| format!("pgvector health failed: {err}"))?;
                if args.force_rebuild {
                    log::info!(
                        "force rebuild requested; existing rows will be replaced via upsert"
                    );
                }
                store
                    .index_items(&namespace, &items)
                    .map_err(|err| format!("indexing failed: {err}"))?;
                println!(
                    "Indexed {} NCIt concepts into namespace '{}' using pgvector backend",
                    items.len(),
                    namespace
                );
            }
            #[cfg(not(feature = "backend-pgvector"))]
            {
                return Err("pgvector backend not compiled in this build".into());
            }
        }
        other => {
            return Err(format!(
                "backend '{:?}' not yet supported by build-vector-index",
                other
            )
            .into());
        }
    }

    Ok(())
}
