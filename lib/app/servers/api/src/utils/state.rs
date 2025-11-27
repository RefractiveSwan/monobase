use std::sync::Arc;

use chrono::Utc;
use refractive_swan_contracts::{AdminEvent, AdminEventKind};
use refractive_swan_mesh_dto::MeshNodeId;
use refractive_swan_mesh_node::{NodeDataPlane, NodePlaneConfig};
use refractive_swan_web_dto::EvalRunResponse;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::server::handlers::DatasetNodeRegistry;

const ADMIN_EVENTS_LIMIT: usize = 100;

/// Application node state that wires domain ports + adapters (pipeline, datamart,
/// datasets, metrics, compliance policy) for handlers.
#[derive(Clone)]
pub struct ApiState {
    pub(crate) plane: Arc<NodeDataPlane>,
    pub(crate) latest_eval: Arc<Mutex<Option<EvalRunResponse>>>,
    pub(crate) dataset_registry: Arc<Mutex<DatasetNodeRegistry>>,
    pub(crate) admin_events: Arc<Mutex<Vec<AdminEvent>>>,
}

impl ApiState {
    pub fn from_plane_config(config: NodePlaneConfig) -> Self {
        let plane = NodeDataPlane::from_config(MeshNodeId::new_random(), config);
        Self {
            plane: Arc::new(plane),
            latest_eval: Arc::new(Mutex::new(None)),
            dataset_registry: Arc::new(Mutex::new(DatasetNodeRegistry::default())),
            admin_events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(crate) async fn record_admin_event(
        &self,
        kind: AdminEventKind,
        message: impl Into<String>,
    ) {
        let event = AdminEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            kind,
            message: message.into(),
            mesh_node_id: Some(self.plane.node_id().to_string()),
        };
        let mut events = self.admin_events.lock().await;
        events.insert(0, event);
        if events.len() > ADMIN_EVENTS_LIMIT {
            events.truncate(ADMIN_EVENTS_LIMIT);
        }
    }

    pub(crate) async fn admin_events_snapshot(&self) -> Vec<AdminEvent> {
        self.admin_events.lock().await.clone()
    }
}

impl Default for ApiState {
    fn default() -> Self {
        let config = NodePlaneConfig::from_env("app.web.api")
            .expect("refractive_swan_api: failed to load plane configuration");
        Self::from_plane_config(config)
    }
}
