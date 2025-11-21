use std::sync::Arc;

use refractive_swan_compliance::Policy;
use refractive_swan_datamart::{DatamartSink, SqliteDatamart};
use refractive_swan_eval::DatasetStore;
use refractive_swan_mesh_dto::MeshNodeId;
use refractive_swan_observability::PipelineMetrics;
use refractive_swan_pipeline::{DefaultPipeline, PipelinePort, VectorPipelineContext};
use tokio::sync::Mutex;

use crate::{config::NodePlaneConfig, vector::vector_context_from_config};

/// Shared orchestration surface for node-local data plane operations.
///
/// This struct wires the domain pipeline, datamart sink, dataset store, and
/// optional vector context together so HTTP/gRPC adapters can remain thin.
pub struct NodeDataPlane {
    node_id: MeshNodeId,
    policy: Policy,
    vector_context: Option<VectorPipelineContext>,
    dataset_store: Arc<dyn DatasetStore + Send + Sync>,
    metrics: Arc<Mutex<PipelineMetrics>>,
    pipeline: Arc<dyn PipelinePort + Send + Sync>,
    datamart: Arc<dyn DatamartSink + Send + Sync>,
}

impl NodeDataPlane {
    /// Build a plane using defaults for pipeline/datamart/vector contexts.
    pub fn from_config(node_id: MeshNodeId, config: NodePlaneConfig) -> Self {
        let NodePlaneConfig {
            policy,
            dataset_store,
            datamart,
            vector,
        } = config;
        let dataset_store: Arc<dyn DatasetStore + Send + Sync> = Arc::new(dataset_store);
        let vector_context = vector.as_ref().and_then(vector_context_from_config);
        let pipeline: Arc<dyn PipelinePort + Send + Sync> = Arc::new(DefaultPipeline);
        let datamart: Arc<dyn DatamartSink + Send + Sync> =
            Arc::new(SqliteDatamart::from_optional_config(datamart));
        Self::new(
            node_id,
            policy,
            dataset_store,
            pipeline,
            datamart,
            vector_context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        node_id: MeshNodeId,
        policy: Policy,
        dataset_store: Arc<dyn DatasetStore + Send + Sync>,
        pipeline: Arc<dyn PipelinePort + Send + Sync>,
        datamart: Arc<dyn DatamartSink + Send + Sync>,
        vector_context: Option<VectorPipelineContext>,
    ) -> Self {
        Self {
            node_id,
            policy,
            vector_context,
            dataset_store,
            metrics: Arc::new(Mutex::new(PipelineMetrics::default())),
            pipeline,
            datamart,
        }
    }

    /// Node identifier advertised to the mesh hub.
    pub fn node_id(&self) -> &MeshNodeId {
        &self.node_id
    }

    /// Compliance policy enforced for pipeline + analytics jobs.
    pub fn policy(&self) -> &Policy {
        &self.policy
    }

    /// Mutable access to the compliance policy (tests/config updates).
    pub fn policy_mut(&mut self) -> &mut Policy {
        &mut self.policy
    }

    /// Optional vector context supplied by the platform/store layer.
    pub fn vector_context(&self) -> Option<VectorPipelineContext> {
        self.vector_context.clone()
    }

    /// Cloneable metrics handle for HTTP adapters.
    pub fn metrics(&self) -> Arc<Mutex<PipelineMetrics>> {
        Arc::clone(&self.metrics)
    }

    pub fn dataset_store(&self) -> Arc<dyn DatasetStore + Send + Sync> {
        Arc::clone(&self.dataset_store)
    }

    pub fn datamart(&self) -> Arc<dyn DatamartSink + Send + Sync> {
        Arc::clone(&self.datamart)
    }

    pub fn pipeline(&self) -> Arc<dyn PipelinePort + Send + Sync> {
        Arc::clone(&self.pipeline)
    }
}
