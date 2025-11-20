use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Threshold configuration used to derive the `MappingState`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MappingThresholds {
    pub auto_map_min: f32,
    pub needs_review_min: f32,
}

impl Default for MappingThresholds {
    fn default() -> Self {
        Self {
            auto_map_min: 0.95,
            needs_review_min: 0.60,
        }
    }
}
