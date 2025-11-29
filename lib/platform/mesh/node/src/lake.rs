use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Serialize};

/// Minimal warehouse snapshot for future lake exports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarehouseSnapshot {
    pub fact_rows: Vec<serde_json::Value>,
    pub dim_rows: HashMap<String, Vec<serde_json::Value>>,
}

/// Optional lake writer hook (design stub).
pub trait LakeWriter: Send + Sync {
    fn write_snapshot(&self, _snapshot: &WarehouseSnapshot) -> Result<(), LakeError> {
        Err(LakeError::Disabled)
    }
}

/// Optional lake reader hook (design stub).
pub trait LakeReader: Send + Sync {
    fn read_latest(&self) -> Result<Option<WarehouseSnapshot>, LakeError> {
        Ok(None)
    }
}

/// Lake error types.
#[derive(Debug)]
pub enum LakeError {
    Disabled,
    Failed(Box<dyn Error + Send + Sync>),
}
