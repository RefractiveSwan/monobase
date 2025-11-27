use std::collections::HashSet;

use chrono::Utc;
use refractive_swan_eval::DatasetManifest;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UploadDatasetRequest {
    pub manifest: DatasetManifest,
    pub ndjson_b64: String,
}

#[derive(Debug, Default)]
pub struct DatasetNodeRegistry {
    pub last_refresh_iso: Option<String>,
    disabled: HashSet<String>,
}

impl DatasetNodeRegistry {
    pub fn mark_refreshed(&mut self) {
        self.last_refresh_iso = Some(Utc::now().to_rfc3339());
    }

    pub fn enable(&mut self, name: &str) {
        self.disabled.remove(name);
    }

    pub fn disable(&mut self, name: &str) {
        self.disabled.insert(name.to_string());
    }

    pub fn is_disabled(&self, name: &str) -> bool {
        self.disabled.contains(name)
    }
}
