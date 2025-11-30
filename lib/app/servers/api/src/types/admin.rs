use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TerminologyInsights {
    pub total_systems: usize,
    pub licensed: usize,
    pub open: usize,
    pub ontologies: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceView {
    pub mode: String,
    pub actions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IngestionSummary {
    pub bundles: usize,
    pub mapping_count: usize,
    pub license_blocked: usize,
    pub vector_queries: usize,
    pub vector_hits: usize,
    pub vector_fallbacks: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VectorStatusView {
    pub backend: String,
    pub namespace: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatamartHealthView {
    pub status: String,
    pub detail: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeatureTogglesView {
    pub vector_enabled: bool,
    pub mesh_node_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshFeatureTogglesView {
    pub vector: FeatureTogglesView,
    pub mesh: refractive_swan_web_dto::MeshTogglesView,
}
