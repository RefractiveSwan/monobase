//! Pipeline-level contracts re-exported for cross-surface use.

pub use refractive_swan_core::{
    mapping::{
        DimNCITConcept, MappingResult, MappingSourceVersion, MappingState, MappingStrategy,
        MappingThresholds,
    },
    staging::{StgServiceRequestFlat, StgSrCodeExploded},
};
pub use refractive_swan_pipeline::PipelineOutput;
