//! Warehouse/load contracts shared across CLI, API, and analytics surfaces.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Summary of rows inserted or updated during a warehouse load.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoadSummary {
    pub patients: u64,
    pub encounters: u64,
    pub codes: u64,
    pub ncit: u64,
    pub facts: u64,
}

impl LoadSummary {
    /// Merge another summary into this one.
    pub fn accumulate(&mut self, other: &LoadSummary) {
        self.patients += other.patients;
        self.encounters += other.encounters;
        self.codes += other.codes;
        self.ncit += other.ncit;
        self.facts += other.facts;
    }
}
