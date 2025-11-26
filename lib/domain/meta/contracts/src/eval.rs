//! Eval contracts aligning CLI/API outputs.

pub use refractive_swan_eval::{DatasetManifest, DatasetTier, EvalSummary};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Payload returned when running an eval job.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvalRunResponse {
    pub dataset: String,
    pub manifest: Option<DatasetManifest>,
    pub summary: EvalSummary,
}
