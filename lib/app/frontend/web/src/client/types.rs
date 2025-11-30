use refractive_swan_contracts::pipeline::{
    DimNCITConcept, MappingResult, StgServiceRequestFlat, StgSrCodeExploded, ValidationReport,
};
use refractive_swan_observability::VectorUsageSnapshot;
use refractive_swan_web_dto::{NodeView, PipelineOutput};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HealthResponse {
    pub status: String,
    #[serde(default)]
    pub cache_backend: Option<String>,
    #[serde(default)]
    pub cache: Option<Value>,
    #[serde(default)]
    pub dp_budget_remaining: Option<f64>,
    #[serde(default)]
    pub dp_budget_status: Option<String>,
    #[serde(default)]
    pub metrics: Option<Value>,
    #[serde(default)]
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HubNodesResponse {
    pub nodes: Vec<NodeView>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CohortFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ncit_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_to: Option<String>,
}

impl CohortFilters {
    pub fn to_query_string(&self) -> String {
        serde_urlencoded::to_string(self).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MapBundlesResponse {
    pub flats: Vec<StgServiceRequestFlat>,
    pub exploded_codes: Vec<StgSrCodeExploded>,
    pub mapping_results: Vec<MappingResult>,
    pub dim_concepts: Vec<DimNCITConcept>,
    pub vector_usage: Option<VectorUsageSnapshot>,
    pub validation_reports: Vec<ValidationReport>,
}

impl MapBundlesResponse {
    pub fn new(output: PipelineOutput, validation_reports: Vec<ValidationReport>) -> Self {
        Self {
            flats: output.flats,
            exploded_codes: output.exploded_codes,
            mapping_results: output.mapping_results,
            dim_concepts: output.dim_concepts,
            vector_usage: output.vector_usage,
            validation_reports,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MeshFeatureToggles {
    pub vector: FeatureTogglesView,
    pub mesh: MeshTogglesView,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeatureTogglesView {
    pub vector_enabled: bool,
    pub mesh_node_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MeshTogglesView {
    pub mesh_enabled: bool,
    pub hub_enabled: bool,
    pub hub_url: Option<String>,
    pub node_id: Option<String>,
    pub cache_backend: Option<String>,
    pub warehouse_backend: Option<String>,
}
