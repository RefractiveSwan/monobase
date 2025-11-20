use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// State assigned to the mapping after thresholds are applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MappingState {
    AutoMapped,
    NeedsReview,
    NoMatch,
}
