use refractive_swan_contracts::pipeline::{
    DimNCITConcept, MappingResult, StgServiceRequestFlat, StgSrCodeExploded, ValidationReport,
};
use refractive_swan_observability::VectorUsageSnapshot;
use refractive_swan_web_dto::PipelineOutput;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HealthResponse {
    pub status: String,
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
