use serde::{Deserialize, Serialize};

/// Strategies used by the mapping engine. Keeps provenance readable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MappingStrategy {
    Lexical,
    Vector,
    Rule,
    Composite,
    Manual,
    Unmapped,
}
