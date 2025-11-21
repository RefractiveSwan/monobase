use pgvector::Vector;
use tokio::runtime::Runtime;
use tokio_postgres::{Client, NoTls};

use crate::{
    VectorBackend, VectorItem, VectorSearchHit, VectorSearchResult, VectorStore, VectorStoreConfig,
    VectorStoreError,
};

const CREATE_TABLE_SQL: &str = r#"
CREATE EXTENSION IF NOT EXISTS vector;
CREATE TABLE IF NOT EXISTS ncit_vectors (
  namespace TEXT NOT NULL,
  ref_id TEXT NOT NULL,
  embedding vector NOT NULL,
  embedding_version TEXT,
  dim INT,
  PRIMARY KEY(namespace, ref_id)
);
"#;

pub struct PgVectorStore {
    conn_str: String,
}

impl PgVectorStore {
    pub fn from_config(config: &VectorStoreConfig) -> Result<Self, VectorStoreError> {
        let url = config
            .url
            .as_ref()
            .ok_or_else(|| VectorStoreError::SearchFailed("missing pgvector url".into()))?
            .clone();
        Ok(Self { conn_str: url })
    }

    fn with_client<F, Fut, T>(&self, op: F) -> Result<T, VectorStoreError>
    where
        F: FnOnce(Client) -> Fut,
        Fut: std::future::Future<Output = Result<T, VectorStoreError>>,
    {
        let rt = Runtime::new().map_err(|_| VectorStoreError::BackendUnavailable)?;
        rt.block_on(async {
            let (client, connection) = tokio_postgres::connect(&self.conn_str, NoTls)
                .await
                .map_err(|_| VectorStoreError::BackendUnavailable)?;
            tokio::spawn(async move {
                if let Err(err) = connection.await {
                    eprintln!("pgvector connection error: {err}");
                }
            });
            op(client).await
        })
    }

    async fn ensure_table(&self, client: &Client) -> Result<(), VectorStoreError> {
        client
            .batch_execute(CREATE_TABLE_SQL)
            .await
            .map_err(|err| VectorStoreError::IndexFailed(err.to_string()))
    }
}

impl VectorStore for PgVectorStore {
    fn backend(&self) -> VectorBackend {
        VectorBackend::PgVector
    }

    fn health(&self, _namespace: &str) -> Result<(), VectorStoreError> {
        self.with_client(|client| async move {
            client
                .simple_query("SELECT 1")
                .await
                .map_err(|_| VectorStoreError::BackendUnavailable)?;
            Ok(())
        })
    }

    fn index_items(&self, namespace: &str, items: &[VectorItem]) -> Result<(), VectorStoreError> {
        if items.is_empty() {
            return Ok(());
        }
        let dim = items
            .first()
            .map(|item| item.embedding.vector.len())
            .unwrap_or(0);
        if dim == 0 {
            return Err(VectorStoreError::IndexFailed("empty embedding".into()));
        }
        if !items.iter().all(|item| item.embedding.vector.len() == dim) {
            return Err(VectorStoreError::IndexFailed(
                "dimension mismatch in batch".into(),
            ));
        }

        self.with_client(|client| async move {
            self.ensure_table(&client).await?;
            let stmt = client
                .prepare(
                    "INSERT INTO ncit_vectors(namespace, ref_id, embedding, embedding_version, dim)
                     VALUES ($1, $2, $3, $4, $5)
                     ON CONFLICT (namespace, ref_id)
                     DO UPDATE SET embedding = EXCLUDED.embedding, embedding_version = EXCLUDED.embedding_version, dim = EXCLUDED.dim",
                )
                .await
                .map_err(|err| VectorStoreError::IndexFailed(err.to_string()))?;

            for item in items {
                let vector = Vector::from(item.embedding.vector.clone());
                client
                    .execute(
                        &stmt,
                        &[
                            &namespace,
                            &item.ref_id,
                            &vector,
                            &item.embedding.metadata.embedding_version,
                            &(item.embedding.metadata.dim as i32),
                        ],
                    )
                    .await
                    .map_err(|err| VectorStoreError::IndexFailed(err.to_string()))?;
            }
            Ok(())
        })
    }

    fn search(
        &self,
        namespace: &str,
        query_vec: &[f32],
        top_k: usize,
    ) -> Result<VectorSearchResult, VectorStoreError> {
        if top_k == 0 {
            return Ok(VectorSearchResult::default());
        }
        let query = Vector::from(query_vec.to_vec());
        self.with_client(|client| async move {
            self.ensure_table(&client).await?;
            let stmt = client
                .prepare(
                    "SELECT ref_id, (embedding <-> $2) as distance
                     FROM ncit_vectors
                     WHERE namespace = $1
                     ORDER BY embedding <-> $2
                     LIMIT $3",
                )
                .await
                .map_err(|err| VectorStoreError::SearchFailed(err.to_string()))?;
            let rows = client
                .query(&stmt, &[&namespace, &query, &(top_k as i64)])
                .await
                .map_err(|err| VectorStoreError::SearchFailed(err.to_string()))?;
            let hits = rows
                .into_iter()
                .map(|row| VectorSearchHit {
                    ref_id: row.get::<_, String>("ref_id"),
                    score: 1.0 - row.get::<_, f32>("distance"),
                })
                .collect();
            Ok(VectorSearchResult {
                hits,
                capacity: None,
            })
        })
    }
}
