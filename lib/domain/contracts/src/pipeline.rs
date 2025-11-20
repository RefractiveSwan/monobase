//! Pipeline-level contracts re-exported for cross-surface use.

pub use dfps_core::{
    mapping::{
        DimNCITConcept, MappingResult, MappingSourceVersion, MappingState, MappingStrategy,
        MappingThresholds,
    },
    staging::{StgServiceRequestFlat, StgSrCodeExploded},
};
pub use dfps_pipeline::PipelineOutput;
