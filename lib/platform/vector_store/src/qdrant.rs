use std::time::Duration;

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::{
    VectorBackend, VectorItem, VectorSearchHit, VectorSearchResult, VectorStore, VectorStoreConfig,
    VectorStoreError,
};

#[derive(Serialize)]
struct CreateCollectionRequest {
    vectors: VectorParams,
}

#[derive(Serialize)]
struct VectorParams {
    size: usize,
    distance: String,
}

#[derive(Serialize)]
struct PointsUpsertRequest<'a> {
    points: &'a [PointRecord<'a>],
}

#[derive(Serialize)]
struct PointRecord<'a> {
    id: &'a str,
    vector: &'a [f32],
    payload: serde_json::Value,
}

#[derive(Serialize)]
struct SearchRequest<'a> {
    vector: &'a [f32],
    limit: usize,
}

#[derive(Deserialize)]
struct SearchResponse {
    result: Vec<SearchHit>,
}

#[derive(Deserialize)]
struct SearchHit {
    id: serde_json::Value,
    score: f32,
}

/// Minimal Qdrant HTTP client implementing `VectorStore`.
pub struct QdrantVectorStore {
    client: Client,
    base_url: String,
}

impl QdrantVectorStore {
    pub fn from_config(config: &VectorStoreConfig) -> Result<Self, VectorStoreError> {
        let url = config
            .url
            .as_ref()
            .ok_or_else(|| VectorStoreError::SearchFailed("missing qdrant url".into()))?;
        let client = Client::builder()
            .timeout(Duration::from_millis(config.health_timeout_ms))
            .build()
            .map_err(|_| VectorStoreError::BackendUnavailable)?;

        Ok(Self {
            client,
            base_url: url.trim_end_matches('/').to_string(),
        })
    }

    fn collection_url(&self, namespace: &str) -> String {
        format!("{}/collections/{}", self.base_url, namespace)
    }

    fn ensure_collection(&self, namespace: &str, dim: usize) -> Result<(), VectorStoreError> {
        let url = self.collection_url(namespace);
        let exists = self
            .client
            .get(&url)
            .send()
            .map(|resp| resp.status().is_success())
            .unwrap_or(false);
        if exists {
            return Ok(());
        }

        let body = CreateCollectionRequest {
            vectors: VectorParams {
                size: dim,
                distance: "Cosine".into(),
            },
        };
        self.client
            .put(&url)
            .json(&body)
            .send()
            .map_err(|err| VectorStoreError::IndexFailed(err.to_string()))?
            .error_for_status()
            .map_err(|err| VectorStoreError::IndexFailed(err.to_string()))?;
        Ok(())
    }
}

impl VectorStore for QdrantVectorStore {
    fn backend(&self) -> VectorBackend {
        VectorBackend::Qdrant
    }

    fn health(&self, _namespace: &str) -> Result<(), VectorStoreError> {
        let url = format!("{}/health", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .map_err(|_| VectorStoreError::BackendUnavailable)?;
        if resp.status().is_success() {
            return Ok(());
        }
        Err(VectorStoreError::BackendUnavailable)
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

        self.ensure_collection(namespace, dim)?;

        let points: Vec<PointRecord> = items
            .iter()
            .map(|item| PointRecord {
                id: item.ref_id.as_str(),
                vector: &item.embedding.vector,
                payload: serde_json::json!({
                    "embedding_version": item.embedding.metadata.embedding_version,
                    "dim": item.embedding.metadata.dim
                }),
            })
            .collect();

        let url = format!("{}/points?wait=true", self.collection_url(namespace));
        self.client
            .put(&url)
            .json(&PointsUpsertRequest {
                points: points.as_slice(),
            })
            .send()
            .map_err(|err| VectorStoreError::IndexFailed(err.to_string()))?
            .error_for_status()
            .map_err(|err| VectorStoreError::IndexFailed(err.to_string()))?;
        Ok(())
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

        let url = format!("{}/points/search", self.collection_url(namespace));
        let response = self
            .client
            .post(&url)
            .json(&SearchRequest {
                vector: query_vec,
                limit: top_k,
            })
            .send()
            .map_err(|err| VectorStoreError::SearchFailed(err.to_string()))?
            .error_for_status()
            .map_err(|err| VectorStoreError::SearchFailed(err.to_string()))?;

        let body: SearchResponse = response
            .json()
            .map_err(|err| VectorStoreError::SearchFailed(err.to_string()))?;
        let hits = body
            .result
            .into_iter()
            .map(|hit| VectorSearchHit {
                ref_id: match hit.id {
                    serde_json::Value::String(id) => id,
                    serde_json::Value::Number(num) => num.to_string(),
                    other => other.to_string(),
                },
                score: hit.score,
            })
            .collect();
        Ok(VectorSearchResult {
            hits,
            capacity: None,
        })
    }
}
